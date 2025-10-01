use crate::input::{deinit, get_input};
use crate::player::player::Player;
use humantime::format_duration;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::{path::Path, process::exit};
use tokio::time::sleep;

pub async fn play_music<P: AsRef<Path>>(path: P) {
    let player = Player::new(&path);
    let metadata = player.metadata();
    let filename = path.as_ref().file_name().unwrap().to_str().unwrap();

    println!(
        "{}K/{}ch",
        player.sample_rate(),
        player.channels()
    );
    crate::display_info::display_info(filename, &metadata);
    println!("Welcome to minau!");
    let playing = tokio::spawn(async { player.play() });
    let _ = tokio::spawn(get_input());
    let duration = metadata.duration().as_secs();
    let mut current_secs = 0;
    let pb = ProgressBar::new(duration);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{bar:40.cyan/blue}] ({msg})").unwrap()
        .progress_chars("#>-"));

    pb.set_position(0);
    pb.set_message(format!("{}/{}", format_duration(Duration::from_secs(duration)), format_duration(Duration::from_secs(current_secs))));
    loop {
        if playing.is_finished() {
            pb.finish_and_clear();
            deinit();
            exit(0);
        }
        sleep(Duration::from_secs(1)).await;
        current_secs += 1;
        pb.set_position(current_secs);
        pb.set_message(format!("{}/{}", format_duration(Duration::from_secs(duration)), format_duration(Duration::from_secs(current_secs))));
    }
}
