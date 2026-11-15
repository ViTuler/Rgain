use crate::db;
use crate::logutil;
use crate::sidecar::{self, SidecarState};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "venv",
    ".idea",
    ".vscode",
    "vendor",
];

const TEXT_EXTS: &[&str] = &[
    "md", "txt", "py", "rs", "ts", "tsx", "js", "jsx", "vue", "cpp", "cc", "cxx", "c", "h",
    "hpp", "rc", "java", "kt", "go", "json", "toml", "yaml", "yml", "css", "scss", "html",
    "xml", "cmake", "makefile", "mk", "sh", "ps1", "sql", "ini", "cfg", "conf",
];

const DOC_EXTS: &[&str] = &["pdf", "epub", "mobi"];
/// Soft cap for text/code files while walking a folder.
const MAX_TEXT_FILE_BYTES: u64 = 1_500_000;
/// Books (pdf/epub/mobi) may be large; skip only absurd sizes during walk.
const MAX_DOC_FILE_BYTES: u64 = 200_000_000;
/// Corpus budget sent to the LLM (large books truncate with a warning).
const MAX_TOTAL_CHARS: usize = 400_000;
const MAX_FILES: usize = 120;

pub fn is_supported_book_file(path: &Path) -> bool {
    path.is_file() && is_doc_ext(&ext_of(path))
}

pub fn is_supported_project_root(path: &Path) -> bool {
    path.is_dir() || is_supported_book_file(path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFile {
    pub path: String,
    pub kind: String,
    pub content_hash: String,
    pub text: String,
    pub cached: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    pub project_id: i64,
    pub run_id: i64,
    pub status: String,
    pub files_processed: usize,
    pub files_cached: usize,
    pub graph: Option<Value>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn ext_of(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn is_text_ext(ext: &str) -> bool {
    TEXT_EXTS.iter().any(|e| *e == ext)
}

fn is_doc_ext(ext: &str) -> bool {
    DOC_EXTS.iter().any(|e| *e == ext)
}

fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.iter().any(|d| *d == name)
}

pub fn collect_candidate_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if root.is_file() {
        if !is_supported_book_file(root) {
            return Err(format!(
                "unsupported_book_file:{}",
                ext_of(root)
            ));
        }
        return Ok(vec![root.to_path_buf()]);
    }
    if !root.is_dir() {
        return Err("root_path_missing".into());
    }
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    if out.len() > MAX_FILES {
        out.truncate(MAX_FILES);
    }
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if out.len() >= MAX_FILES {
        return Ok(());
    }
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != ".env.example" {
            // keep hidden files skipped except we already listed common dirs
            if path.is_dir() {
                continue;
            }
        }
        if path.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }
            walk(root, &path, out)?;
            continue;
        }
        let ext = ext_of(&path);
        let file_name = name.to_ascii_lowercase();
        let is_makefile = file_name == "makefile" || file_name == "cmakelists.txt";
        if !(is_text_ext(&ext) || is_doc_ext(&ext) || is_makefile) {
            continue;
        }
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        let max_bytes = if is_doc_ext(&ext) {
            MAX_DOC_FILE_BYTES
        } else {
            MAX_TEXT_FILE_BYTES
        };
        if meta.len() > max_bytes {
            continue;
        }
        out.push(path);
        if out.len() >= MAX_FILES {
            break;
        }
    }
    Ok(())
}

