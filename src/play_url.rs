use crate::input::deinit;
use crate::{err, input};
use bytes::Bytes;
use crossterm::cursor::MoveToPreviousLine;
use crossterm::terminal::{self, Clear, ClearType, SetTitle};
use crossterm::{cursor, execute};
use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};
use unicode_width::UnicodeWidthStr;
use std::io::{self, stdout, Read, Result as IoResult, Write};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::{env, thread};
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::mpsc;

// 非同期ストリームを同期的なReadに変換するアダプター
struct StreamReader {
    rx: mpsc::Receiver<Bytes>,
    current: Option<Bytes>,
    offset: usize,
}

// MediaSource trait implementation for StreamReader
impl symphonia::core::io::MediaSource for StreamReader {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

// Implement Seek for StreamReader (not supported)
impl std::io::Seek for StreamReader {
    fn seek(&mut self, _: std::io::SeekFrom) -> IoResult<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "StreamReader does not support seeking",
        ))
    }
}

impl StreamReader {
    fn new(rx: mpsc::Receiver<Bytes>) -> Self {
        Self {
            rx,
            current: None,
            offset: 0,
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        loop {
            if let Some(chunk) = &self.current
                && self.offset < chunk.len() {
                    let remaining = chunk.len() - self.offset;
                    let to_copy = remaining.min(buf.len());

                    buf[..to_copy].copy_from_slice(&chunk[self.offset..self.offset + to_copy]);

                    self.offset += to_copy;
                    return Ok(to_copy);
                }

            match self.rx.blocking_recv() {
                Some(chunk) => {
                    self.current = Some(chunk);
                    self.offset = 0;
                }
                None => return Ok(0),
            }
        }
    }
}

// Symphoniaでデコードしたサンプルを保持するソース
pub struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    current_samples: Vec<f32>,
    current_index: usize,
    finished: Arc<Mutex<bool>>,
}

impl SymphoniaSource {
    fn new(
        format: Box<dyn FormatReader>,
        decoder: Box<dyn symphonia::core::codecs::Decoder>,
        track_id: u32,
        sample_rate: u32,
        channels: u16,
    ) -> Self {
        Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            current_samples: Vec::new(),
            current_index: 0,
            finished: Arc::new(Mutex::new(false)),
        }
    }

    fn decode_next_packet(&mut self) -> bool {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    *self.finished.lock().unwrap() = true;
                    return false;
                }
                Err(_) => {
                    *self.finished.lock().unwrap() = true;
                    return false;
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    self.current_samples = convert_samples(decoded);
                    self.current_index = 0;
                    return true;
                }
                Err(_) => continue,
            }
        }
    }

    #[allow(unused)]
    pub fn is_finished(&self) -> bool {
        *self.finished.lock().unwrap()
    }
}

fn convert_samples(buffer: AudioBufferRef) -> Vec<f32> {
    let spec = *buffer.spec();
    let duration = buffer.frames();

    let mut sample_buf = SampleBuffer::<f32>::new(duration as u64, spec);
    sample_buf.copy_interleaved_ref(buffer);
    sample_buf.samples().to_vec()
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_index < self.current_samples.len() {
                let sample = self.current_samples[self.current_index];
                self.current_index += 1;
                return Some(sample);
            }

            if !self.decode_next_packet() {
                return None;
            }
        }
    }
}

impl Source for SymphoniaSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct UrlPlayer {
    sink: Sink,
    _stream: OutputStream,
    _source: Arc<Mutex<Option<Arc<Mutex<SymphoniaSource>>>>>,
    sample_rate: u32,
    channel: u32,
    downloaded_bytes: Arc<Mutex<u64>>,
    total_bytes: Arc<Mutex<Option<u64>>>,
}

