use crate::input::{deinit, get_input, init_terminal};
use crate::{
    err,
    player::{
        info,
        metadata::{self, MetaData},
        player::Player,
    },
};
use crossterm::event::poll;
use crossterm::style::Stylize;
use crossterm::{cursor, execute, terminal::Clear};
use humantime::format_duration;
use std::io::{self, Write};
use std::time::Duration;
use std::{path::Path, process::exit};
use tokio::time::sleep;

pub async fn play_music<P: AsRef<Path>>(path: P) {
    let player = Player::new(&path);
    let metadata = player.metadata();
    let filename = path.as_ref().file_name().unwrap().to_str().unwrap();
    let path = path.as_ref().to_str().unwrap();

    println!(
        "{}: {}K/{}ch",
        path,
        player.sample_rate(),
        player.channels()
    );
    crate::display_info::display_info(filename, &metadata);

    let playing = tokio::spawn(async { player.play() });
    let key_thread = tokio::spawn(async { get_input() });
    let duration = Duration::from_secs(metadata.duration().as_secs());
    let mut current_secs = 0;

    loop {
        if playing.is_finished() {
            key_thread.abort();
            println!();
            deinit();
            return;
        }
        display_status(duration, Duration::from_secs(current_secs));
        sleep(Duration::from_secs(1)).await;
        current_secs += 1;
    }
}

fn display_status(full: Duration, curr: Duration) {
    let mut stdout = io::stdout();

    execute!(
        stdout,
        cursor::MoveToPreviousLine(1),
        cursor::MoveToColumn(0),
        Clear(crossterm::terminal::ClearType::CurrentLine),
    )
    .unwrap_or_else(|e| {
        err!("Failed to display status info: {}", e);
        exit(1);
    });

    let total = format_duration(full);
    let curr = format_duration(curr);
    println!(
        "{}: {}",
        total.to_string().underline_red(),
        curr.to_string().underline_green()
    );
    stdout.flush().unwrap();
}
