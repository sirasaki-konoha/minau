mod display_image;
mod m3u;
mod display_info;
mod info;
mod input;
mod macros;
mod play_music;
mod player;
use std::{path::Path, process::exit};

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

const DEFAULT_VOLUME: u16 = 100;
const MIN_VOLUME: u16 = 1;
const MAX_VOLUME: u16 = 100;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let volume = args
        .volume
        .map(|vol| {
            if (MIN_VOLUME..=MAX_VOLUME).contains(&vol) {
                Ok(vol as f32 / 100.0)
            } else {
                Err(vol)
            }
        })
        .unwrap_or(Ok(DEFAULT_VOLUME as f32 / 100.0))
        .unwrap_or_else(|vol| {
            err!("{} is not available volume", vol);
            exit(1);
        });

    if args.files.is_empty() {
        err!("Music file is not specified!");
        exit(1);
    }

    for path in args.files {
        let path_extens: &Path = path.as_ref();
        if let Some(ext) = path_extens.extension()
            && ext == "m3u" {
                m3u::play_m3u(&path, volume, args.gui).await;
            }

        play_music::play_music(&path, volume, args.gui).await;
    }
}
