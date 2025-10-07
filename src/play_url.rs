#![allow(clippy::needless_range_loop)]
use crate::input::deinit;
use crate::{err, input};
use async_channel::Receiver;
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossterm::cursor::MoveToPreviousLine;
use crossterm::terminal::{self, Clear, ClearType, SetTitle};
use crossterm::{cursor, execute};
use http_body_util::BodyExt;
use hyper::Request;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use parking_lot::Mutex;
use ringbuf::HeapRb;
use std::collections::VecDeque;
use std::env;
use std::io::{self, Read, Result as IoResult, Write, stdout};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use unicode_width::UnicodeWidthStr;

macro_rules! eprintln {
    ($($msg: expr), *) => {
        if ::std::cfg!(debug_assertions) {
            ::std::eprintln!($($msg), *);
        }
    };
}

struct StreamReader {
    buffer: Arc<StdMutex<VecDeque<u8>>>,
    eof: Arc<AtomicBool>,
}

impl symphonia::core::io::MediaSource for StreamReader {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

impl std::io::Seek for StreamReader {
    fn seek(&mut self, _: std::io::SeekFrom) -> IoResult<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "StreamReader does not support seeking",
        ))
    }
}

impl StreamReader {
    fn new(rx: Receiver<Bytes>) -> Self {
        let buffer = Arc::new(StdMutex::new(VecDeque::new()));
        let eof = Arc::new(AtomicBool::new(false));

        let buffer_clone = Arc::clone(&buffer);
        let eof_clone = Arc::clone(&eof);

        std::thread::spawn(move || {
            loop {
                match rx.recv_blocking() {
                    Ok(chunk) => {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.extend(chunk.iter());
                    }
                    Err(_) => {
                        eof_clone.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        });

        Self { buffer, eof }
    }

    fn wait_for_data(&self, min_size: usize, timeout: Duration) -> bool {
        let start = std::time::Instant::now();

        loop {
            let size = {
                let buf = self.buffer.lock().unwrap();
                buf.len()
            };

            if size >= min_size || self.eof.load(Ordering::Relaxed) {
                return true;
            }

            if start.elapsed() >= timeout {
                eprintln!(
                    "[StreamReader] Timeout waiting for {} bytes (have: {})",
                    min_size, size
                );
                return false;
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        loop {
            {
                let mut buffer = self.buffer.lock().unwrap();
                if !buffer.is_empty() {
                    let to_copy = buffer.len().min(buf.len());
                    for i in 0..to_copy {
                        buf[i] = buffer.pop_front().unwrap();
                    }
                    return Ok(to_copy);
                }
            }

            if self.eof.load(Ordering::Relaxed) {
                return Ok(0);
            }

            if !self.wait_for_data(1, Duration::from_secs(5)) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout waiting for data",
                ));
            }
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

// シンプルな線形補間リサンプラー
fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32, channels: usize) -> Vec<f32> {
    if from_rate == to_rate {
        return input.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let input_frames = input.len() / channels;
    let output_frames = (input_frames as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_frames * channels);

    for out_frame in 0..output_frames {
        let src_pos = out_frame as f64 * ratio;
        let src_frame = src_pos.floor() as usize;
        let frac = src_pos - src_frame as f64;

        if src_frame + 1 >= input_frames {
            // 最後のフレームをそのまま使用
            for ch in 0..channels {
                let idx = src_frame * channels + ch;
                if idx < input.len() {
                    output.push(input[idx]);
                } else {
                    output.push(0.0);
                }
            }
        } else {
            // 線形補間
            for ch in 0..channels {
                let idx1 = src_frame * channels + ch;
                let idx2 = (src_frame + 1) * channels + ch;
                let sample1 = input[idx1];
                let sample2 = input[idx2];
                let interpolated = sample1 + (sample2 - sample1) * frac as f32;
                output.push(interpolated);
            }
        }
    }

    output
}

pub struct UrlPlayer {
    _stream: Stream,
    paused: Arc<AtomicBool>,
    volume: Arc<Mutex<f32>>,
    finished: Arc<AtomicBool>,
    sample_rate: u32,
    channels: u32,
    downloaded_bytes: Arc<Mutex<u64>>,
    total_bytes: Arc<Mutex<Option<u64>>>,
}

unsafe impl Send for UrlPlayer {}

impl UrlPlayer {
    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock() = volume.clamp(0.0, 1.0);
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock()
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn get_downloaded_bytes(&self) -> u64 {
        *self.downloaded_bytes.lock()
    }

    pub fn get_downloaded_mb(&self) -> f64 {
        self.get_downloaded_bytes() as f64 / 1024.0 / 1024.0
    }

    pub fn get_total_bytes(&self) -> Option<u64> {
        *self.total_bytes.lock()
    }

    pub fn get_total_mb(&self) -> Option<f64> {
        self.get_total_bytes().map(|b| b as f64 / 1024.0 / 1024.0)
    }

    pub fn get_download_progress(&self) -> Option<f32> {
        let downloaded = self.get_downloaded_bytes();
        self.get_total_bytes()
            .map(|total| (downloaded as f32 / total as f32) * 100.0)
    }
}

pub async fn setup_url_player(
    url: &str,
    volume: f32,
) -> Result<UrlPlayer, Box<dyn std::error::Error>> {
    let https = HttpsConnector::new();
    let client: Client<_, String> = Client::builder(TokioExecutor::new()).build(https);

    let mut current_url = url.to_string();
    let mut redirect_count = 0;
    let max_redirects = 10;

    let response = loop {
        let uri = current_url.parse::<hyper::Uri>()?;
        let req = Request::builder()
            .uri(uri)
            .header("User-Agent", format!("minau/{}", env!("CARGO_PKG_VERSION")))
            .body(String::new())?;

        let resp = client.request(req).await?;
        let status = resp.status();

        if status.is_redirection() {
            if redirect_count >= max_redirects {
                return Err("Too many redirects".into());
            }

            if let Some(location) = resp.headers().get("location") {
                current_url = location.to_str()?.to_string();

                if !current_url.starts_with("http") {
                    let base_uri = url.parse::<hyper::Uri>()?;
                    let scheme = base_uri.scheme_str().unwrap_or("https");
                    let authority = base_uri.authority().ok_or("No authority in URL")?;
                    current_url = format!("{}://{}{}", scheme, authority, current_url);
                }

                redirect_count += 1;
                continue;
            } else {
                return Err("Redirect without Location header".into());
            }
        }

        if !status.is_success() {
            return Err(format!("HTTP Error: {}", status).into());
        }

        break resp;
    };

    let total_bytes = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let (tx, rx) = async_channel::bounded::<Bytes>(50);

    let downloaded_bytes = Arc::new(Mutex::new(0u64));
    let downloaded_clone = Arc::clone(&downloaded_bytes);

    std::thread::spawn(move || {
        smol::block_on(async {
            let mut body = response.into_body();

            while let Some(result) = body.frame().await {
                match result {
                    Ok(frame) => {
                        if let Some(chunk) = frame.data_ref() {
                            let chunk_size = chunk.len() as u64;
                            *downloaded_clone.lock() += chunk_size;

                            match tx.send(chunk.clone()).await {
                                Ok(_) => {}
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        err!("Stream Error: {}", e);
                        break;
                    }
                }
            }
            drop(tx);
        })
    });

    let downloaded_bytes_clone = Arc::clone(&downloaded_bytes);
    let player = std::thread::spawn(
        move || -> Result<UrlPlayer, Box<dyn std::error::Error + Send + Sync>> {
            let reader = StreamReader::new(rx);

            if !reader.wait_for_data(64 * 1024, Duration::from_secs(10)) {
                return Err("Failed to buffer initial data".into());
            }

            let buffered_size = 256 * 1024;
            let mut hint = Hint::new();

            let detect_buf: Vec<u8> = {
                let buffer = reader.buffer.lock().unwrap();
                let detect_size = buffer.len().min(2000);
                buffer.iter().take(detect_size).copied().collect()
            };

            if let Some(kind) = infer::get(&detect_buf) {
                match kind.mime_type() {
                    "audio/mpeg" => hint.with_extension("mp3"),
                    "audio/flac" => hint.with_extension("flac"),
                    "audio/ogg" => hint.with_extension("ogg"),
                    "audio/wav" => hint.with_extension("wav"),
                    "audio/aac" => hint.with_extension("aac"),
                    "audio/mp4" => hint.with_extension("m4a"),
                    _ => {
                        err!("Stream is not supported mime type!");
                        exit(1);
                    }
                };
            }

            let mss = MediaSourceStream::new(
                Box::new(reader),
                symphonia::core::io::MediaSourceStreamOptions {
                    buffer_len: buffered_size,
                },
            );

            let meta_opts: MetadataOptions = Default::default();
            let fmt_opts: FormatOptions = Default::default();

            let probed =
                symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

            let format = probed.format;

            let track = format
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                .ok_or("Track not found")?;

            let track_id = track.id;
            let codec_params = &track.codec_params;

            let sample_rate = codec_params
                .sample_rate
                .ok_or("Samplerate is not available")?;
            let channels = codec_params.channels.ok_or("Channels is not available")?;
            let channels_count = channels.count() as u16;

            let dec_opts: DecoderOptions = Default::default();
            let decoder = symphonia::default::get_codecs().make(codec_params, &dec_opts)?;

            // cpal setup
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .ok_or("No output device available")?;
            let device_config = device.default_output_config().unwrap();

            let output_sample_rate = device_config.sample_rate().0;

            eprintln!(
                "[Audio] Source: {}Hz, Device: {}Hz, Channels: {}",
                sample_rate, output_sample_rate, channels_count
            );

            let config = StreamConfig {
                channels: device_config.channels(),
                sample_rate: device_config.sample_rate(),
                buffer_size: cpal::BufferSize::Default,
            };

            let paused = Arc::new(AtomicBool::new(false));
            let volume_arc = Arc::new(Mutex::new(volume));
            let finished = Arc::new(AtomicBool::new(false));

            let format = Arc::new(StdMutex::new(format));
            let decoder = Arc::new(StdMutex::new(decoder));

            let (mut producer, mut consumer) =
                HeapRb::<f32>::new(output_sample_rate as usize * 2).split();

            let format_clone = Arc::clone(&format);
            let decoder_clone = Arc::clone(&decoder);
            let finished_clone = Arc::clone(&finished);

            // Decoder thread
            std::thread::spawn(move || {
                let mut current_samples = Vec::new();
                let mut current_index = 0;

                loop {
                    if finished_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    while producer.free_len() > 4096 {
                        if current_index >= current_samples.len() {
                            let mut format = format_clone.lock().unwrap();
                            let mut decoder = decoder_clone.lock().unwrap();

                            let packet = match format.next_packet() {
                                Ok(packet) => packet,
                                Err(Error::IoError(e))
                                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                                {
                                    finished_clone.store(true, Ordering::Relaxed);
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("[Decoder] Error reading packet: {:?}", e);
                                    finished_clone.store(true, Ordering::Relaxed);
                                    break;
                                }
                            };

                            if packet.track_id() != track_id {
                                continue;
                            }

                            match decoder.decode(&packet) {
                                Ok(decoded) => {
                                    let raw_samples = convert_samples(decoded);

                                    // サンプルレート変換
                                    current_samples = if sample_rate != output_sample_rate {
                                        resample_linear(
                                            &raw_samples,
                                            sample_rate,
                                            output_sample_rate,
                                            channels_count as usize,
                                        )
                                    } else {
                                        raw_samples
                                    };

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
                        }
                    }

                    std::thread::sleep(Duration::from_millis(5));
                }
            });

            let paused_stream = Arc::clone(&paused);
            let volume_stream = Arc::clone(&volume_arc);

            let stream = device.build_output_stream(
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
            )?;

            stream.play()?;

            Ok(UrlPlayer {
                _stream: stream,
                paused,
                volume: volume_arc,
                finished,
                sample_rate,
                channels: channels_count as u32,
                downloaded_bytes: downloaded_bytes_clone,
                total_bytes: Arc::new(Mutex::new(total_bytes)),
            })
        },
    )
    .join()
    .unwrap()
    .unwrap();

    Ok(player)
}

pub async fn play_url(url: &str, volume: f32, title_override: Option<String>) {
    let p = match setup_url_player(url, volume).await {
        Ok(player) => player,
        Err(e) => {
            err!("Failed to setup url player: {}", e);
            return;
        }
    };

    let title = title_override.unwrap_or_else(|| url.to_string());
    println!(
        "{}kHz/{}ch | Unknown",
        p.sample_rate() as f32 / 1000.0,
        p.channels()
    );
    let player = Arc::new(Mutex::new(p));
    let key_state = Arc::new(Mutex::new(false));

    println!("{}", title);
    let thread = smol::spawn(input::get_input_url_mode(
        Arc::clone(&player),
        title.clone(),
        key_state.clone(),
    ));

    set_terminal_title(&title);

    let mut first = false;

    loop {
        smol::Timer::after(Duration::from_millis(200)).await;

        let locked = Arc::clone(&player);
        let locked = locked.lock();

        if !first {
            execute!(
                stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine)
            )
            .unwrap();
        } else {
            first = !first;
        }

        if let Some(progress) = locked.get_download_progress() {
            print!(
                "{:.1}% ({:.2} / {:.2} MB)",
                progress,
                locked.get_downloaded_mb(),
                locked.get_total_mb().unwrap(),
            );
        } else {
            print!("({:.2} MB)", locked.get_downloaded_mb());
        }
        io::stdout().flush().unwrap();

        if thread.is_finished() {
            cleanup_and_exit(&title);
            break;
        }
        if locked.is_empty() {
            *key_state.lock() = true;
            cleanup_and_exit(&title);
            break;
        }
    }
}

fn set_terminal_title(title: &str) {
    execute!(stdout(), SetTitle(title.to_string())).unwrap();
}

fn reset_terminal_title() {
    let cwd = env::current_dir().unwrap().display().to_string();
    execute!(stdout(), SetTitle(cwd)).unwrap();
    print!("\x1b]2;\x07");
    stdout().flush().unwrap();
}

fn cleanup_and_exit(title: &str) {
    let text_width = UnicodeWidthStr::width(title);
    let (cols, _rows) = terminal::size().unwrap_or((80, 24));
    let lines_needed = (text_width as u16).div_ceil(cols).max(1) - 1;

    execute!(
        std::io::stdout(),
        MoveToPreviousLine(2),
        Clear(crossterm::terminal::ClearType::FromCursorDown),
    )
    .unwrap();

    for _ in 0..lines_needed {
        execute!(
            std::io::stdout(),
            MoveToPreviousLine(1),
            Clear(ClearType::FromCursorDown),
        )
        .unwrap();
    }

    reset_terminal_title();
    deinit();
}
