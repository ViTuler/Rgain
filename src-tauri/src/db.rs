use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
    pub python_path: String,
    pub explain_hotkey: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            python_path: r"E:\Conda\envs\vi\python.exe".into(),
            explain_hotkey: "Ctrl+Shift+E".into(),
        }
    }
}

pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("rgain.sqlite3")
}

pub fn open_db(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| e.to_string())?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_path TEXT NOT NULL UNIQUE,
            kind TEXT NOT NULL DEFAULT 'code',
            title TEXT,
            content_hash TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            prompt TEXT,
            model TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            finished_at TEXT
        );

        CREATE TABLE IF NOT EXISTS file_extracts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            kind TEXT NOT NULL,
            text TEXT,
            warnings TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(project_id, path, content_hash)
        );

        CREATE TABLE IF NOT EXISTS graph_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
            payload_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS term_explanations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER REFERENCES runs(id) ON DELETE SET NULL,
            term TEXT NOT NULL,
            context TEXT,
            explanation TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_settings(conn: &Connection) -> Result<AppSettings, String> {
    let mut settings = AppSettings::default();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        match key.as_str() {
            "api_base_url" => settings.api_base_url = value,
            "api_key" => settings.api_key = value,
            "model" => settings.model = value,
            "python_path" => settings.python_path = value,
            "explain_hotkey" => settings.explain_hotkey = value,
            _ => {}
        }
    }
    Ok(settings)
}

pub fn save_settings(conn: &Connection, settings: &AppSettings) -> Result<(), String> {
    let pairs = [
        ("api_base_url", settings.api_base_url.as_str()),
        ("api_key", settings.api_key.as_str()),
        ("model", settings.model.as_str()),
        ("python_path", settings.python_path.as_str()),
        ("explain_hotkey", settings.explain_hotkey.as_str()),
    ];
    for (key, value) in pairs {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn count_runs(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}
