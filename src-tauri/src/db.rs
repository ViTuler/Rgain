use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

/// Full settings used only inside Rust (api_key never leaves backend via get_settings).
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
    /// User-managed agent prompt (optional). Built-in schema prompt is separate.
    pub agent_prompt: String,
    /// UI + LLM output locale: zh | en | ja | ko
    pub ui_locale: String,
    pub explain_hotkey: String,
    /// Show selection magnifier for term explain.
    pub selection_explain_enabled: bool,
    pub dev_port: u16,
    /// Right-side node explain body, px.
    pub explain_font_px: u16,
    /// Selection / term explain body, px.
    pub term_font_px: u16,
    /// Logic-graph label size, px.
    pub graph_font_px: u16,
    /// Max extracted-text size (MiB) per LLM pass; oversized corpora are chunk-merged.
    pub chunk_size_mb: f64,
    /// Ask the model to run a thinking / reasoning pass when the API supports it.
    pub think_enabled: bool,
    /// Directory names or relative paths skipped while scanning a project folder.
    pub ignore_paths: Vec<String>,
    pub mcp_enabled: bool,
    pub mcp_server_url: String,
}

/// Settings payload for the frontend.
/// `api_key` is masked when set (`sk****xxxx`); empty / mask on save keeps the stored secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicSettings {
    pub api_base_url: String,
    #[serde(default)]
    pub api_key: String,
    pub api_key_set: bool,
    pub model: String,
    pub agent_prompt: String,
    #[serde(default = "default_ui_locale")]
    pub ui_locale: String,
    pub explain_hotkey: String,
    #[serde(default = "default_true")]
    pub selection_explain_enabled: bool,
    pub dev_port: u16,
    #[serde(default = "default_explain_font_px")]
    pub explain_font_px: u16,
    #[serde(default = "default_term_font_px")]
    pub term_font_px: u16,
    #[serde(default = "default_graph_font_px")]
    pub graph_font_px: u16,
    #[serde(default = "default_chunk_size_mb")]
    pub chunk_size_mb: f64,
    #[serde(default = "default_true")]
    pub think_enabled: bool,
    #[serde(default = "default_ignore_paths")]
    pub ignore_paths: Vec<String>,
    pub mcp_enabled: bool,
    pub mcp_server_url: String,
}

fn default_ui_locale() -> String {
    "zh".into()
}

fn default_true() -> bool {
    true
}

fn default_chunk_size_mb() -> f64 {
    0.5
}

pub const MIN_FONT_PX: u16 = 10;
pub const MAX_FONT_PX: u16 = 32;

fn default_explain_font_px() -> u16 {
    14
}

fn default_term_font_px() -> u16 {
    14
}

fn default_graph_font_px() -> u16 {
    12
}

pub fn default_ignore_paths() -> Vec<String> {
    [
        "node_modules",
        "dist",
        "dist-ssr",
        "build",
        "out",
        "target",
        ".git",
        ".svn",
        "__pycache__",
        ".venv",
        "venv",
        ".tox",
        ".pytest_cache",
        ".mypy_cache",
        ".ruff_cache",
        ".cache",
        ".parcel-cache",
        ".turbo",
        ".next",
        ".nuxt",
        ".output",
        "coverage",
        "htmlcov",
        ".nyc_output",
        "vendor",
        "bower_components",
        ".gradle",
        ".dart_tool",
        "bin",
        "obj",
        ".idea",
        ".vscode",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub fn normalize_ignore_path(raw: &str) -> String {
    let mut text = raw.trim().replace('\\', "/");
    while let Some(rest) = text.strip_prefix("./") {
        text = rest.to_string();
    }
    text = text.trim_matches('/').to_string();
    if text.is_empty() || text == "." || text == ".." || text.split('/').any(|part| part == "..") {
        return String::new();
    }
    if text.len() > 120 || text.chars().nth(1) == Some(':') {
        return String::new();
    }
    text
}

pub fn normalize_ignore_paths(paths: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in paths {
        let path = normalize_ignore_path(&raw);
        let key = path.to_ascii_lowercase();
        if path.is_empty() || !seen.insert(key) {
            continue;
        }
        out.push(path);
        if out.len() >= 64 {
            break;
        }
    }
    out
}

pub fn clamp_font_px(value: u16, fallback: u16) -> u16 {
    if value < MIN_FONT_PX {
        return fallback.clamp(MIN_FONT_PX, MAX_FONT_PX);
    }
    value.clamp(MIN_FONT_PX, MAX_FONT_PX)
}

pub fn clamp_chunk_size_mb(value: f64) -> f64 {
    if !value.is_finite() {
        return default_chunk_size_mb();
    }
    let rounded = (value * 100.0).round() / 100.0;
    rounded.clamp(0.1, 8.0)
}

pub fn normalize_ui_locale(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "en" | "en-us" | "en_gb" => "en".into(),
        "ja" | "jp" | "ja-jp" => "ja".into(),
        "ko" | "kr" | "ko-kr" => "ko".into(),
        "zh" | "zh-cn" | "zh-hans" | "cn" => "zh".into(),
        _ => "zh".into(),
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            agent_prompt: String::new(),
            ui_locale: "zh".into(),
            explain_hotkey: "Ctrl+Shift+E".into(),
            selection_explain_enabled: true,
            dev_port: crate::dev_port::DEFAULT_DEV_PORT,
            explain_font_px: default_explain_font_px(),
            term_font_px: default_term_font_px(),
            graph_font_px: default_graph_font_px(),
            chunk_size_mb: default_chunk_size_mb(),
            think_enabled: true,
            ignore_paths: default_ignore_paths(),
            mcp_enabled: false,
            mcp_server_url: String::new(),
        }
    }
}

