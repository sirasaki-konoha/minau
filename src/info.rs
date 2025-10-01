use crossterm::{cursor, execute, terminal::Clear, terminal::ClearType};
use std::process::exit;
use crate::err;

pub fn info<P: AsRef<str>>(msg: P) {
    let message = msg.as_ref();
    let mut stdout = std::io::stdout();

    execute!(
        stdout,
        cursor::MoveToPreviousLine(1),
        cursor::MoveToColumn(0),
        Clear(ClearType::CurrentLine)
    )
    .unwrap_or_else(|e| {
        err!("Failed to display info: {}", e);
        exit(1);
    });

    println!("{message}");
}
