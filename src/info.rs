use crate::{err, player::metadata::MetaData};
use crossterm::{cursor, execute, terminal::Clear, terminal::ClearType};
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep};

static LAST_CALL: once_cell::sync::Lazy<Arc<Mutex<Option<Instant>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

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

pub fn info_with_restore<P: AsRef<str>>(msg: P, filename: String, metadata: MetaData) {
    info(msg);

    tokio::spawn(async move {
        let call_time = Instant::now();
        {
            let mut last = LAST_CALL.lock().await;
            *last = Some(call_time);
        }

        sleep(Duration::from_millis(2400)).await;

        let last = LAST_CALL.lock().await;
        if let Some(last_time) = *last
            && last_time != call_time {
                return;
            }

        execute!(
            std::io::stdout(),
            cursor::MoveToPreviousLine(1),
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine)
        )
        .unwrap_or_else(|e| {
            err!("Failed to display info: {}", e);
            exit(1);
        });
        crate::display_info::display_info(&filename, &metadata);
    });
}
