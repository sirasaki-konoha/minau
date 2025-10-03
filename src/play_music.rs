use crate::input::{deinit, get_input};
use crate::player::metadata::MetaData;
use crate::player::player_structs::Player;
use crate::{display_image, display_info};
use crossterm::cursor::MoveToPreviousLine;
use crossterm::terminal::ClearType;
use crossterm::terminal::{Clear, SetTitle};
use crossterm::{execute, terminal};
use humantime::format_duration;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::io::{Write, stdout};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use unicode_width::UnicodeWidthStr;

const TICK_INTERVAL_MS: u64 = 100;
const TICKS_PER_SECOND: u32 = 10;

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
    let title = metadata.title().unwrap_or(path_display.clone());

    set_terminal_title(&title);

    let value = metadata.clone();
    let file_clone = filename.clone();
    let player_bind = player.clone();

    let bind = path_display.clone();
    let play_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(really_play(player_bind, value, file_clone, bind, volume));
    });

    if gui && let Some(pic) = metadata.picture() {
        if cfg!(target_os = "linux") && env::var("WAYLAND_DISPLAY").is_ok() {
            unsafe { env ::remove_var("WAYLAND_DISPLAY") };
        }

        display_image::display(pic, path_display);
    }

    play_thread.join().unwrap();

    reset_terminal_title();
}

fn set_terminal_title(title: &str) {
    execute!(stdout(), SetTitle(format!("{} - minau", title))).unwrap();
}

fn reset_terminal_title() {
    let cwd = env::current_dir().unwrap().display().to_string();
    execute!(stdout(), SetTitle(cwd)).unwrap();
    print!("\x1b]2;\x07");
    stdout().flush().unwrap();
}

async fn really_play(
    player: Player,
    metadata: MetaData,
    filename: String,
    path: String,
    volume: f32,
) {
    let sample_rate_khz = player.sample_rate() as f32 / 1000.0;
    let duration = metadata.duration();

    println!(
        "{}kHz/{}ch | {}",
        sample_rate_khz,
        player.channels(),
        format_duration(Duration::from_secs(duration.as_secs()))
    );
    crate::display_info::display_info(&filename, &metadata);

    let music_play = Arc::new(Mutex::new(player.play().set_volume(volume)));
    let key_state = Arc::new(Mutex::new(false));

    let key_thread = tokio::spawn(get_input(
        Arc::clone(&music_play),
        Arc::clone(&key_state),
        filename.clone(),
        path,
        metadata.clone(),
    ));

    let duration_secs = duration.as_secs();
    let pb = create_progress_bar(duration_secs);

    let mut current_secs = 0u64;
    let mut tick_count = 0u32;

    loop {
        if key_thread.is_finished() {
            cleanup_and_exit(&pb, metadata, &filename);
            return;
        }

        if music_play.lock().unwrap().is_empty() {
            *key_state.lock().unwrap() = true;
            cleanup_and_exit(&pb, metadata, &filename);
            return;
        }

        sleep(Duration::from_millis(TICK_INTERVAL_MS)).await;

        if !music_play.lock().unwrap().is_paused() {
            tick_count += 1;

            if tick_count >= TICKS_PER_SECOND {
                tick_count = 0;
                current_secs += 1;
                update_progress(&pb, current_secs, duration_secs);
            }
        }
    }
}

fn create_progress_bar(duration: u64) -> ProgressBar {
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
        format_duration(Duration::from_secs(0)),
        format_duration(Duration::from_secs(duration))
    ));
    pb
}

fn update_progress(pb: &ProgressBar, current: u64, total: u64) {
    pb.set_position(current);
    pb.set_message(format!(
        "{} / {}",
        format_duration(Duration::from_secs(current)),
        format_duration(Duration::from_secs(total))
    ));
}

fn cleanup_and_exit(pb: &ProgressBar, metadata: MetaData, path: &str) {
    let text_width = UnicodeWidthStr::width(display_info::string_info(path, &metadata).as_str());
    let (cols, _rows) = terminal::size().unwrap_or((80, 24));
    let lines_needed = ((text_width as u16 + cols - 1) / cols).max(1) - 1;

    for _ in 0..lines_needed {
        execute!(
            std::io::stdout(),
            MoveToPreviousLine(1),
            Clear(ClearType::FromCursorDown),
        )
        .unwrap();
    }

    pb.finish_and_clear();
    deinit();
}
