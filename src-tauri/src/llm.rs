use crate::db::AppSettings;
use crate::graph::{extract_json_object, validate_graph};
use crate::ingest::load_agent_prompt;
use serde_json::{json, Value};
use std::time::Duration;

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(20))
        .pool_max_idle_per_host(2)
        .build()
        .map_err(|e| format!("http_client:{e}"))
}

pub fn chat_completion(settings: &AppSettings, system: &str, user: &str) -> Result<String, String> {
    if settings.api_key.trim().is_empty() {
        return Err("api_key_missing".into());
    }
    let base = settings.api_base_url.trim_end_matches('/');
    let url = format!("{base}/chat/completions");

    let body = json!({
        "model": settings.model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ]
    });

    let client = http_client()?;
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        // Avoid opaque decode failures from unexpected Content-Encoding.
        .header("Accept-Encoding", "identity")
        .json(&body)
        .send()
        .map_err(|e| format!("http_error:{e}"))?;

    let status = response.status();
    let bytes = response
        .bytes()
        .map_err(|e| format!("read_body:{e}"))?;
    let text = match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(err) => String::from_utf8_lossy(err.as_bytes()).to_string(),
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
        "Language appendix:\nWrite all human-readable string fields (meta.title, meta.summary, module names, node labels, explain, term text) in {lang}. Keep JSON keys, ids, and enum tokens (axis/kind/relation) unchanged."
    )
}

/// Built-in schema prompt + language appendix + optional user prompt (user wins conflicts).
fn resolve_system_prompt(settings: &AppSettings) -> String {
    let mut system = load_agent_prompt();
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

pub fn generate_graph(settings: &AppSettings, corpus: &str) -> Result<Value, String> {
    let system = resolve_system_prompt(settings);
    let user = format!(
        "Analyze the following extracts and produce the graph JSON.\nLanguage for human-readable fields: {}.\n\n{}",
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

/// Lightweight model availability check (does not require saving settings first).
pub fn test_connection(settings: &AppSettings) -> Result<String, String> {
    let reply = chat_completion(
        settings,
        "You are a connectivity probe. Reply with exactly: ok",
        "ping",
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

/// Explain a selected term with optional graph/document context.
pub fn explain_term(settings: &AppSettings, term: &str, context: &str) -> Result<String, String> {
    let term = term.trim();
    if term.is_empty() {
        return Err("term_empty".into());
    }
    let mut system = load_term_prompt();
    system.push_str("\n\n");
    system.push_str(&language_appendix(&settings.ui_locale));
    let user_prompt = settings.agent_prompt.trim();
    if !user_prompt.is_empty() {
        system.push_str(
            "\n\n--- User Agent Prompt (overrides conflicting instructions above) ---\n",
        );
        system.push_str(user_prompt);
    }
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
    chat_completion(settings, &system, &user)
}

fn parse_and_validate(raw: &str) -> Result<Value, String> {
    let value = extract_json_object(raw)?;
    validate_graph(&value)?;
    Ok(value)
}