fn rel_path(root: &Path, path: &Path) -> String {
    if root.is_file() {
        return root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("book")
            .replace('\\', "/");
    }
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_text_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    // Reject obvious binary
    if bytes.iter().take(1024).any(|b| *b == 0) {
        return Err("binary_file".into());
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn extract_doc_via_sidecar(
    sidecar: &SidecarState,
    path: &Path,
) -> Result<(String, Vec<String>), String> {
    logutil::info(&format!("extract_doc_begin path={}", path.display()));
    let payload = json!({ "path": path.to_string_lossy() });
    let value = sidecar::request(sidecar, "extract", Some(payload))?;
    if value.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        let err = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("extract_failed");
        logutil::error(&format!(
            "extract_doc_fail path={} err={err}",
            path.display()
        ));
        return Err(err.to_string());
    }
    let data = value.get("data").cloned().unwrap_or(json!({}));
    let text = data
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let warnings = data
        .get("warnings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if text.trim().is_empty() {
        let detail = if warnings.is_empty() {
            "extract_empty_text".to_string()
        } else {
            format!("extract_empty_text:{}", warnings.join(";"))
        };
        logutil::warn(&format!(
            "extract_doc_empty path={} detail={detail}",
            path.display()
        ));
        return Err(detail);
    }
    logutil::info(&format!(
        "extract_doc_ok path={} chars={} warnings={}",
        path.display(),
        text.len(),
        warnings.len()
    ));
    Ok((text, warnings))
}

pub fn extract_project_files(
    conn: &Connection,
    sidecar: &SidecarState,
    project_id: i64,
    root: &Path,
) -> Result<(Vec<ExtractedFile>, Vec<String>), String> {
    logutil::info(&format!(
        "extract_project_begin project_id={project_id} root={}",
        root.display()
    ));
    let files = collect_candidate_files(root)?;
    logutil::info(&format!(
        "extract_candidates project_id={project_id} count={}",
        files.len()
    ));
    let mut extracted = Vec::new();
    let mut warnings = Vec::new();
    let mut total_chars = 0usize;

    for path in files {
        let rel = rel_path(root, &path);
        let hash = match file_sha256(&path) {
            Ok(h) => h,
            Err(err) => {
                warnings.push(format!("{rel}: hash_failed:{err}"));
                continue;
            }
        };

        if let Some((text, kind)) = db::get_cached_extract(conn, project_id, &rel, &hash)? {
            total_chars = total_chars.saturating_add(text.len());
            extracted.push(ExtractedFile {
                path: rel,
                kind,
                content_hash: hash,
                text,
                cached: true,
                warnings: vec![],
            });
            if total_chars >= MAX_TOTAL_CHARS {
                warnings.push("content_truncated_by_budget".into());
                break;
            }
            continue;
        }

        let ext = ext_of(&path);
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let (kind, text, file_warnings) = if is_doc_ext(&ext) {
            match extract_doc_via_sidecar(sidecar, &path) {
                Ok((text, w)) => (ext.clone(), text, w),
                Err(err) => {
                    warnings.push(format!("{rel}: {err}"));
                    continue;
                }
            }
        } else if is_text_ext(&ext) || file_name == "makefile" || file_name == "cmakelists.txt" {
            match read_text_file(&path) {
                Ok(text) => ("text".into(), text, vec![]),
                Err(err) => {
                    warnings.push(format!("{rel}: {err}"));
                    continue;
                }
            }
        } else {
            continue;
        };

        db::upsert_file_extract(
            conn,
            project_id,
            &rel,
            &hash,
            &kind,
            &text,
            &file_warnings.join(";"),
        )?;

        total_chars = total_chars.saturating_add(text.len());
        extracted.push(ExtractedFile {
            path: rel,
            kind,
            content_hash: hash,
            text,
            cached: false,
            warnings: file_warnings,
        });
        if total_chars >= MAX_TOTAL_CHARS {
            warnings.push("content_truncated_by_budget".into());
            break;
        }
    }

    Ok((extracted, warnings))
}

pub fn build_corpus(files: &[ExtractedFile]) -> String {
    let mut out = String::new();
    for file in files {
        if file.text.trim().is_empty() {
            continue;
        }
        let chunk = format!("\n\n===== FILE: {} ({}) =====\n{}", file.path, file.kind, file.text);
        if out.len() + chunk.len() > MAX_TOTAL_CHARS {
            let remain = MAX_TOTAL_CHARS.saturating_sub(out.len());
            if remain > 64 {
                out.push_str(&chunk[..remain]);
                out.push_str("\n...[truncated]...");
            }
            break;
        }
        out.push_str(&chunk);
    }
    out
}

pub fn load_agent_prompt() -> String {
    let candidates = [
        PathBuf::from("prompts/agent_system.txt"),
        PathBuf::from("../prompts/agent_system.txt"),
    ];
    for path in candidates {
        if let Ok(text) = fs::read_to_string(&path) {
            return text;
        }
    }
    include_str!("../../prompts/agent_system.txt").to_string()
}
