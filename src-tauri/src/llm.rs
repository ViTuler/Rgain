use crate::db::AppSettings;
use crate::graph::{extract_json_object, normalize_graph, validate_graph};
use crate::ingest::{self, load_agent_prompt, load_book_prompt, load_merge_prompt};
use crate::logutil;
use serde_json::{json, Value};
use std::time::Duration;

const GRAPH_TIMEOUT_SECS: u64 = 600;
const PROBE_TIMEOUT_SECS: u64 = 60;

fn error_chain(err: &dyn std::error::Error) -> String {
    let mut parts = vec![err.to_string()];
    let mut source = err.source();
    while let Some(next) = source {
        parts.push(next.to_string());
        source = next.source();
    }
    parts.join(" | ")
}

fn http_client(timeout_secs: u64) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(2)
        .build()
        .map_err(|e| format!("http_client:{}", error_chain(&e)))
}

fn apply_think(body: &mut Value, enabled: bool) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };
    obj.insert("enable_thinking".into(), json!(enabled));
    obj.insert(
        "thinking".into(),
        json!({ "type": if enabled { "enabled" } else { "disabled" } }),
    );
}

fn think_param_rejected(status: reqwest::StatusCode, text: &str) -> bool {
    if status.as_u16() != 400 && status.as_u16() != 422 {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    let mentions = lower.contains("enable_thinking")
        || lower.contains("thinking")
        || lower.contains("reasoning_effort");
    let rejected = lower.contains("unknown")
        || lower.contains("unrecognized")
        || lower.contains("unexpected")
        || lower.contains("unsupported")
        || lower.contains("not support")
        || lower.contains("invalid");
    mentions && rejected
}

fn post_chat(
    settings: &AppSettings,
    body: &Value,
    timeout_secs: u64,
) -> Result<(reqwest::StatusCode, String), String> {
    let base = settings.api_base_url.trim_end_matches('/');
    let url = format!("{base}/chat/completions");
    let client = http_client(timeout_secs)?;
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        // Identity avoids a truncated gzip stream being reported as a decode error.
        .header("Accept-Encoding", "identity")
        .json(body)
        .send()
        .map_err(|e| format!("http_error:{}", error_chain(&e)))?;

    let status = response.status();
    let bytes = response
        .bytes()
        .map_err(|e| format!("read_body:{}", error_chain(&e)))?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    Ok((status, text))
}

pub fn chat_completion(settings: &AppSettings, system: &str, user: &str) -> Result<String, String> {
    chat_completion_timeout(settings, system, user, GRAPH_TIMEOUT_SECS)
}

fn chat_completion_timeout(
    settings: &AppSettings,
    system: &str,
    user: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    if settings.api_key.trim().is_empty() {
        return Err("api_key_missing".into());
    }

    let mut body = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ]
    });
    apply_think(&mut body, settings.think_enabled);
    logutil::info(&format!(
        "llm_req model={} think={} timeout_s={timeout_secs}",
        settings.model, settings.think_enabled
    ));

    let (status, text) = post_chat(settings, &body, timeout_secs)?;
    let (status, text) = if think_param_rejected(status, &text) {
        logutil::warn("llm_think_unsupported_retry_without_flag");
        if let Some(obj) = body.as_object_mut() {
            obj.remove("enable_thinking");
            obj.remove("thinking");
        }
        post_chat(settings, &body, timeout_secs)?
    } else {
        (status, text)
    };

    if !status.is_success() {
        let snippet: String = text.chars().take(500).collect();
        return Err(format!("llm_http_{status}:{snippet}"));
    }
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        let snippet: String = text.chars().take(300).collect();
        format!("llm_json:{e}; body={snippet}")
    })?;
    value
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "llm_empty_content".into())
}

fn language_label(ui_locale: &str) -> &'static str {
    match crate::db::normalize_ui_locale(ui_locale).as_str() {
        "en" => "English",
        "ja" => "Japanese",
        "ko" => "Korean",
        _ => "Chinese (Simplified)",
    }
}

fn language_appendix(ui_locale: &str) -> String {
    let lang = language_label(ui_locale);
    format!(
        "Language appendix:\nWrite human-readable strings (meta.title, meta.summary, module names, node labels, explain, story fields, term text, open_items) in {lang}. Keep JSON keys, ids, and enum tokens (axis/kind/relation) unchanged."
    )
}

