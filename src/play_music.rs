use crate::info::info;
use crate::input::{deinit, get_input};
use crate::player::player::Player;
use humantime::format_duration;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

pub async fn play_music<P: AsRef<Path>>(path: P, volume: f32) {
    let player = Player::new(&path);
    let metadata = player.metadata();
    let filename = path.as_ref().file_name().unwrap().to_str().unwrap();

    println!("{}K/{}ch", player.sample_rate(), player.channels());
    crate::display_info::display_info(filename, &metadata);
    println!("Welcome to minau!");

    let music_play = Arc::new(Mutex::new(player.play().set_volume(volume)));
    let music_play_clone = Arc::clone(&music_play);
    let key_state = Arc::new(Mutex::new(false));
    let key_state_clone = Arc::clone(&key_state);
    let key_thread = tokio::spawn(get_input(music_play_clone, key_state_clone));
    let duration = metadata.duration().as_secs();
    let mut current_secs = 0;
    let pb = ProgressBar::new(duration);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    pb.set_position(0);
    pb.set_message(format!(
        "{}/{}",
        format_duration(Duration::from_secs(duration)),
        format_duration(Duration::from_secs(current_secs))
    ));
    let mut tick_count = 0;

    loop {
        // キー入力があったら終了
        if key_thread.is_finished() {
            pb.finish_and_clear();
            deinit();
            drop(music_play);
            return;
        }

        // 再生が終了したかチェック
        if music_play.lock().unwrap().is_empty() {
            let mut key = key_state.lock().unwrap();
            *key = true;
            info(format!(
                "{}/{} (finish)",
                format_duration(Duration::from_secs(duration)),
                format_duration(Duration::from_secs(current_secs))
            ));
            pb.finish_and_clear();
            deinit();
            return;
        }

        sleep(Duration::from_millis(100)).await;
        // 一時停止中はカウントしない
        if !music_play.lock().unwrap().is_paused() {
            tick_count += 1;

            // 10tick (1秒) ごとに更新
            if tick_count >= 10 {
                tick_count = 0;
                current_secs += 1;
                pb.set_position(current_secs);
                pb.set_message(format!(
                    "{}/{}",
                    format_duration(Duration::from_secs(current_secs)),
                    format_duration(Duration::from_secs(duration))
                ));
            }
        }
    }
}