/// Mask plain key for UI: `sk****xxxx` (prefix 2 + stars + last 4).
pub fn mask_api_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = key.chars().collect();
    let n = chars.len();
    let prefix_len = if n >= 2 { 2 } else { n };
    let suffix_len = 4.min(n.saturating_sub(prefix_len));
    if n <= prefix_len + suffix_len {
        let prefix: String = chars.iter().take(prefix_len).collect();
        return format!("{prefix}****");
    }
    let prefix: String = chars[..prefix_len].iter().collect();
    let suffix: String = chars[n - suffix_len..].iter().collect();
    // At least 4 stars; grow with key length, cap for readability.
    let star_count = (n - prefix_len - suffix_len).clamp(4, 16);
    format!("{prefix}{}{suffix}", "*".repeat(star_count))
}

pub fn looks_like_masked_api_key(value: &str) -> bool {
    let v = value.trim();
    v.contains("****")
}

/// True when frontend did not supply a new plain key.
pub fn is_keep_api_key(incoming: &str, current_plain: &str) -> bool {
    let v = incoming.trim();
    if v.is_empty() || v == "******" || v == "********" {
        return true;
    }
    if looks_like_masked_api_key(v) {
        return true;
    }
    if !current_plain.is_empty() && v == mask_api_key(current_plain) {
        return true;
    }
    false
}

impl AppSettings {
    pub fn to_public(&self) -> PublicSettings {
        let set = !self.api_key.trim().is_empty();
        PublicSettings {
            api_base_url: self.api_base_url.clone(),
            api_key: if set {
                mask_api_key(&self.api_key)
            } else {
                String::new()
            },
            api_key_set: set,
            model: self.model.clone(),
            agent_prompt: self.agent_prompt.clone(),
            ui_locale: normalize_ui_locale(&self.ui_locale),
            explain_hotkey: self.explain_hotkey.clone(),
            selection_explain_enabled: self.selection_explain_enabled,
            dev_port: self.dev_port,
            explain_font_px: clamp_font_px(self.explain_font_px, default_explain_font_px()),
            term_font_px: clamp_font_px(self.term_font_px, default_term_font_px()),
            graph_font_px: clamp_font_px(self.graph_font_px, default_graph_font_px()),
            chunk_size_mb: clamp_chunk_size_mb(self.chunk_size_mb),
            think_enabled: self.think_enabled,
            ignore_paths: normalize_ignore_paths(self.ignore_paths.clone()),
            mcp_enabled: self.mcp_enabled,
            mcp_server_url: self.mcp_server_url.clone(),
        }
    }
}

/// Fixed SQLCipher passphrase for debug / `tauri dev`.
/// Release builds only use this to migrate older DBs, then rekey to a random keyring secret.
const LEGACY_SQLITE_KEY: &str = "#sqPX)4XvaaLBd";

/// Must match `tauri.conf.json` → `identifier` (app data + credential scope).
const KEYRING_SERVICE: &str = "com.ViTuler.Rgain";
const KEYRING_USER: &str = "sqlcipher-db-key";

pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("Rgain.sqlite3")
}

fn db_is_readable(conn: &Connection) -> bool {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        row.get::<_, i64>(0)
    })
    .is_ok()
}

fn apply_cipher_key(conn: &Connection, key: &str) -> Result<(), String> {
    conn.pragma_update(None, "key", key)
        .map_err(|e| format!("sqlcipher key: {e}"))
}

fn rekey_cipher(conn: &Connection, key: &str) -> Result<(), String> {
    conn.pragma_update(None, "rekey", key)
        .map_err(|e| format!("sqlcipher rekey: {e}"))?;
    if !db_is_readable(conn) {
        return Err("sqlcipher: rekey succeeded but database is not readable".into());
    }
    Ok(())
}

fn sql_quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn remove_sqlite_sidecars(path: &Path) {
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut p = path.as_os_str().to_os_string();
        p.push(suffix);
        let _ = std::fs::remove_file(PathBuf::from(p));
    }
}

/// Plaintext → SQLCipher: `PRAGMA rekey` is not allowed; use ATTACH + sqlcipher_export.
fn encrypt_plaintext_db(path: &Path, key: &str) -> Result<Connection, String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("rgain.sqlite3");
    let tmp = parent.join(format!("{stem}.encrypting.tmp"));
    let bak = parent.join(format!("{stem}.pre_cipher.bak"));

    let _ = std::fs::remove_file(&tmp);

    let plain = Connection::open(path).map_err(|e| e.to_string())?;
    if !db_is_readable(&plain) {
        return Err(
            "sqlcipher: database exists but cannot be opened (missing keyring secret or corrupt)"
                .into(),
        );
    }
    let _ = plain.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

    let tmp_lit = sql_quote_literal(&tmp.to_string_lossy());
    let key_lit = sql_quote_literal(key);
    plain
        .execute_batch(&format!(
            "ATTACH DATABASE {tmp_lit} AS encrypted KEY {key_lit};
             SELECT sqlcipher_export('encrypted');
             DETACH DATABASE encrypted;"
        ))
        .map_err(|e| format!("sqlcipher_export: {e}"))?;
    drop(plain);

    remove_sqlite_sidecars(path);
    if bak.exists() {
        let _ = std::fs::remove_file(&bak);
    }
    std::fs::rename(path, &bak).map_err(|e| format!("sqlcipher backup rename: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::rename(&bak, path);
        format!("sqlcipher replace encrypted: {e}")
    })?;

    try_open_with_key(path, key)
}

fn generate_db_key() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn dpapi_key_path(db_path: &Path) -> PathBuf {
    db_path.with_file_name("sqlcipher.dpapi")
}

#[cfg(windows)]
fn dpapi_protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: plain.len() as u32,
        pbData: plain.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &mut input,
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("dpapi protect failed".into());
    }
    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
        let owned = slice.to_vec();
        let _ = LocalFree(output.pbData as _);
        owned
    };
    Ok(bytes)
}

#[cfg(windows)]
fn dpapi_unprotect(blob: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: blob.len() as u32,
        pbData: blob.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &mut input,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("dpapi unprotect failed".into());
    }
    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(output.pbData, output.cbData as usize);
        let owned = slice.to_vec();
        let _ = LocalFree(output.pbData as _);
        owned
    };
    Ok(bytes)
}

#[cfg(not(windows))]
fn dpapi_protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    Ok(plain.to_vec())
}

#[cfg(not(windows))]
fn dpapi_unprotect(blob: &[u8]) -> Result<Vec<u8>, String> {
    Ok(blob.to_vec())
}

fn load_dpapi_key(db_path: &Path) -> Option<String> {
    let path = dpapi_key_path(db_path);
    let blob = std::fs::read(&path).ok()?;
    let plain = dpapi_unprotect(&blob).ok()?;
    let key = String::from_utf8(plain).ok()?;
    let key = key.trim().to_string();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn save_dpapi_key(db_path: &Path, key: &str) -> Result<(), String> {
    let path = dpapi_key_path(db_path);
    let blob = dpapi_protect(key.as_bytes())?;
    std::fs::write(&path, blob).map_err(|e| format!("write dpapi key: {e}"))
}

fn load_keyring_key() -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).ok()?;
    match entry.get_password() {
        Ok(key) if !key.trim().is_empty() => Some(key),
        _ => None,
    }
}

