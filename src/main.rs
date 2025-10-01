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
    files: Vec<String>,
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

    for path in args.files {
        play_music::play_music(path, volume).await;
    }
}
