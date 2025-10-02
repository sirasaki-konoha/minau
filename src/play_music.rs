use crate::display_image;
use crate::info::info;
use crate::input::{deinit, get_input};
use crate::player::metadata::MetaData;
use crate::player::player_structs::Player;
use crossterm::cursor::{self, MoveToPreviousLine};
use crossterm::execute;
use crossterm::terminal::Clear;
use humantime::format_duration;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::io::{stdout, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

pub async fn play_music<P: AsRef<Path>>(path: P, volume: f32, gui: bool) {
    let player = Player::new(&path);
    let metadata = player.metadata();

    let filename = path
        .as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let path_display = path.as_ref().display().to_string();

    let value = metadata.clone();
    let file_clone = filename.clone();
    let player_bind = player.clone();

    if !cfg!(target_os = "windows") {
        println!(
            "\x1b]2;{} - minau\x07",
            metadata.title().unwrap_or(path_display.clone())
        );
    }

    let play_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(really_play(player_bind, value, file_clone, volume));
    });

    if gui
        && let Some(pic) = metadata.picture() {
            // Wayland環境でminifb使うとウィンドウ閉じるときにメッセージ出るの何
            // XWayland強制するしかなくなっちゃったよ
            unsafe { env::set_var("WAYLAND_DISPLAY", "") };
            display_image::display(pic, path_display);
        }

    play_thread.join().unwrap();
    if !cfg!(target_os = "windows") {
        print!("\x1b]2;\x07");
        stdout().flush().unwrap();
    }
}

async fn really_play(player: Player, metadata: MetaData, filename: String, volume: f32) {
    if !cfg!(target_os = "windows") {
        execute!(
            stdout(),
            cursor::MoveToPreviousLine(1),
            Clear(crossterm::terminal::ClearType::FromCursorDown)
        ).unwrap();
    }
    
    let sample_rate_khz = player.sample_rate() as f32 / 1000.0;
    println!(
        "{}kHz/{}ch | {}",
        sample_rate_khz,
        player.channels(),
        format_duration(Duration::from_secs(metadata.duration().as_secs()))
    );
    crate::display_info::display_info(&filename, &metadata);

    let music_play = Arc::new(Mutex::new(player.play().set_volume(volume)));
    let music_play_clone = Arc::clone(&music_play);
    let key_state = Arc::new(Mutex::new(false));
    let key_state_clone = Arc::clone(&key_state);
    let key_thread = tokio::spawn(get_input(
        music_play_clone,
        key_state_clone,
        filename.to_string(),
        metadata.clone(),
    ));
    let duration = metadata.duration().as_secs();
    let mut current_secs = 0;
    let pb = ProgressBar::new(duration);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    pb.set_position(0);
    pb.set_message(format!(
        "{} / {}",
        format_duration(Duration::from_secs(current_secs)),
        format_duration(Duration::from_secs(duration))
    ));
    let mut tick_count = 0;

    loop {
        if key_thread.is_finished() {
            pb.finish_and_clear();
            deinit();
            drop(music_play);
            return;
        }

        if music_play.lock().unwrap().is_empty() {
            let mut key = key_state.lock().unwrap();
            *key = true;
            info(format!(
                "{} / {}",
                format_duration(Duration::from_secs(current_secs)),
                format_duration(Duration::from_secs(duration))
            ));
            pb.finish_and_clear();
            deinit();
            execute!(
                std::io::stdout(),
                MoveToPreviousLine(2),
                Clear(crossterm::terminal::ClearType::FromCursorDown),
            )
            .unwrap();
            return;
        }

        sleep(Duration::from_millis(100)).await;
        if !music_play.lock().unwrap().is_paused() {
            tick_count += 1;

            if tick_count >= 10 {
                tick_count = 0;
                current_secs += 1;
                pb.set_position(current_secs);
                pb.set_message(format!(
                    "{} / {}",
                    format_duration(Duration::from_secs(current_secs)),
                    format_duration(Duration::from_secs(duration))
                ));
            }
        }
    }
}
