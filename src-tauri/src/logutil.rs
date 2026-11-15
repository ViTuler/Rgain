use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

pub fn init(app_data_dir: &Path) -> Result<PathBuf, String> {
    let dir = app_data_dir.join("logs");
    fs::create_dir_all(&dir).map_err(|e| format!("create log dir: {e}"))?;
    let path = dir.join("rgain.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open log file: {e}"))?;
    let _ = LOG_PATH.set(path.clone());
    let _ = LOG_FILE.set(Mutex::new(file));
    write("INFO", &format!("log_init path={}", path.display()));
    Ok(path)
}

pub fn log_dir() -> Option<PathBuf> {
    LOG_PATH.get().and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

fn now_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Keep stamp simple and locale-free for file grepping.
    format!("{secs}")
}

pub fn write(level: &str, message: &str) {
    let line = format!("[{}] [{level}] {message}\n", now_stamp());
    if let Some(lock) = LOG_FILE.get() {
        if let Ok(mut file) = lock.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
            return;
        }
    }
    // Fallback before init / if lock fails.
    eprint!("{line}");
}

pub fn info(message: &str) {
    write("INFO", message);
}

pub fn warn(message: &str) {
    write("WARN", message);
}

pub fn error(message: &str) {
    write("ERROR", message);
}
