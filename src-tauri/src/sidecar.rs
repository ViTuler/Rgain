use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct SidecarState {
    pub child: Mutex<Option<SidecarProcess>>,
}

pub struct SidecarProcess {
    pub child: Child,
    pub stdin: ChildStdin,
    pub reader: BufReader<std::process::ChildStdout>,
    pub python_path: String,
    pub next_id: u64,
}

pub fn resolve_python_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // `tauri dev` cwd is usually `src-tauri` or repo root.
    let candidates = [
        cwd.join("python"),
        cwd.join("..").join("python"),
        cwd.join("../python"),
    ];
    for candidate in candidates {
        if candidate.join("rgain_parser").exists() {
            return candidate.canonicalize().unwrap_or(candidate);
        }
    }
    cwd.join("python")
}

pub fn ensure_sidecar(
    state: &SidecarState,
    python_path: &str,
) -> Result<(), String> {
    let mut guard = state.child.lock().map_err(|e| e.to_string())?;
    if let Some(proc) = guard.as_mut() {
        // Reuse if same interpreter and still alive.
        if proc.python_path == python_path {
            match proc.child.try_wait() {
                Ok(None) => return Ok(()),
                Ok(Some(status)) => {
                    let _ = status;
                }
                Err(_) => {}
            }
        } else {
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
        *guard = None;
    }

    let python_root = resolve_python_root();
    if !python_root.join("rgain_parser").exists() {
        return Err(format!(
            "python package not found under {}",
            python_root.display()
        ));
    }

    let mut child = Command::new(python_path)
        .arg("-u")
        .arg("-m")
        .arg("rgain_parser")
        .current_dir(&python_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn python failed: {e} (path={python_path})"))?;

    let stdin = child.stdin.take().ok_or("missing stdin")?;
    let stdout = child.stdout.take().ok_or("missing stdout")?;
    let mut reader = BufReader::new(stdout);

    // Wait for ready event.
    let ready = read_line_timeout(&mut reader, Duration::from_secs(5))?;
    let ready_json: Value =
        serde_json::from_str(&ready).map_err(|e| format!("ready parse: {e}; line={ready}"))?;
    if ready_json.get("event").and_then(|v| v.as_str()) != Some("ready") {
        return Err(format!("sidecar not ready: {ready}"));
    }

    *guard = Some(SidecarProcess {
        child,
        stdin,
        reader,
        python_path: python_path.to_string(),
        next_id: 1,
    });
    Ok(())
}

fn read_line_timeout(
    reader: &mut BufReader<std::process::ChildStdout>,
    timeout: Duration,
) -> Result<String, String> {
    // Simple blocking read with wall-clock timeout wrapper.
    // For Phase A this is enough; Phase B can move to async threads.
    let started = Instant::now();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return Err("sidecar closed stdout".into()),
            Ok(_) => {
                let trimmed = line.trim().to_string();
                if !trimmed.is_empty() {
                    return Ok(trimmed);
                }
            }
            Err(e) => return Err(e.to_string()),
        }
        if started.elapsed() > timeout {
            return Err("sidecar read timeout".into());
        }
    }
}

pub fn request(
    state: &SidecarState,
    python_path: &str,
    cmd: &str,
    payload: Option<Value>,
) -> Result<Value, String> {
    ensure_sidecar(state, python_path)?;
    let mut guard = state.child.lock().map_err(|e| e.to_string())?;
    let proc = guard.as_mut().ok_or("sidecar not running")?;

    let id = proc.next_id;
    proc.next_id += 1;

    let mut msg = json!({
        "id": id,
        "cmd": cmd,
    });
    if let Some(payload) = payload {
        msg["payload"] = payload;
    }

    writeln!(proc.stdin, "{}", msg).map_err(|e| e.to_string())?;
    proc.stdin.flush().map_err(|e| e.to_string())?;

    let line = read_line_timeout(&mut proc.reader, Duration::from_secs(8))?;
    let value: Value =
        serde_json::from_str(&line).map_err(|e| format!("response parse: {e}; line={line}"))?;
    Ok(value)
}

pub fn shutdown(state: &SidecarState) {
    if let Ok(mut guard) = state.child.lock() {
        if let Some(mut proc) = guard.take() {
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
    }
}
