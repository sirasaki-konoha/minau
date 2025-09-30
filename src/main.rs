mod display_info;
mod info;
mod input;
mod macros;
mod play_music;
mod player;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    file: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    play_music::play_music(args.file).await;
}
