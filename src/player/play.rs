use std::time::Duration;

use crate::err;
use crate::player::player_structs::Player;
use rodio::{OutputStream, Sink};

pub struct MusicPlay {
    sink: Sink,
    _stream_handle: OutputStream,
}

impl Player {
    pub fn play(self) -> MusicPlay {
        let mut stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().unwrap_or_else(|e| {
                err!("Failed to open stream: {}", e);
                std::process::exit(1);
            });
        stream_handle.log_on_drop(false);

        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        sink.append(self.decoder);

        MusicPlay {
            sink,
            _stream_handle: stream_handle,
        }
    }
}

impl MusicPlay {
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        self.sink.play();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn get_volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(self, vol: f32) -> Self {
        self.sink.set_volume(vol);
        self
    }

    pub fn set_volume_mut(&mut self, vol: f32) {
        self.sink.set_volume(vol);
    }

    #[allow(unused)]
    pub fn seek(&self, dur: Duration) -> Result<(), rodio::source::SeekError> {
        self.sink.try_seek(dur)
    }

    pub fn get_pos(&self) -> std::time::Duration {
        self.sink.get_pos()
    }
}
