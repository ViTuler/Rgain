use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const MIN_DEV_PORT: u16 = 20000;
pub const DEFAULT_DEV_PORT: u16 = 20015;

pub fn clamp_dev_port(port: u16) -> u16 {
    if port < MIN_DEV_PORT {
        MIN_DEV_PORT
    } else {
        port
    }
}

pub fn project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [cwd.clone(), cwd.join("..")];
    for candidate in candidates {
        if candidate.join("package.json").exists() || candidate.join("rgain.dev.json").exists() {
            return candidate.canonicalize().unwrap_or(candidate);
        }
    }
    cwd
}

pub fn read_dev_port_file() -> Result<u16, String> {
    let path = project_root().join("rgain.dev.json");
    let raw = fs::read_to_string(&path).map_err(|e| format!("read rgain.dev.json: {e}"))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse rgain.dev.json: {e}"))?;
    let port = value
        .get("dev_port")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "rgain.dev.json missing dev_port".to_string())?;
    if port > 65535 {
        return Err("dev_port out of range".into());
    }
    Ok(clamp_dev_port(port as u16))
}

pub fn sync_dev_port_files(port: u16) -> Result<(), String> {
    let port = clamp_dev_port(port);
    let root = project_root();
    write_rgain_dev_json(&root, port)?;
    update_tauri_dev_url(&root, port)?;
    Ok(())
}

fn write_rgain_dev_json(root: &Path, port: u16) -> Result<(), String> {
    let path = root.join("rgain.dev.json");
    let body = json!({ "dev_port": port });
    fs::write(&path, format!("{}\n", serde_json::to_string_pretty(&body).map_err(|e| e.to_string())?))
        .map_err(|e| format!("write rgain.dev.json: {e}"))?;
    Ok(())
}

fn update_tauri_dev_url(root: &Path, port: u16) -> Result<(), String> {
    let path = root.join("src-tauri").join("tauri.conf.json");
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("read tauri.conf.json: {e}"))?;
    let mut conf: Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse tauri.conf.json: {e}"))?;
    conf["build"]["devUrl"] = Value::String(format!("http://localhost:{port}"));
    let out = serde_json::to_string_pretty(&conf).map_err(|e| e.to_string())?;
    fs::write(&path, format!("{out}\n")).map_err(|e| format!("write tauri.conf.json: {e}"))?;
    Ok(())
}
