use crate::err;
use crate::player::player_structs::Player;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use ringbuf::HeapRb;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::Decoder;
use symphonia::core::errors::Error;
use symphonia::core::formats::SeekMode;
use symphonia::core::formats::{FormatReader, SeekTo};
use symphonia::core::units::Time;

pub struct MusicPlay {
    stream: Stream,
    seeking: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    volume: Arc<parking_lot::Mutex<f32>>,
    position: Arc<AtomicU64>,
    finished: Arc<AtomicBool>,
    format: Arc<Mutex<Box<dyn FormatReader>>>,
    decoder: Arc<Mutex<Box<dyn Decoder>>>,
    track_id: u32,
    sample_rate: u32,
    output_sample_rate: u32,
}
unsafe impl Send for MusicPlay {}

impl Player {
    pub fn play(self) -> MusicPlay {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap_or_else(|| {
            err!("No output device available");
            std::process::exit(1);
        });
        let device_config = device.default_output_config().unwrap();

        let config = StreamConfig {
            channels: device_config.channels(),
            sample_rate: device_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let output_sample_rate = device_config.sample_rate().0;
        let input_sample_rate = self.sample_rate;

        let paused = Arc::new(AtomicBool::new(false));
        let volume = Arc::new(parking_lot::Mutex::new(1.0f32));
        let position = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicBool::new(false));
        let seeking = Arc::new(AtomicBool::new(false));

        let format = Arc::clone(&self.format);
        let decoder = Arc::clone(&self.decoder);
        let track_id = self.track_id;

        let (mut producer, mut consumer) =
            HeapRb::<f32>::new(output_sample_rate as usize).split();

        let paused_clone = Arc::clone(&paused);
        let position_clone = Arc::clone(&position);
        let finished_clone = Arc::clone(&finished);
        let seeking_clone = Arc::clone(&seeking);
        let channels = self.channels as usize;

        let paused_stream = Arc::clone(&paused);
        let volume_stream = Arc::clone(&volume);

        std::thread::spawn(move || {
            let mut current_samples = Vec::new();
            let mut current_index = 0;
            let mut frames_played = 0u64;

            // リサンプラーの初期化
            let needs_resampling = input_sample_rate != output_sample_rate;
            let mut resampler: Option<SincFixedIn<f32>> = if needs_resampling {
                let params = SincInterpolationParameters {
                    sinc_len: 256,
                    f_cutoff: 0.95,
                    interpolation: SincInterpolationType::Linear,
                    oversampling_factor: 256,
                    window: WindowFunction::BlackmanHarris2,
                };
                
                match SincFixedIn::new(
                    output_sample_rate as f64 / input_sample_rate as f64,
                    2.0,
                    params,
                    1024,
                    channels,
                ) {
                    Ok(r) => {
                        eprintln!("Resampling: {}Hz -> {}Hz", input_sample_rate, output_sample_rate);
                        Some(r)
                    }
                    Err(e) => {
                        err!("Failed to create resampler: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            loop {
                if finished_clone.load(Ordering::Relaxed) {
                    break;
                }

                if seeking_clone.load(Ordering::Relaxed) {
                    current_samples.clear();
                    current_index = 0;
                    
                    // リサンプラーのリセット
                    if let Some(ref mut r) = resampler {
                        r.reset();
                    }
                    
                    frames_played = position_clone.load(Ordering::Relaxed) * output_sample_rate as u64;
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }

                // Fill buffer
                while producer.free_len() > 2048 {
                    if current_index >= current_samples.len() {
                        // Decode next packet
                        let mut format = format.lock().unwrap();
                        let mut decoder = decoder.lock().unwrap();

                        let packet = match format.next_packet() {
                            Ok(packet) => packet,
                            Err(Error::IoError(e))
                                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                            {
                                finished_clone.store(true, Ordering::Relaxed);
                                break;
                            }
                            Err(_) => {
                                finished_clone.store(true, Ordering::Relaxed);
                                break;
                            }
                        };

                        if packet.track_id() != track_id {
                            continue;
                        }

                        match decoder.decode(&packet) {
                            Ok(decoded) => {
                                let samples = convert_samples(decoded);
                                
                                // リサンプリングが必要な場合
                                if let Some(ref mut resampler) = resampler {
                                    let frames = samples.len() / channels;
                                    
                                    // チャンネルごとに分離
                                    let mut channel_data = vec![vec![0.0f32; frames]; channels];
                                    for (i, &sample) in samples.iter().enumerate() {
                                        let ch = i % channels;
                                        let frame = i / channels;
                                        channel_data[ch][frame] = sample;
                                    }

                                    // リサンプリング実行
                                    match resampler.process(&channel_data, None) {
                                        Ok(resampled) => {
                                            // インターリーブ
                                            let out_frames = resampled[0].len();
                                            let mut interleaved = Vec::with_capacity(out_frames * channels);
                                            
                                            for frame in 0..out_frames {
                                                for ch in 0..channels {
                                                    interleaved.push(resampled[ch][frame]);
                                                }
                                            }
                                            
                                            current_samples = interleaved;
                                        }
                                        Err(e) => {
                                            err!("Resampling error: {}", e);
                                            current_samples = samples;
                                        }
                                    }
                                } else {
                                    current_samples = samples;
                                }
                                
                                current_index = 0;
                            }
                            Err(_) => continue,
                        }
                    }

                    if current_index < current_samples.len() {
                        let sample = current_samples[current_index];
                        if producer.push(sample).is_err() {
                            break;
                        }
                        current_index += 1;

                        // フレームカウント（出力サンプルレートベース）
                        if current_index % channels == 0 {
                            frames_played += 1;

                            if frames_played % (output_sample_rate as u64) == 0 {
                                position_clone.store(
                                    frames_played / output_sample_rate as u64,
                                    Ordering::Relaxed
                                );
                            }
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(1));
            }
        });

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if paused_stream.load(Ordering::Relaxed) {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }

                    let vol = *volume_stream.lock();

                    for sample in data.iter_mut() {
                        *sample = consumer.pop().unwrap_or(0.0) * vol;
                    }
                },
                move |err| {
                    err!("Stream error: {}", err);
                },
                None,
            )
            .unwrap_or_else(|e| {
                err!("Failed to build output stream: {}", e);
                std::process::exit(1);
            });

        stream.play().unwrap_or_else(|e| {
            err!("Failed to play stream: {}", e);
            std::process::exit(1);
        });

        MusicPlay {
            stream,
            seeking,
            paused,
            volume,
            position,
            finished,
            format: self.format,
            decoder: self.decoder,
            track_id: self.track_id,
            sample_rate: self.sample_rate,
            output_sample_rate,
        }
    }
}

fn convert_samples(buffer: AudioBufferRef) -> Vec<f32> {
    let spec = *buffer.spec();
    let duration = buffer.frames();

    let mut sample_buf = SampleBuffer::<f32>::new(duration as u64, spec);
    sample_buf.copy_interleaved_ref(buffer);
    sample_buf.samples().to_vec()
}

impl MusicPlay {
    pub fn is_empty(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn pause(&mut self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&mut self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock()
    }

    pub fn set_volume(self, vol: f32) -> Self {
        *self.volume.lock() = vol.clamp(0.0, 1.0);
        self
    }

    pub fn set_volume_mut(&mut self, vol: f32) {
        *self.volume.lock() = vol.clamp(0.0, 1.0);
    }

    pub fn seek(&self, dur: Duration) -> Result<(), String> {
        let time_secs = dur.as_secs();

        self.seeking.store(true, Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(20));

        let mut format = self.format.lock().unwrap();
        let mut decoder = self.decoder.lock().unwrap();

        let seek_to = SeekTo::Time {
            time: Time::from(time_secs),
            track_id: Some(self.track_id),
        };

        format
            .seek(SeekMode::Accurate, seek_to)
            .map_err(|e| format!("Seek failed: {}", e))?;

        decoder.reset();

        self.position.store(time_secs, Ordering::Relaxed);
        self.seeking.store(false, Ordering::Relaxed);

        Ok(())
    }

    pub fn get_pos(&self) -> Duration {
        Duration::from_secs(self.position.load(Ordering::Relaxed))
    }
}