fn save_keyring_key(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("keyring entry: {e}"))?;
    entry
        .set_password(key)
        .map_err(|e| format!("keyring set: {e}"))
}

/// Persist release key to DPAPI file (authoritative) and best-effort keyring mirror.
fn persist_release_key(db_path: &Path, key: &str) -> Result<(), String> {
    save_dpapi_key(db_path, key)?;
    if let Err(err) = save_keyring_key(key) {
        crate::logutil::warn(&format!("keyring mirror failed (dpapi ok): {err}"));
    }
    Ok(())
}

/// Release: load existing key, or create one only when the DB does not exist yet.
fn get_or_create_release_db_key(db_path: &Path, preexisting_db: bool) -> Result<String, String> {
    if let Some(key) = load_dpapi_key(db_path) {
        crate::logutil::info("db_key_source=dpapi_file");
        let _ = save_keyring_key(&key);
        return Ok(key);
    }
    if let Some(key) = load_keyring_key() {
        crate::logutil::info("db_key_source=keyring");
        // Heal missing DPAPI mirror so later restarts do not regenerate.
        if let Err(err) = save_dpapi_key(db_path, &key) {
            crate::logutil::warn(&format!("dpapi heal failed: {err}"));
        }
        return Ok(key);
    }

    if preexisting_db {
        return Err(
            "sqlcipher: existing DB found but no persisted key \
             (sqlcipher.dpapi / keyring missing). Delete Rgain.sqlite3 to reset."
                .into(),
        );
    }

    let key = generate_db_key();
    persist_release_key(db_path, &key)?;
    crate::logutil::info("db_key_source=generated_new");
    Ok(key)
}

fn resolve_db_key(db_path: &Path, preexisting_db: bool) -> Result<String, String> {
    if cfg!(debug_assertions) {
        // Dev / tauri dev: stable fixed key, no Windows Credential Manager.
        crate::logutil::info("db_key_source=legacy_debug");
        Ok(LEGACY_SQLITE_KEY.to_string())
    } else {
        get_or_create_release_db_key(db_path, preexisting_db)
    }
}

fn try_open_with_key(path: &Path, key: &str) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    apply_cipher_key(&conn, key)?;
    if db_is_readable(&conn) {
        return Ok(conn);
    }
    Err("sqlcipher: key rejected".into())
}

fn open_encrypted_or_migrate(path: &Path) -> Result<Connection, String> {
    let preexisting = path.is_file()
        && std::fs::metadata(path)
            .map(|m| m.len() > 0)
            .unwrap_or(false);

    let db_key = resolve_db_key(path, preexisting)?;

    // New / empty file.
    if !preexisting {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        apply_cipher_key(&conn, &db_key)?;
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")
            .map_err(|e| format!("sqlcipher init: {e}"))?;
        return Ok(conn);
    }

    if let Ok(conn) = try_open_with_key(path, &db_key) {
        return Ok(conn);
    }

    // Release only: migrate DBs that were still on the old fixed passphrase.
    if !cfg!(debug_assertions) {
        if let Ok(conn) = try_open_with_key(path, LEGACY_SQLITE_KEY) {
            rekey_cipher(&conn, &db_key)?;
            let _ = persist_release_key(path, &db_key);
            return Ok(conn);
        }
    }

    // Plaintext DB (pre-SQLCipher): export into a new encrypted file.
    encrypt_plaintext_db(path, &db_key).map_err(|err| {
        if err.contains("missing keyring secret or corrupt") {
            format!(
                "sqlcipher: cannot open existing DB with current key. \
                 Delete Rgain.sqlite3 (and sqlcipher.dpapi) under the app data dir to reset. \
                 detail={err}"
            )
        } else {
            err
        }
    })
}

pub fn open_db(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let conn = open_encrypted_or_migrate(path)?;
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
            finished_at TEXT,
            title TEXT
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

        CREATE TABLE IF NOT EXISTS graph_layouts (
            run_id INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
            view_key TEXT NOT NULL DEFAULT '2d-flow',
            layout_json TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (run_id, view_key)
        );
        "#,
    )
    .map_err(|e| e.to_string())?;
    ensure_runs_title_column(conn)?;
    backfill_run_titles(conn)?;
    Ok(())
}

