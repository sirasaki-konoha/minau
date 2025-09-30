use crossterm::{cursor, execute, terminal::Clear, terminal::ClearType};
use std::io::Write;
use std::process::exit;
use tokio::io::AsyncWriteExt;

use crate::err;

pub fn info<P: AsRef<str>>(msg: P) {
    let message = msg.as_ref();
    let mut stdout = std::io::stdout();

    execute!(
        stdout,
        cursor::MoveToColumn(0),
        Clear(ClearType::CurrentLine)
    )
    .unwrap_or_else(|e| {
        err!("Failed to display info: {}", e);
        exit(1);
    });

    print!("{message}");
    stdout.flush().unwrap();
}