impl UrlPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut stream = OutputStreamBuilder::open_default_stream().unwrap_or_else(|e| {
            err!("Failed to open stream: {}", e);
            exit(1);
        });
        let sink = Sink::connect_new(stream.mixer());
        stream.log_on_drop(false);

        Ok(Self {
            sink,
            _stream: stream,
            sample_rate: 0,
            channel: 0,
            _source: Arc::new(Mutex::new(None)),
            downloaded_bytes: Arc::new(Mutex::new(0)),
            total_bytes: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn get_volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    pub fn channels(&self) -> u32 {
        self.channel
    }

    pub fn get_downloaded_bytes(&self) -> u64 {
        *self.downloaded_bytes.lock().unwrap()
    }

    pub fn get_downloaded_mb(&self) -> f64 {
        self.get_downloaded_bytes() as f64 / 1024.0 / 1024.0
    }

    pub fn get_total_bytes(&self) -> Option<u64> {
        *self.total_bytes.lock().unwrap()
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
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP Error: {}", response.status()).into());
    }

    // Content-Lengthヘッダーから総サイズを取得
    let total_bytes = response.content_length();

    let (tx, rx) = mpsc::channel::<Bytes>(32);

    let downloaded_bytes = Arc::new(Mutex::new(0u64));
    let downloaded_clone = Arc::clone(&downloaded_bytes);

    // 非同期タスクでストリームを受信
    tokio::spawn(async move {
        let mut response = response;

        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    let chunk_size = chunk.len() as u64;
                    *downloaded_clone.lock().unwrap() += chunk_size;

                    if tx.send(chunk).await.is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    err!("Stream Error: {}", e);
                    break;
                }
            }
        }
    });

    // 別スレッドでsymphonia + rodioのセットアップ
    let url_owned = url.to_string();
    let downloaded_bytes_clone = Arc::clone(&downloaded_bytes);
    let player = std::thread::spawn(
        move || -> Result<UrlPlayer, Box<dyn std::error::Error + Send + Sync>> {
            let reader = StreamReader::new(rx);
            let mss = MediaSourceStream::new(Box::new(reader), Default::default());

            let mut hint = Hint::new();
            // URLから拡張子を推測
            if url_owned.contains(".mp3") {
                hint.with_extension("mp3");
            } else if url_owned.contains(".flac") {
                hint.with_extension("flac");
            } else if url_owned.contains(".ogg") {
                hint.with_extension("ogg");
            }

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

            let dec_opts: DecoderOptions = Default::default();
            let decoder = symphonia::default::get_codecs().make(codec_params, &dec_opts)?;

            let source = SymphoniaSource::new(
                format,
                decoder,
                track_id,
                sample_rate,
                channels.count() as u16,
            );

            let mut player = UrlPlayer::new().unwrap();
            player.set_volume(volume);
            player.downloaded_bytes = downloaded_bytes_clone;
            *player.total_bytes.lock().unwrap() = total_bytes;
            player.sink.append(source);
            player.channel = channels.count() as u32;
            player.sample_rate = sample_rate;

            Ok(player)
        },
    )
    .join()
    .unwrap()
    .unwrap();

    Ok(player)
}

pub async fn play_url(url: &str, volume: f32) {
    let p = setup_url_player(url, volume).await.unwrap();
    println!("{}kHz/{}ch | Unknown",
        p.sample_rate() as f32 / 1000.0,
        p.channels()
    );
    let player = Arc::new(Mutex::new(p));
    let key_state = Arc::new(Mutex::new(false));

    println!("{}", url);
    let thread = tokio::spawn(input::get_input_url_mode(
        Arc::clone(&player),
        url.to_string(),
        key_state.clone()
    ));
    
    set_terminal_title(url);
    
    let mut first = false;

    loop {
        thread::sleep(Duration::from_millis(200));
        
        let locked = Arc::clone(&player);
        let locked = locked.lock().unwrap();
        
        if !first {
            execute!(
                stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine)
            ).unwrap();
        } else {
            first = !first;
        }

        if let Some(progress) = locked.get_download_progress() {

            print!("{:.1}% ({:.2} / {:.2} MB)",
                progress,
                locked.get_downloaded_mb(),
                locked.get_total_mb().unwrap(),
            );
        } else {
            print!("({:.2} MB)", locked.get_downloaded_mb());
        }
        io::stdout().flush().unwrap();

        // pb.set_length(locked.get_downloaded_bytes());
        // if let Some(progress) = locked.get_download_progress() {
        //     pb.set_message(format!(
        //         "{:.1}% ({:.2}/{:.2} MB)",
        //         progress,
        //         locked.get_downloaded_mb(),
        //         locked.get_total_mb().unwrap()
        //     ));
        //     dbg!("locked");
        // }

        if thread.is_finished() {
            break;
        }
        if locked.is_empty() {
            *key_state.lock().unwrap() = true;
            cleanup_and_exit(url);
            break;
        }
    }
}


fn set_terminal_title(url: &str) {
    execute!(stdout(), SetTitle(format!("{}", url))).unwrap();
}

fn reset_terminal_title() {
    let cwd = env::current_dir().unwrap().display().to_string();
    execute!(stdout(), SetTitle(cwd)).unwrap();
    print!("\x1b]2;\x07");
    stdout().flush().unwrap();
}

fn cleanup_and_exit(url: &str){
    let text_width = UnicodeWidthStr::width(url);
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