/// Built-in schema prompt + language appendix + optional user prompt (user wins conflicts).
fn is_book(source_kind: &str) -> bool {
    source_kind == "book"
}

fn resolve_system_prompt(settings: &AppSettings, source_kind: &str) -> String {
    let mut system = if is_book(source_kind) {
        load_book_prompt()
    } else {
        load_agent_prompt()
    };
    system.push_str("\n\n");
    system.push_str(&language_appendix(&settings.ui_locale));
    let user = settings.agent_prompt.trim();
    if !user.is_empty() {
        system.push_str(
            "\n\n--- User Agent Prompt (overrides conflicting instructions above) ---\n",
        );
        system.push_str(user);
    }
    system
}

/// One ingest session: split oversized corpus by `chunk_size_mb`, request partial graphs, then merge.
pub fn generate_graph_session(
    settings: &AppSettings,
    corpus: &str,
    source_kind: &str,
) -> Result<(Value, usize, Vec<String>), String> {
    let chunk_bytes = ingest::chunk_bytes_from_mb(settings.chunk_size_mb);
    let chunks = ingest::split_corpus_chunks(corpus, chunk_bytes);
    let total = chunks.len().max(1);
    let mut warnings = Vec::new();
    logutil::info(&format!(
        "llm_session_begin kind={source_kind} chunks={total} chunk_size_mb={} corpus_bytes={}",
        settings.chunk_size_mb,
        corpus.len()
    ));

    if total == 1 {
        let graph = generate_graph_for_chunk(settings, &chunks[0], 1, 1, source_kind)?;
        return Ok((graph, 1, warnings));
    }

    warnings.push(format!(
        "chunked_session:{}x{:.2}MB",
        total, settings.chunk_size_mb
    ));
    let mut partials = Vec::with_capacity(total);
    for (idx, chunk) in chunks.iter().enumerate() {
        let part = idx + 1;
        logutil::info(&format!(
            "llm_chunk_begin part={part}/{total} bytes={}",
            chunk.len()
        ));
        match generate_graph_for_chunk(settings, chunk, part, total, source_kind) {
            Ok(graph) => {
                logutil::info(&format!("llm_chunk_ok part={part}/{total}"));
                partials.push(graph);
            }
            Err(err) => {
                let msg = format!("chunk_{part}_failed:{err}");
                logutil::error(&msg);
                warnings.push(msg);
            }
        }
    }

    if partials.is_empty() {
        return Err("all_chunks_failed".into());
    }
    if partials.len() == 1 {
        warnings.push("chunk_merge_skipped_single_ok".into());
        return Ok((partials.remove(0), total, warnings));
    }

    logutil::info(&format!(
        "llm_merge_begin partials={} of {}",
        partials.len(),
        total
    ));
    let merged = merge_partial_graphs(settings, &partials, source_kind)?;
    logutil::info("llm_merge_ok");
    Ok((merged, total, warnings))
}

fn generate_graph_for_chunk(
    settings: &AppSettings,
    corpus: &str,
    part: usize,
    total: usize,
    source_kind: &str,
) -> Result<Value, String> {
    let system = resolve_system_prompt(settings, source_kind);
    let chunk_rule = if total > 1 {
        format!(
            "This is chunk {part}/{total} of a larger source. Chunks follow chapter boundaries when the source has them. Cover every detail in THIS chunk, including every chapter that appears here. Do not treat a chapter that is absent from this chunk as absent from the book. Prefix module and node ids with c{part}_. Do not cap the node count. Put unfinished steps in open_items.\n"
        )
    } else {
        String::new()
    };
    let user = format!(
        "{chunk_rule}Analyze the following extracts and produce the graph JSON.\nLanguage for human-readable fields: {}.\n\n{}",
        language_label(&settings.ui_locale),
        corpus
    );

    let first = chat_completion(settings, &system, &user)?;
    match parse_and_validate(&first) {
        Ok(graph) => Ok(graph),
        Err(err) => {
            let repair_user = format!(
                "Your previous output failed validation: {err}\nFix and return corrected JSON only.\n\nPrevious output:\n{first}"
            );
            let second = chat_completion(settings, &system, &repair_user)?;
            parse_and_validate(&second)
        }
    }
}

