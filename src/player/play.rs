use crate::err;
use crate::player::player::Player;

impl Player {
    /// this function is call sleep() until end
    pub fn play(self) {
        let mut stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().unwrap_or_else(|e| {
                err!("Failed to open stream: {}", e);
                std::process::exit(1);
            });
        stream_handle.log_on_drop(false);

        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        sink.append(self.decoder);

        sink.sleep_until_end();
    }
}
