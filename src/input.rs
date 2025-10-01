use std::{io::stdout, process::exit, time::Duration};

use crossterm::{
    cursor::{Hide, Show},
    event::{Event, KeyCode, poll, read},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::{err, info::info};

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

pub async fn get_input() {
    init_terminal();
    loop {
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
                    KeyCode::Char(c) => {
                        info(format!("Unknown key: {}", c));
                    }
                    _ => {}
                }
            }
        }
    }
}
