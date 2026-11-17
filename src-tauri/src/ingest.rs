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
use std::sync::Mutex;

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
/// Safety cap on total extracted UTF-8 chars kept in memory before chunked LLM passes.
const MAX_EXTRACT_CHARS: usize = 30_000_000;
const MAX_FILES: usize = 120;

pub fn chunk_bytes_from_mb(mb: f64) -> usize {
    let mb = db::clamp_chunk_size_mb(mb);
    (mb * 1024.0 * 1024.0).round() as usize
}

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
    #[serde(default)]
    pub chunks_processed: usize,
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

fn path_is_ignored(rel: &str, name: &str, ignores: &[String]) -> bool {
    let rel_key = rel.replace('\\', "/").trim_matches('/').to_ascii_lowercase();
    let name_key = name.to_ascii_lowercase();
    for raw in ignores {
        let pattern = raw.replace('\\', "/").trim_matches('/').to_ascii_lowercase();
        if pattern.is_empty() {
            continue;
        }
        if !pattern.contains('/') {
            if name_key == pattern {
                return true;
            }
            continue;
        }
        if rel_key == pattern || rel_key.starts_with(&(pattern.clone() + "/")) {
            return true;
        }
    }
    false
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// True when `path` resolves inside `root` (both canonicalized). Security (S3).
fn path_within_root(root: &Path, path: &Path) -> bool {
    let Ok(root_c) = root.canonicalize() else {
        return false;
    };
    let Ok(path_c) = path.canonicalize() else {
        return false;
    };
    path_c.starts_with(&root_c)
}

pub fn collect_candidate_files(root: &Path, ignore_paths: &[String]) -> Result<Vec<PathBuf>, String> {
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
    walk(root, root, ignore_paths, &mut out)?;
    out.sort();
    if out.len() > MAX_FILES {
        out.truncate(MAX_FILES);
    }
    Ok(out)
}

fn walk(
    root: &Path,
    dir: &Path,
    ignore_paths: &[String],
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if out.len() >= MAX_FILES {
        return Ok(());
    }
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // Do not follow symlinks — prevents walking outside the project root (S3).
        if is_symlink(&path) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| format!("file_type {}: {e}", path.display()))?;
        if file_type.is_dir() {
            let rel = rel_path(root, &path);
            if path_is_ignored(&rel, &name, ignore_paths) {
                continue;
            }
            walk(root, &path, ignore_paths, out)?;
            continue;
        }
        if !file_type.is_file() {
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
        if !path_within_root(root, &path) {
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

fn with_db_conn<T>(
    db_conn: &Mutex<Connection>,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    // Security best practice (S2): hold the SQLite mutex only for short DB ops,
    // never across sidecar/disk extract which can take minutes.
    let conn = db_conn.lock().map_err(|e| e.to_string())?;
    f(&conn)
}

/// Extract project files. Cache lookup / upsert take short DB locks; sidecar and
/// filesystem I/O run unlocked so other IPC stays responsive during long books.
pub fn extract_project_files(
    db_conn: &Mutex<Connection>,
    sidecar: &SidecarState,
    project_id: i64,
    root: &Path,
    ignore_paths: &[String],
) -> Result<(Vec<ExtractedFile>, Vec<String>), String> {
    logutil::info(&format!(
        "extract_project_begin project_id={project_id} root={}",
        root.display()
    ));
    let files = collect_candidate_files(root, ignore_paths)?;
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

        let cached = with_db_conn(db_conn, |conn| {
            db::get_cached_extract(conn, project_id, &rel, &hash)
        })?;
        if let Some((text, kind)) = cached {
            total_chars = total_chars.saturating_add(text.len());
            extracted.push(ExtractedFile {
                path: rel,
                kind,
                content_hash: hash,
                text,
                cached: true,
                warnings: vec![],
            });
            if total_chars >= MAX_EXTRACT_CHARS {
                warnings.push("content_truncated_by_extract_cap".into());
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
        // I/O outside the DB lock (sidecar extract may take minutes).
        let (kind, text, file_warnings) = if is_doc_ext(&ext) {
            // Defense in depth: refuse docs that resolved outside the project root.
            if root.is_dir() && !path_within_root(root, &path) {
                warnings.push(format!("{rel}: path_outside_project_root"));
                continue;
            }
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

        with_db_conn(db_conn, |conn| {
            db::upsert_file_extract(
                conn,
                project_id,
                &rel,
                &hash,
                &kind,
                &text,
                &file_warnings.join(";"),
            )
        })?;

        total_chars = total_chars.saturating_add(text.len());
        extracted.push(ExtractedFile {
            path: rel,
            kind,
            content_hash: hash,
            text,
            cached: false,
            warnings: file_warnings,
        });
        if total_chars >= MAX_EXTRACT_CHARS {
            warnings.push("content_truncated_by_extract_cap".into());
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
        let chunk = format!(
            "\n\n===== FILE: {} ({}) =====\n{}",
            file.path, file.kind, file.text
        );
        if out.len() >= MAX_EXTRACT_CHARS {
            break;
        }
        if out.len() + chunk.len() > MAX_EXTRACT_CHARS {
            let remain = MAX_EXTRACT_CHARS.saturating_sub(out.len());
            if remain > 64 {
                let end = floor_char_boundary(&chunk, remain);
                out.push_str(&chunk[..end]);
                out.push_str("\n...[truncated]...");
            }
            break;
        }
        out.push_str(&chunk);
    }
    out
}

fn floor_char_boundary(text: &str, max_bytes: usize) -> usize {
    if max_bytes >= text.len() {
        return text.len();
    }
    let mut idx = max_bytes;
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// A heading this far from the next one is a real chapter, not a TOC line or a cross-reference.
const MIN_CHAPTER_BODY: usize = 8 * 1024;

/// Split on chapter headings when the source has them. `max_bytes` only splits a
/// chapter that is itself longer than the cap, on a paragraph or file break.
/// Sources without chapters stay one piece until they pass `max_bytes`.
pub fn split_corpus_chunks(corpus: &str, max_bytes: usize) -> Vec<String> {
    let max_bytes = max_bytes.max(64 * 1024);
    let ranges = chapter_ranges(corpus);
    if ranges.len() >= 2 {
        let mut chunks = Vec::new();
        for (start, end) in ranges {
            let piece = &corpus[start..end];
            if piece.trim().is_empty() {
                continue;
            }
            if piece.len() <= max_bytes {
                chunks.push(piece.to_string());
            } else {
                chunks.extend(split_by_size(piece, max_bytes));
            }
        }
        if chunks.len() >= 2 {
            logutil::info(&format!(
                "corpus_split mode=chapter chunks={} max_bytes={max_bytes}",
                chunks.len()
            ));
            return chunks;
        }
    }
    if corpus.len() <= max_bytes {
        return vec![corpus.to_string()];
    }
    let chunks = split_by_size(corpus, max_bytes);
    logutil::info(&format!(
        "corpus_split mode=size chunks={} max_bytes={max_bytes}",
        chunks.len()
    ));
    chunks
}

fn split_by_size(corpus: &str, max_bytes: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < corpus.len() {
        if corpus.len() - start <= max_bytes {
            chunks.push(corpus[start..].to_string());
            break;
        }
        let soft_end = floor_char_boundary(corpus, start + max_bytes);
        let window = &corpus[start..soft_end];
        let break_at = window
            .rfind("\n\n")
            .map(|i| start + i + 2)
            .filter(|i| *i > start + max_bytes / 4)
            .or_else(|| {
                window
                    .rfind('\n')
                    .map(|i| start + i + 1)
                    .filter(|i| *i > start + max_bytes / 4)
            })
            .unwrap_or(soft_end);
        let end = floor_char_boundary(corpus, break_at.max(start + 1));
        chunks.push(corpus[start..end].to_string());
        start = end;
    }
    chunks
}

/// Byte ranges covering `text`. Cuts sit on real chapter openings.
fn chapter_ranges(text: &str) -> Vec<(usize, usize)> {
    let heads = chapter_heading_bytes(text);
    if heads.len() < 2 {
        return Vec::new();
    }
    let mut cuts = Vec::new();
    for (index, at) in heads.iter().enumerate() {
        let next = heads.get(index + 1).copied().unwrap_or(text.len());
        if next.saturating_sub(*at) >= MIN_CHAPTER_BODY {
            cuts.push(*at);
        }
    }
    if cuts.len() < 2 {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    if cuts[0] > 0 {
        ranges.push((0, cuts[0]));
    }
    for index in 0..cuts.len() {
        let end = cuts.get(index + 1).copied().unwrap_or(text.len());
        if end > cuts[index] {
            ranges.push((cuts[index], end));
        }
    }
    ranges
}

fn chapter_heading_bytes(text: &str) -> Vec<usize> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut out = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        let at_line_start = index == 0 || chars[index - 1].1 == '\n';
        if let Some(consumed) = match_cn_chapter(&chars[index..]) {
            out.push(chars[index].0);
            index += consumed.max(1);
            continue;
        }
        if at_line_start {
            if let Some(consumed) = match_en_chapter(&chars[index..]) {
                out.push(chars[index].0);
                index += consumed.max(1);
                continue;
            }
        }
        index += 1;
    }
    out
}

fn match_cn_chapter(chars: &[(usize, char)]) -> Option<usize> {
    if chars.first().map(|item| item.1) != Some('第') {
        return None;
    }
    let mut index = 1usize;
    while index < chars.len() && chars[index].1 == ' ' {
        index += 1;
    }
    let num_at = index;
    if index < chars.len() && chars[index].1.is_ascii_digit() {
        while index < chars.len() && chars[index].1.is_ascii_digit() {
            index += 1;
        }
        let width = index - num_at;
        if width == 0 || width > 3 {
            return None;
        }
    } else {
        while index < chars.len() && is_cn_numeral(chars[index].1) {
            index += 1;
        }
        let width = index - num_at;
        if width == 0 || width > 6 {
            return None;
        }
    }
    while index < chars.len() && chars[index].1 == ' ' {
        index += 1;
    }
    if index >= chars.len() || chars[index].1 != '章' {
        return None;
    }
    index += 1;
    while index < chars.len() && chars[index].1 == ' ' {
        index += 1;
    }
    if index < chars.len() && chars[index].1 == '\u{3000}' {
        return Some(index + 1);
    }
    None
}

fn is_cn_numeral(ch: char) -> bool {
    matches!(
        ch,
        '零' | '〇' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '百' | '千'
    )
}

fn match_en_chapter(chars: &[(usize, char)]) -> Option<usize> {
    let mut index = 0usize;
    while index < chars.len() && (chars[index].1 == '#' || chars[index].1 == ' ') {
        index += 1;
    }
    let word = ['c', 'h', 'a', 'p', 't', 'e', 'r'];
    if index + word.len() >= chars.len() {
        return None;
    }
    for (offset, expected) in word.iter().enumerate() {
        if chars[index + offset].1.to_ascii_lowercase() != *expected {
            return None;
        }
    }
    index += word.len();
    if chars[index].1 != ' ' && chars[index].1 != '\t' {
        return None;
    }
    while index < chars.len() && (chars[index].1 == ' ' || chars[index].1 == '\t') {
        index += 1;
    }
    if index >= chars.len() || !chars[index].1.is_ascii_digit() {
        return None;
    }
    Some(index + 1)
}

fn load_prompt_file(name: &str, fallback: &str) -> String {
    let candidates = [
        PathBuf::from("prompts").join(name),
        PathBuf::from("../prompts").join(name),
    ];
    for path in candidates {
        if let Ok(text) = fs::read_to_string(&path) {
            return text;
        }
    }
    fallback.to_string()
}

pub fn load_agent_prompt() -> String {
    load_prompt_file(
        "agent_system.txt",
        include_str!("../../prompts/agent_system.txt"),
    )
}

pub fn load_book_prompt() -> String {
    load_prompt_file(
        "book_system.txt",
        include_str!("../../prompts/book_system.txt"),
    )
}

pub fn load_merge_prompt(book: bool) -> String {
    if book {
        load_prompt_file(
            "book_merge.txt",
            include_str!("../../prompts/book_merge.txt"),
        )
    } else {
        load_prompt_file(
            "graph_merge.txt",
            include_str!("../../prompts/graph_merge.txt"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::split_corpus_chunks;

    fn long_cjk(ch: char, count: usize) -> String {
        ch.to_string().repeat(count)
    }

    #[test]
    fn splits_real_chapters_and_keeps_cross_refs() {
        let toc = "目录 第 1 章\u{3000}甲要 第 2 章\u{3000}乙要\n";
        let chapter_1 = format!(
            "第 1 章\u{3000}甲要{}见第 2 章再谈{}",
            long_cjk('甲', 3000),
            long_cjk('甲', 200)
        );
        let chapter_2 = format!("第 2 章\u{3000}乙要{}", long_cjk('乙', 3000));
        let corpus = format!("{toc}{chapter_1}{chapter_2}");
        let chunks = split_corpus_chunks(&corpus, 8 * 1024 * 1024);
        assert_eq!(chunks.concat(), corpus);
        assert!(chunks.len() >= 2, "chunks={}", chunks.len());
        assert!(
            chunks.iter().any(|chunk| chunk.starts_with("第 2 章\u{3000}乙要")),
            "chapter 2 was not its own chunk"
        );
        let chapter_1_chunk = chunks
            .iter()
            .find(|chunk| chunk.contains("见第 2 章再谈"))
            .expect("chapter 1");
        assert!(!chapter_1_chunk.starts_with("第 2 章"));
        assert!(chapter_1_chunk.contains("见第 2 章再谈"));
    }

    #[test]
    fn splits_an_oversized_chapter_without_dropping_text() {
        let corpus = format!(
            "第 1 章\u{3000}甲{}\n\n段\n\n第 2 章\u{3000}乙{}",
            long_cjk('甲', 30_000),
            long_cjk('乙', 3000)
        );
        let chunks = split_corpus_chunks(&corpus, 64 * 1024);
        assert!(chunks.len() >= 3, "chunks={}", chunks.len());
        assert_eq!(chunks.concat(), corpus);
        assert!(chunks.last().unwrap().starts_with("第 2 章\u{3000}乙"));
    }

    #[test]
    fn ignore_matches_dir_name_and_relative_path() {
        let ignores = vec!["node_modules".into(), "src/generated".into()];
        assert!(super::path_is_ignored("node_modules", "node_modules", &ignores));
        assert!(super::path_is_ignored("pkg/Node_Modules", "Node_Modules", &ignores));
        assert!(super::path_is_ignored("src/generated", "generated", &ignores));
        assert!(super::path_is_ignored("src/generated/extra", "extra", &ignores));
        assert!(!super::path_is_ignored("src/app", "app", &ignores));
    }
}
