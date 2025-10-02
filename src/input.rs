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

        if poll(Duration::from_millis(100)).unwrap() {
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
                        if play.is_paused() {
                            play.resume();
                            info_with_restore("Resumed", filename.clone(), metadata.clone());
                        } else {
                            play.pause();
                            info_with_restore("Paused", filename.clone(), metadata.clone());
                        }
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let mut play = music_play.lock().unwrap();
                        let vol = play.get_volume();
                        if vol >= 1.0 {
                            info_with_restore(
                                &format!("{}", "Already at maximum volume!".red()),
                                filename.clone(),
                                metadata.clone(),
                            );
                            play.set_volume_mut(1.0);
                        } else {
                            let new_vol = vol + 0.05;
                            play.set_volume_mut(new_vol);
                            let formated = (new_vol * 100.0).round() as u16;
                            info_with_restore(
                                &format!("Volume set to {}", formated.to_string().cyan()),
                                filename.clone(),
                                metadata.clone(),
                            );
                        }
                    }

                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let mut play = music_play.lock().unwrap();
                        let vol = play.get_volume();
                        if vol <= 0.0 {
                            info_with_restore(
                                &format!("{}", "Already at minimum volume!".red()),
                                filename.clone(),
                                metadata.clone(),
                            );
                        } else {
                            let new_vol = vol - 0.05;
                            play.set_volume_mut(new_vol);
                            let formated = (new_vol * 100.0).round() as u16;
                            info_with_restore(
                                &format!("Volume set to {}", formated.to_string().cyan()),
                                filename.clone(),
                                metadata.clone(),
                            );
                        }
                    }
                    KeyCode::Char(c) => {
                        info_with_restore(
                            &format!("Unknown key: {}", c.red()),
                            filename.clone(),
                            metadata.clone(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
