use crate::{err, info::info, player::play::MusicPlay};
use crossterm::{
    cursor::{Hide, Show},
    event::{Event, KeyCode, poll, read},
    execute,
    style::Stylize,
    terminal::{disable_raw_mode, enable_raw_mode},
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

pub async fn get_input(music_play: Arc<Mutex<MusicPlay>>, quit: Arc<Mutex<bool>>) {
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
                        return;
                    }
                    KeyCode::Char(' ') => {
                        let mut play = music_play.lock().unwrap();
                        if play.is_paused() {
                            play.resume();
                            info("Resumed");
                        } else {
                            play.pause();
                            info("Paused");
                        }
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let mut play = music_play.lock().unwrap();
                        let vol = play.get_volume();
                        if vol == 1.0 {
                            info(format!("{}", "Already at maximum volume!".red()));
                        } else {
                            play.set_volume_mut(vol + 0.1);
                            let formated = (vol * 100.0).round() as u16;
                            info(&format!("Volme set to {}", formated));
                        }
                    }

                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let mut play = music_play.lock().unwrap();
                        let vol = play.get_volume();
                        if vol < 0.0 {
                            info(format!("{}", "Already at minimum volume!".red()));
                        } else {
                            play.set_volume_mut(vol - 0.1);
                            let formated = (vol * 100.0).round() as u16;
                            info(&format!("Volme set to {}", formated));
                        }
                    }
                    KeyCode::Char(c) => {
                        info(format!("Unknown key: {}", c));
                    }
                    _ => {}
                }
            }
        }
    }
}