fn ensure_runs_title_column(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(runs)")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    let mut has_title = false;
    for row in rows {
        if row.map_err(|e| e.to_string())? == "title" {
            has_title = true;
            break;
        }
    }
    if !has_title {
        conn.execute("ALTER TABLE runs ADD COLUMN title TEXT", [])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn backfill_run_titles(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT r.id, g.payload_json
             FROM runs r
             LEFT JOIN graph_snapshots g ON g.id = (
               SELECT id FROM graph_snapshots WHERE run_id = r.id ORDER BY id DESC LIMIT 1
             )
             WHERE r.title IS NULL OR trim(r.title) = ''",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (run_id, payload) = row.map_err(|e| e.to_string())?;
        let title = payload
            .as_deref()
            .and_then(title_from_graph_json)
            .unwrap_or_else(|| format!("Run #{run_id}"));
        conn.execute(
            "UPDATE runs SET title=?1 WHERE id=?2",
            params![title, run_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn title_from_graph_json(payload_json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(payload_json).ok()?;
    value
        .get("meta")?
        .get("title")?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
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
            "agent_prompt" => settings.agent_prompt = value,
            "ui_locale" => settings.ui_locale = normalize_ui_locale(&value),
            // python_path legacy rows ignored — sidecar resolves dev/prod automatically
            "python_path" => {}
            "explain_hotkey" => settings.explain_hotkey = value,
            "selection_explain_enabled" => {
                settings.selection_explain_enabled =
                    matches!(value.as_str(), "1" | "true" | "TRUE" | "yes")
            }
            "mcp_enabled" => {
                settings.mcp_enabled = matches!(value.as_str(), "1" | "true" | "TRUE" | "yes")
            }
            "mcp_server_url" => settings.mcp_server_url = value,
            "dev_port" => {
                if let Ok(port) = value.parse::<u16>() {
                    settings.dev_port = crate::dev_port::clamp_dev_port(port);
                }
            }
            "explain_font_px" => {
                if let Ok(px) = value.parse::<u16>() {
                    settings.explain_font_px = clamp_font_px(px, default_explain_font_px());
                }
            }
            "term_font_px" => {
                if let Ok(px) = value.parse::<u16>() {
                    settings.term_font_px = clamp_font_px(px, default_term_font_px());
                }
            }
            "graph_font_px" => {
                if let Ok(px) = value.parse::<u16>() {
                    settings.graph_font_px = clamp_font_px(px, default_graph_font_px());
                }
            }
            "chunk_size_mb" => {
                if let Ok(mb) = value.parse::<f64>() {
                    settings.chunk_size_mb = clamp_chunk_size_mb(mb);
                }
            }
            "think_enabled" => {
                settings.think_enabled = matches!(value.as_str(), "1" | "true" | "TRUE" | "yes")
            }
            "ignore_paths" => {
                let parsed: Vec<String> = serde_json::from_str(&value).unwrap_or_default();
                settings.ignore_paths = normalize_ignore_paths(parsed);
            }
            // theme is frontend-only (localStorage); ignore legacy DB rows
            _ => {}
        }
    }
    settings.ui_locale = normalize_ui_locale(&settings.ui_locale);
    settings.chunk_size_mb = clamp_chunk_size_mb(settings.chunk_size_mb);
    Ok(settings)
}

pub fn save_settings(conn: &Connection, settings: &AppSettings) -> Result<(), String> {
    let previous_port = get_settings(conn).ok().map(|s| s.dev_port);
    let next_port = crate::dev_port::clamp_dev_port(settings.dev_port);
    let dev_port = next_port.to_string();
    let mcp_enabled = if settings.mcp_enabled { "true" } else { "false" };
    let selection_explain_enabled = if settings.selection_explain_enabled {
        "true"
    } else {
        "false"
    };
    let ui_locale = normalize_ui_locale(&settings.ui_locale);
    let chunk_size_mb = clamp_chunk_size_mb(settings.chunk_size_mb).to_string();
    let explain_font_px =
        clamp_font_px(settings.explain_font_px, default_explain_font_px()).to_string();
    let term_font_px = clamp_font_px(settings.term_font_px, default_term_font_px()).to_string();
    let graph_font_px = clamp_font_px(settings.graph_font_px, default_graph_font_px()).to_string();
    let think_enabled = if settings.think_enabled { "true" } else { "false" };
    let ignore_paths = serde_json::to_string(&normalize_ignore_paths(settings.ignore_paths.clone()))
        .unwrap_or_else(|_| "[]".into());
    let pairs = [
        ("api_base_url", settings.api_base_url.as_str()),
        ("api_key", settings.api_key.as_str()),
        ("model", settings.model.as_str()),
        ("agent_prompt", settings.agent_prompt.as_str()),
        ("ui_locale", ui_locale.as_str()),
        ("explain_hotkey", settings.explain_hotkey.as_str()),
        ("selection_explain_enabled", selection_explain_enabled),
        ("dev_port", dev_port.as_str()),
        ("explain_font_px", explain_font_px.as_str()),
        ("term_font_px", term_font_px.as_str()),
        ("graph_font_px", graph_font_px.as_str()),
        ("chunk_size_mb", chunk_size_mb.as_str()),
        ("think_enabled", think_enabled),
        ("ignore_paths", ignore_paths.as_str()),
        ("mcp_enabled", mcp_enabled),
        ("mcp_server_url", settings.mcp_server_url.as_str()),
    ];
    for (key, value) in pairs {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    // Only rewrite vite/tauri port files when the port actually changes — avoids tauri-dev restarts.
    if previous_port != Some(next_port) {
        crate::dev_port::sync_dev_port_files(next_port)?;
    }
    Ok(())
}

/// Apply frontend public settings; keep stored api_key unless a new one is provided.
pub fn save_public_settings(conn: &Connection, incoming: &PublicSettings) -> Result<(), String> {
    let mut current = get_settings(conn)?;
    current.api_base_url = incoming.api_base_url.clone();
    current.model = incoming.model.clone();
    current.agent_prompt = incoming.agent_prompt.clone();
    current.ui_locale = normalize_ui_locale(&incoming.ui_locale);
    current.explain_hotkey = incoming.explain_hotkey.clone();
    current.selection_explain_enabled = incoming.selection_explain_enabled;
    current.dev_port = crate::dev_port::clamp_dev_port(incoming.dev_port);
    current.explain_font_px = clamp_font_px(incoming.explain_font_px, default_explain_font_px());
    current.term_font_px = clamp_font_px(incoming.term_font_px, default_term_font_px());
    current.graph_font_px = clamp_font_px(incoming.graph_font_px, default_graph_font_px());
    current.chunk_size_mb = clamp_chunk_size_mb(incoming.chunk_size_mb);
    current.think_enabled = incoming.think_enabled;
    current.ignore_paths = normalize_ignore_paths(incoming.ignore_paths.clone());
    current.mcp_enabled = incoming.mcp_enabled;
    current.mcp_server_url = incoming.mcp_server_url.clone();
    if !is_keep_api_key(&incoming.api_key, &current.api_key) {
        current.api_key = incoming.api_key.trim().to_string();
    }
    save_settings(conn, &current)
}

pub fn upsert_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn count_runs(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRow {
    pub id: i64,
    pub root_path: String,
    pub kind: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRow {
    pub id: i64,
    pub project_id: Option<i64>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub status: String,
    pub created_at: String,
    /// Display title in history; seeded from graph `meta.title`.
    pub title: Option<String>,
}

pub fn upsert_project(
    conn: &Connection,
    root_path: &str,
    kind: &str,
    title: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO projects(root_path, kind, title, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(root_path) DO UPDATE SET
           kind=excluded.kind,
           title=excluded.title,
           updated_at=datetime('now')",
        params![root_path, kind, title],
    )
    .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id FROM projects WHERE root_path=?1",
        params![root_path],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn get_project(conn: &Connection, id: i64) -> Result<ProjectRow, String> {
    conn.query_row(
        "SELECT id, root_path, kind, title FROM projects WHERE id=?1",
        params![id],
        |row| {
            Ok(ProjectRow {
                id: row.get(0)?,
                root_path: row.get(1)?,
                kind: row.get(2)?,
                title: row.get(3)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn list_runs(conn: &Connection, limit: i64) -> Result<Vec<RunRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, prompt, model, status, created_at, title
             FROM runs ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(RunRow {
                id: row.get(0)?,
                project_id: row.get(1)?,
                prompt: row.get(2)?,
                model: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                title: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn set_run_title(conn: &Connection, run_id: i64, title: &str) -> Result<(), String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err("title_empty".into());
    }
    let n = conn
        .execute(
            "UPDATE runs SET title=?1 WHERE id=?2",
            params![trimmed, run_id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("run_not_found".into());
    }

    // Keep latest snapshot meta.title in sync for reopen UI.
    if let Some(payload) = get_graph_for_run(conn, run_id)? {
        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&payload) {
            if let Some(meta) = value.get_mut("meta").and_then(|m| m.as_object_mut()) {
                meta.insert(
                    "title".into(),
                    serde_json::Value::String(trimmed.to_string()),
                );
                if let Ok(next) = serde_json::to_string(&value) {
                    conn.execute(
                        "UPDATE graph_snapshots SET payload_json=?1
                         WHERE id = (
                           SELECT id FROM graph_snapshots WHERE run_id=?2 ORDER BY id DESC LIMIT 1
                         )",
                        params![next, run_id],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    Ok(())
}

pub fn delete_run(conn: &Connection, run_id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM runs WHERE id=?1", params![run_id])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("run_not_found".into());
    }
    Ok(())
}

pub fn create_run(
    conn: &Connection,
    project_id: i64,
    prompt: &str,
    model: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO runs(project_id, prompt, model, status) VALUES (?1, ?2, ?3, 'running')",
        params![project_id, prompt, model],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn finish_run(conn: &Connection, run_id: i64, status: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE runs SET status=?1, finished_at=datetime('now') WHERE id=?2",
        params![status, run_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn save_graph_snapshot(conn: &Connection, run_id: i64, payload_json: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO graph_snapshots(run_id, payload_json) VALUES (?1, ?2)",
        params![run_id, payload_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermExplanationRow {
    pub id: i64,
    pub run_id: Option<i64>,
    pub term: String,
    pub context: Option<String>,
    pub explanation: String,
    pub created_at: String,
}

pub fn save_term_explanation(
    conn: &Connection,
    run_id: Option<i64>,
    term: &str,
    context: &str,
    explanation: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO term_explanations(run_id, term, context, explanation)
         VALUES (?1, ?2, ?3, ?4)",
        params![run_id, term, context, explanation],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn list_term_explanations(
    conn: &Connection,
    limit: i64,
) -> Result<Vec<TermExplanationRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, term, context, explanation, created_at
             FROM term_explanations ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(TermExplanationRow {
                id: row.get(0)?,
                run_id: row.get(1)?,
                term: row.get(2)?,
                context: row.get(3)?,
                explanation: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn get_graph_for_run(conn: &Connection, run_id: i64) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT payload_json FROM graph_snapshots WHERE run_id=?1 ORDER BY id DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![run_id]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(row.get(0).map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

pub const DEFAULT_LAYOUT_VIEW_KEY: &str = "2d-flow";

pub fn get_graph_layout(
    conn: &Connection,
    run_id: i64,
    view_key: &str,
) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT layout_json FROM graph_layouts WHERE run_id=?1 AND view_key=?2 LIMIT 1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query(params![run_id, view_key])
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(row.get(0).map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

pub fn save_graph_layout(
    conn: &Connection,
    run_id: i64,
    view_key: &str,
    layout_json: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO graph_layouts(run_id, view_key, layout_json, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(run_id, view_key) DO UPDATE SET
           layout_json=excluded.layout_json,
           updated_at=datetime('now')",
        params![run_id, view_key, layout_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_graph_layout(
    conn: &Connection,
    run_id: i64,
    view_key: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM graph_layouts WHERE run_id=?1 AND view_key=?2",
        params![run_id, view_key],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_cached_extract(
    conn: &Connection,
    project_id: i64,
    path: &str,
    content_hash: &str,
) -> Result<Option<(String, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT text, kind FROM file_extracts
             WHERE project_id=?1 AND path=?2 AND content_hash=?3
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query(params![project_id, path, content_hash])
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let text: Option<String> = row.get(0).map_err(|e| e.to_string())?;
        let kind: String = row.get(1).map_err(|e| e.to_string())?;
        Ok(Some((text.unwrap_or_default(), kind)))
    } else {
        Ok(None)
    }
}

pub fn upsert_file_extract(
    conn: &Connection,
    project_id: i64,
    path: &str,
    content_hash: &str,
    kind: &str,
    text: &str,
    warnings: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO file_extracts(project_id, path, content_hash, kind, text, warnings)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(project_id, path, content_hash) DO UPDATE SET
           kind=excluded.kind,
           text=excluded.text,
           warnings=excluded.warnings",
        params![project_id, path, content_hash, kind, text, warnings],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