fn resolve_merge_system_prompt(settings: &AppSettings, source_kind: &str) -> String {
    let mut system = load_merge_prompt(is_book(source_kind));
    system.push_str("\n\n");
    system.push_str(&language_appendix(&settings.ui_locale));
    let user = settings.agent_prompt.trim();
    if !user.is_empty() {
        system.push_str(
            "\n\n--- User Agent Prompt (overrides conflicting instructions above) ---\n",
        );
        system.push_str(user);
    }
    system
}

fn merge_partial_graphs(
    settings: &AppSettings,
    partials: &[Value],
    source_kind: &str,
) -> Result<Value, String> {
    let system = resolve_merge_system_prompt(settings, source_kind);
    let mut user = format!(
        "Merge the following {} partial graphs into one final graph JSON.\nLanguage for human-readable fields: {}.\n\n",
        partials.len(),
        language_label(&settings.ui_locale)
    );
    for (i, graph) in partials.iter().enumerate() {
        user.push_str(&format!(
            "----- PARTIAL {} -----\n{}\n\n",
            i + 1,
            serde_json::to_string(graph).unwrap_or_else(|_| "{}".into())
        ));
    }

    let first = chat_completion(settings, &system, &user)?;
    match parse_and_validate(&first) {
        Ok(graph) => Ok(graph),
        Err(err) => {
            let repair_user = format!(
                "Your previous merge failed validation: {err}\nFix and return corrected JSON only.\n\nPrevious output:\n{first}"
            );
            let second = chat_completion(settings, &system, &repair_user)?;
            parse_and_validate(&second)
        }
    }
}

/// Lightweight model availability check (does not require saving settings first).
pub fn test_connection(settings: &AppSettings) -> Result<String, String> {
    let reply = chat_completion_timeout(
        settings,
        "You are a connectivity probe. Reply with exactly: ok",
        "ping",
        PROBE_TIMEOUT_SECS,
    )?;
    let snippet: String = reply.chars().take(120).collect();
    Ok(format!(
        "ok · model={} · reply={}",
        settings.model,
        snippet.trim()
    ))
}

fn load_term_prompt() -> String {
    let candidates = [
        std::path::PathBuf::from("prompts/term_explain.txt"),
        std::path::PathBuf::from("../prompts/term_explain.txt"),
    ];
    for path in candidates {
        if let Ok(text) = std::fs::read_to_string(&path) {
            return text;
        }
    }
    include_str!("../../prompts/term_explain.txt").to_string()
}

fn looks_like_graph(text: &str) -> bool {
    let trimmed = text.trim();
    (trimmed.contains("\"nodes\"") || trimmed.contains("\"modules\"")) && trimmed.contains("\"edges\"")
}

/// Explain a selected term with optional graph/document context.
pub fn explain_term(settings: &AppSettings, term: &str, context: &str) -> Result<String, String> {
    let term = term.trim();
    if term.is_empty() {
        return Err("term_empty".into());
    }
    let mut system = load_term_prompt();
    system.push_str(&format!(
        "\n\nWrite the explanation in {}.\n",
        language_label(&settings.ui_locale)
    ));
    let user_prompt = settings.agent_prompt.trim();
    if !user_prompt.is_empty() {
        system.push_str(
            "\n\n--- User notes (what to emphasize only; do not change the output shape) ---\n",
        );
        system.push_str(user_prompt);
    }
    system.push_str(
        "\n\nOutput lock: Markdown explanation of the selected term only. No JSON object, no graph, no modules, nodes, or edges. Markdown links to real http or https pages are allowed.\n",
    );
    let mut user = format!("Term:\n{term}\n");
    if !context.trim().is_empty() {
        user.push_str("\nContext:\n");
        user.push_str(context.trim());
        user.push('\n');
    }
    user.push_str(&format!(
        "\nExplain the term now in {}.",
        language_label(&settings.ui_locale)
    ));
    let first = chat_completion(settings, &system, &user)?;
    if !looks_like_graph(&first) {
        return Ok(first);
    }
    logutil::info("term_explain_rejected_graph");
    let retry = format!(
        "{user}\nYour previous reply was a graph JSON. Explain the term in Markdown prose. Do not return JSON."
    );
    chat_completion(settings, &system, &retry)
}

fn parse_and_validate(raw: &str) -> Result<Value, String> {
    let mut value = extract_json_object(raw)?;
    normalize_graph(&mut value);
    validate_graph(&value)?;
    Ok(value)
}
