use crate::logutil;
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const DEV_PYTHON: &str = r"E:\Conda\envs\rgain\python.exe";

pub struct SidecarState {
    pub child: Mutex<Option<SidecarProcess>>,
}

pub struct SidecarProcess {
    pub child: Child,
    pub stdin: ChildStdin,
    pub reader: BufReader<std::process::ChildStdout>,
    pub launch_key: String,
    pub next_id: u64,
}

enum SidecarLaunch {
    /// Dev: conda `rgain` python -m rgain_parser
    DevModule { python: PathBuf, cwd: PathBuf },
    /// Prod: PyInstaller binary
    BundledExe { exe: PathBuf },
}

impl SidecarLaunch {
    fn key(&self) -> String {
        match self {
            // utf8v1: force respawn after PYTHONUTF8 stdin fix for Chinese paths
            Self::DevModule { python, cwd } => {
                format!("dev:utf8v1:{}|{}", python.display(), cwd.display())
            }
            Self::BundledExe { exe } => format!("exe:utf8v1:{}", exe.display()),
        }
    }

    fn spawn(&self) -> Result<Child, String> {
        let stderr = sidecar_stderr_stdio();
        match self {
            Self::DevModule { python, cwd } => {
                let mut cmd = Command::new(python);
                cmd.arg("-u")
                    .arg("-m")
                    .arg("rgain_parser")
                    .current_dir(cwd)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(stderr);
                apply_sidecar_env(&mut cmd);
                hide_console_window(&mut cmd);
                cmd.spawn().map_err(|e| {
                    format!(
                        "spawn dev python failed: {e} (python={} cwd={})",
                        python.display(),
                        cwd.display()
                    )
                })
            }
            Self::BundledExe { exe } => {
                let mut cmd = Command::new(exe);
                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(stderr);
                apply_sidecar_env(&mut cmd);
                hide_console_window(&mut cmd);
                cmd.spawn()
                    .map_err(|e| format!("spawn bundled sidecar failed: {e} (exe={})", exe.display()))
            }
        }
    }
}

fn hide_console_window(cmd: &mut Command) {
    // Avoid a flash console when spawning python / console-mode helpers on Windows.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

fn sidecar_stderr_stdio() -> Stdio {
    if let Some(dir) = logutil::log_dir() {
        let path = dir.join("sidecar.stderr.log");
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
            return Stdio::from(file);
        }
    }
    Stdio::piped()
}

fn apply_sidecar_env(cmd: &mut Command) {
    // Windows system codepage stdin would corrupt UTF-8 Chinese book paths.
    cmd.env("PYTHONUTF8", "1");
    cmd.env("PYTHONIOENCODING", "utf-8");
    if let Some(dir) = logutil::log_dir() {
        cmd.env("RGAIN_LOG_DIR", dir);
    }
}

pub fn resolve_python_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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

fn bundled_exe_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("rgain_parser.exe"));
            out.push(dir.join("rgain_parser"));
            out.push(dir.join("sidecars").join("rgain_parser.exe"));
            out.push(dir.join("sidecars").join("rgain_parser"));
            out.push(dir.join("resources").join("rgain_parser.exe"));
            out.push(dir.join("resources").join("rgain_parser"));
        }
    }
    out
}

