//! **minau**: A command-line music player built with Rust using the *rodio* library.
//!
//! **Quick usage:**
//!
//! ```
//! minau <path/to/music/files> [--volume <volume>]
//! ```
//!
//! **Details:**
//! minau is a lightweight command-line music player that uses the Rust *rodio* library. It is highly efficient and works even in resource-constrained environments.
//!
//! **Command-line arguments:**
//!
//! * **files: `<Vec<String>>`** — Accepts music files to play (multiple files can be specified).
//! * **volume** — Adjusts the playback volume. The maximum is 100 and the minimum is 1.

mod display_info;
mod info;
mod input;
mod macros;
mod play_music;
mod player;
use std::process::exit;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Files to play (multiple selections allowed)
    files: Vec<String>,
    /// Specify the default playback volume (minimum: 1, maximum: 100)
    #[arg(short, long)]
    volume: Option<u16>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let volume_percent: u16 = 100;
    let mut volume: f32 = volume_percent as f32 / 100.0;

    if let Some(vol) = args.volume {
        if vol > 100 || vol < 1 {
            err!("{} is not available volume", vol);
            exit(1);
        }
        volume = vol as f32 / 100.0;
    }

    if args.files.is_empty() {
        err!("Music file is not specified!");
        exit(1);
    }

    for path in args.files {
        play_music::play_music(path, volume).await;
    }
}
