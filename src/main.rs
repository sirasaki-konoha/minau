mod display_image;
mod display_info;
mod info;
mod input;
mod macros;
mod play_music;
mod player;
use std::process::exit;

use clap::Parser;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    /// Files to play (multiple selections allowed)
    files: Vec<String>,
    /// Specify the default playback volume (minimum: 1, maximum: 100)
    #[arg(short, long)]
    volume: Option<u16>,
    /// Display album art in a GUI
    #[arg(short, long)]
    gui: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let volume_percent: u16 = 100;
    let mut volume: f32 = volume_percent as f32 / 100.0;

    if let Some(vol) = args.volume {
        if !(1..=100).contains(&vol) {
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
        play_music::play_music(path, volume, args.gui).await;
    }
}