fn resolve_launch() -> Result<SidecarLaunch, String> {
    // Optional override for both modes.
    if let Ok(override_path) = std::env::var("RGAIN_SIDECAR") {
        let path = PathBuf::from(override_path);
        if path.exists() {
            return Ok(SidecarLaunch::BundledExe { exe: path });
        }
        return Err(format!("RGAIN_SIDECAR not found: {}", path.display()));
    }

    if cfg!(debug_assertions) {
        let python = std::env::var("RGAIN_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEV_PYTHON));
        if !python.exists() {
            return Err(format!(
                "dev python missing: {} (set RGAIN_PYTHON or install conda env rgain)",
                python.display()
            ));
        }
        let cwd = resolve_python_root();
        if !cwd.join("rgain_parser").exists() {
            return Err(format!("python package not found under {}", cwd.display()));
        }
        return Ok(SidecarLaunch::DevModule { python, cwd });
    }

    for path in bundled_exe_candidates() {
        if path.exists() {
            return Ok(SidecarLaunch::BundledExe { exe: path });
        }
    }
    Err(
        "bundled sidecar not found (expected rgain_parser[.exe] beside app or under sidecars/)"
            .into(),
    )
}

pub fn ensure_sidecar(state: &SidecarState) -> Result<(), String> {
    let launch = resolve_launch()?;
    let launch_key = launch.key();

    let mut guard = state.child.lock().map_err(|e| e.to_string())?;
    if let Some(proc) = guard.as_mut() {
        if proc.launch_key == launch_key {
            match proc.child.try_wait() {
                Ok(None) => return Ok(()),
                Ok(Some(_)) | Err(_) => {}
            }
        } else {
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
        *guard = None;
    }

    let mut child = launch.spawn()?;
    let stdin = child.stdin.take().ok_or("missing stdin")?;
    let stdout = child.stdout.take().ok_or("missing stdout")?;
    let mut reader = BufReader::new(stdout);

    // onefile PyInstaller cold-extract can exceed 8s on first launch.
    let ready = read_line_timeout(&mut reader, Duration::from_secs(60))?;
    let ready_json: Value =
        serde_json::from_str(&ready).map_err(|e| format!("ready parse: {e}; line={ready}"))?;
    if ready_json.get("event").and_then(|v| v.as_str()) != Some("ready") {
        return Err(format!("sidecar not ready: {ready}"));
    }

    *guard = Some(SidecarProcess {
        child,
        stdin,
        reader,
        launch_key,
        next_id: 1,
    });
    Ok(())
}

fn read_line_timeout(
    reader: &mut BufReader<std::process::ChildStdout>,
    timeout: Duration,
) -> Result<String, String> {
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

pub fn request(state: &SidecarState, cmd: &str, payload: Option<Value>) -> Result<Value, String> {
    ensure_sidecar(state)?;
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

    logutil::info(&format!("sidecar_req id={id} cmd={cmd}"));
    writeln!(proc.stdin, "{}", msg).map_err(|e| e.to_string())?;
    proc.stdin.flush().map_err(|e| e.to_string())?;

    // extract can be slow on large PDFs / EPUBs / MOBIs
    let timeout = if cmd == "extract" {
        Duration::from_secs(600)
    } else {
        Duration::from_secs(8)
    };
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            logutil::error(&format!("sidecar_timeout id={id} cmd={cmd}"));
            return Err("sidecar read timeout".into());
        }
        let line = read_line_timeout(&mut proc.reader, remaining)?;
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(err) => {
                // Libraries (e.g. mobi/loguru) may pollute stdout; skip noise.
                logutil::warn(&format!(
                    "sidecar_skip_non_json id={id} err={err} line={}",
                    truncate_for_log(&line, 240)
                ));
                continue;
            }
        };
        let resp_id = value.get("id").and_then(|v| v.as_u64());
        if resp_id == Some(id) {
            let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            logutil::info(&format!("sidecar_res id={id} cmd={cmd} ok={ok}"));
            return Ok(value);
        }
        logutil::warn(&format!(
            "sidecar_skip_unmatched id={id} got={:?} line={}",
            resp_id,
            truncate_for_log(&line, 240)
        ));
    }
}

fn truncate_for_log(text: &str, max: usize) -> String {
    let cleaned = text.replace('\n', "\\n");
    if cleaned.chars().count() <= max {
        return cleaned;
    }
    cleaned.chars().take(max).collect::<String>() + "…"
}

pub fn shutdown(state: &SidecarState) {
    if let Ok(mut guard) = state.child.lock() {
        if let Some(mut proc) = guard.take() {
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
    }
}

pub fn launch_description() -> String {
    match resolve_launch() {
        Ok(launch) => launch.key(),
        Err(err) => format!("unresolved:{err}"),
    }
}
