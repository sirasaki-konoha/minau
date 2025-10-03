use std::{fs, path::Path, process::exit};
use crate::{err, play_music};

fn parse(m3u: &str) -> Vec<String> {
    m3u.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() || !line.starts_with('#'))
        .map(String::from)
        .collect()
}

pub async fn play_m3u<P: AsRef<Path>>(path: P, volume: f32, gui: bool) {
    let path = path.as_ref();
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        err!("Failed to read m3u file: {}", e);
        exit(1);
    });
    
    for file in parse(&content) {
        // TODO: Support url case
        if file.starts_with("http://") || file.starts_with("https://") {
            err!("{}: minau is not supporting url", &file);
            continue;
        }
        
        let file_path = if Path::new(&file).is_absolute() {
            file
        } else {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&file)
                .to_string_lossy()
                .to_string()
        };
        
        play_music::play_music(file_path, volume, gui).await;
    }
}

