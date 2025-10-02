use crate::{
    err,
    info::{info, info_with_restore},
    player::{metadata::MetaData, play::MusicPlay},
};
use crossterm::{
    cursor::{Hide, MoveToPreviousLine, Show},
    event::{Event, KeyCode, poll, read},
    execute,
    style::Stylize,
    terminal::{Clear, disable_raw_mode, enable_raw_mode},
};
use std::{
    io::stdout,
    process::exit,
    sync::{Arc, Mutex},
    time::Duration,
};

pub fn init_terminal() {
    enable_raw_mode().unwrap_or_else(|e| {
        err!("Failed to initialize terminal: {}", e);
        exit(1);
    });
    execute!(stdout(), Hide).unwrap_or_else(|e| {
        err!("Failed to initialize terminal: {}", e);
        exit(1);
    });
}

pub fn deinit() {
    disable_raw_mode().unwrap_or_else(|e| {
        err!("Failed to disable raw mode: {}", e);
        err!("Please execute 'reset' command");
        exit(1);
    });

    execute!(stdout(), Show).unwrap_or_else(|e| {
        err!("Failed to initialize terminal: {}", e);
        exit(1);
    });
}

const VOLUME_STEP: f32 = 0.05;
const POLL_INTERVAL_MS: u64 = 100;

pub async fn get_input(
    music_play: Arc<Mutex<MusicPlay>>,
    quit: Arc<Mutex<bool>>,
    filename: String,
    metadata: MetaData,
) {
    init_terminal();
    loop {
        if *quit.lock().unwrap() {
            return;
        }

        if !poll(Duration::from_millis(POLL_INTERVAL_MS)).unwrap() {
            continue;
        }

        let event = read().unwrap_or_else(|e| {
            err!("Failed to read key: {}", e);
            exit(1);
        });

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => {
                    info("Exitting...");
                    deinit();
                    println!();
                    exit(0);
                }
                KeyCode::Char('>') | KeyCode::Char('l') => {
                    info("Next track");
                    execute!(
                        std::io::stdout(),
                        MoveToPreviousLine(2),
                        Clear(crossterm::terminal::ClearType::FromCursorDown),
                    )
                    .unwrap();
                    return;
                }
                KeyCode::Char(' ') => {
                    let mut play = music_play.lock().unwrap();
                    let msg = if play.is_paused() {
                        play.resume();
                        "Resumed"
                    } else {
                        play.pause();
                        "Paused"
                    };
                    info_with_restore(msg, filename.clone(), metadata.clone());
                }
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    adjust_volume(&music_play, VOLUME_STEP, &filename, &metadata);
                }
                KeyCode::Char('-') | KeyCode::Char('_') => {
                    adjust_volume(&music_play, -VOLUME_STEP, &filename, &metadata);
                }
                KeyCode::Char(c) => {
                    info_with_restore(
                        format!("Unknown key: {}", c.red()),
                        filename.clone(),
                        metadata.clone(),
                    );
                }
                _ => {}
            }
        }
    }
}

fn adjust_volume(
    music_play: &Arc<Mutex<MusicPlay>>,
    delta: f32,
    filename: &str,
    metadata: &MetaData,
) {
    let mut play = music_play.lock().unwrap();
    let vol = play.get_volume();
    let new_vol = (vol + delta).clamp(0.0, 1.0);

    if new_vol == vol {
        let msg = if delta > 0.0 {
            "Already at maximum volume!".red().to_string()
        } else {
            "Already at minimum volume!".red().to_string()
        };
        info_with_restore(msg, filename.to_string(), metadata.clone());
    } else {
        play.set_volume_mut(new_vol);
        let percent = (new_vol * 100.0).round() as u16;
        info_with_restore(
            format!("Volume set to {}", percent.to_string().cyan()),
            filename.to_string(),
            metadata.clone(),
        );
    }
}
