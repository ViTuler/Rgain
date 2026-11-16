use crate::logutil;
use serde_json::{json, Value};

/// Fill fields the model often omits. Structural gaps stay for validate_graph.
pub fn normalize_graph(value: &mut Value) {
    let Some(nodes) = value.get_mut("nodes").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for (index, node) in nodes.iter_mut().enumerate() {
        let Some(obj) = node.as_object_mut() else {
            continue;
        };
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        match obj.get("axis_order").cloned() {
            Some(v) if v.as_i64().is_some() => {}
            Some(v) if v.as_f64().is_some() => {
                let n = v.as_f64().unwrap_or(index as f64).round() as i64;
                obj.insert("axis_order".into(), json!(n));
            }
            Some(v) if v.as_str().and_then(|s| s.trim().parse::<i64>().ok()).is_some() => {
                let n = v.as_str().unwrap_or("").trim().parse::<i64>().unwrap_or(index as i64);
                obj.insert("axis_order".into(), json!(n));
            }
            _ => {
                obj.insert("axis_order".into(), json!(index as i64));
                if !id.is_empty() {
                    logutil::warn(&format!("graph_fill_axis_order id={id} order={index}"));
                }
            }
        }
    }
}

pub fn validate_graph(value: &Value) -> Result<(), String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "graph_must_be_object".to_string())?;

    let meta = obj
        .get("meta")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "missing_meta".to_string())?;
    for key in ["title", "axis", "summary"] {
        if meta.get(key).and_then(|v| v.as_str()).is_none() {
            return Err(format!("meta.{key}_required"));
        }
    }
    let axis = meta.get("axis").and_then(|v| v.as_str()).unwrap_or("");
    if !matches!(axis, "time" | "logic" | "story") {
        return Err("meta.axis_invalid".into());
    }

    let modules = obj
        .get("modules")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing_modules".to_string())?;
    if modules.is_empty() {
        return Err("modules_empty".into());
    }
    let mut module_ids = std::collections::HashSet::new();
    for m in modules {
        let id = m
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "module.id_required".to_string())?;
        if m.get("name").and_then(|v| v.as_str()).is_none() {
            return Err("module.name_required".into());
        }
        module_ids.insert(id.to_string());
    }

    let nodes = obj
        .get("nodes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing_nodes".to_string())?;
    if nodes.is_empty() {
        return Err("nodes_empty".into());
    }
    let mut node_ids = std::collections::HashSet::new();
    for n in nodes {
        let id = n
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "node.id_required".to_string())?;
        let kind = n.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        if !matches!(kind, "trunk" | "branch") {
            return Err(format!("node.kind_invalid:{id}"));
        }
        if n.get("label").and_then(|v| v.as_str()).is_none() {
            return Err(format!("node.label_required:{id}"));
        }
        if n.get("explain").and_then(|v| v.as_str()).is_none() {
            return Err(format!("node.explain_required:{id}"));
        }
        if n.get("axis_order").and_then(|v| v.as_i64()).is_none()
            && n.get("axis_order").and_then(|v| v.as_f64()).is_none()
        {
            return Err(format!("node.axis_order_required:{id}"));
        }
        let module_id = n
            .get("module_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("node.module_id_required:{id}"))?;
        if !module_ids.contains(module_id) {
            return Err(format!("node.module_id_unknown:{module_id}"));
        }
        if let Some(shared) = n.get("shared") {
            if !shared.is_boolean() {
                return Err(format!("node.shared_invalid:{id}"));
            }
        }
        if let Some(story) = n.get("story") {
            let story = story
                .as_object()
                .ok_or_else(|| format!("node.story_invalid:{id}"))?;
            for key in ["time", "place", "impact"] {
                if let Some(value) = story.get(key) {
                    if !value.is_string() {
                        return Err(format!("node.story.{key}_invalid:{id}"));
                    }
                }
            }
            if let Some(actors) = story.get("actors") {
                let ok = actors
                    .as_array()
                    .is_some_and(|arr| arr.iter().all(|a| a.is_string()));
                if !ok {
                    return Err(format!("node.story.actors_invalid:{id}"));
                }
            }
        }
        node_ids.insert(id.to_string());
    }

    for n in nodes {
        let id = n.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(parent_id) = n.get("parent_id").and_then(|v| v.as_str()) {
            if !parent_id.is_empty() && !node_ids.contains(parent_id) {
                return Err(format!("node.parent_unknown:{id}->{parent_id}"));
            }
        }
    }

    if let Some(edges) = obj.get("edges").and_then(|v| v.as_array()) {
        for e in edges {
            let from = e
                .get("from")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "edge.from_required".to_string())?;
            let to = e
                .get("to")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "edge.to_required".to_string())?;
            if !node_ids.contains(from) || !node_ids.contains(to) {
                return Err(format!("edge.endpoint_unknown:{from}->{to}"));
            }
        }
    } else {
        return Err("missing_edges".into());
    }

    Ok(())
}

pub fn extract_json_object(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    let fixed = repair_latex_json(trimmed);
    if let Some(value) = parse_json_value(&fixed) {
        if fixed != trimmed {
            logutil::info("json_repair_latex_escapes");
        }
        return Ok(value);
    }
    if fixed != trimmed {
        if let Some(value) = parse_json_value(trimmed) {
            return Ok(value);
        }
    }
    Err("json_parse".into())
}

fn parse_json_value(text: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return Some(value);
    }
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

/// Models write `\frac` inside JSON strings. `\f` `\b` `\t` `\r` are also JSON
/// escapes, and `\sum` is not a valid escape at all, so the command is lost
/// before the formula reaches KaTeX. Double those backslashes first.
fn repair_latex_json(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut in_string = false;
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if !in_string {
            out.push(ch);
            if ch == '"' {
                in_string = true;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            out.push(ch);
            in_string = false;
            i += 1;
            continue;
        }
        if ch != '\\' {
            out.push(ch);
            i += 1;
            continue;
        }
        if i + 1 >= chars.len() {
            out.push('\\');
            out.push('\\');
            break;
        }
        let next = chars[i + 1];
        match next {
            '"' | '\\' | '/' => {
                out.push('\\');
                out.push(next);
                i += 2;
            }
            'u' if is_json_unicode(&chars, i + 2) => {
                out.push('\\');
                out.push('u');
                for offset in 0..4 {
                    out.push(chars[i + 2 + offset]);
                }
                i += 6;
            }
            'n' => {
                let word = ascii_letters(&chars, i + 1);
                if is_latex_n_command(&word) {
                    out.push('\\');
                    out.push('\\');
                    i += 1;
                } else {
                    out.push('\\');
                    out.push('n');
                    i += 2;
                }
            }
            'b' | 'f' | 'r' | 't' => {
                let word = ascii_letters(&chars, i + 1);
                if word.len() > 1 {
                    out.push('\\');
                    out.push('\\');
                    i += 1;
                } else {
                    out.push('\\');
                    out.push(next);
                    i += 2;
                }
            }
            _ => {
                out.push('\\');
                out.push('\\');
                i += 1;
            }
        }
    }
    out
}

fn is_json_unicode(chars: &[char], start: usize) -> bool {
    if start + 4 > chars.len() {
        return false;
    }
    chars[start..start + 4]
        .iter()
        .all(|ch| ch.is_ascii_hexdigit())
}

fn ascii_letters(chars: &[char], start: usize) -> String {
    chars[start..]
        .iter()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect()
}

fn is_latex_n_command(word: &str) -> bool {
    matches!(
        word,
        "nabla"
            | "napprox"
            | "ncong"
            | "ne"
            | "neq"
            | "nexists"
            | "newline"
            | "ngeq"
            | "ngeqslant"
            | "ngtr"
            | "ni"
            | "nLeftarrow"
            | "nLeftrightarrow"
            | "nRightarrow"
            | "nVDash"
            | "nVdash"
            | "nleftarrow"
            | "nleftrightarrow"
            | "nleq"
            | "nleqslant"
            | "nless"
            | "nmid"
            | "not"
            | "notin"
            | "nparallel"
            | "nprec"
            | "npreceq"
            | "nrightarrow"
            | "nshortmid"
            | "nsim"
            | "nsubseteq"
            | "nsubseteqq"
            | "nsucc"
            | "nsucceq"
            | "nsupseteq"
            | "nsupseteqq"
            | "ntriangleleft"
            | "ntriangleright"
            | "nu"
            | "nvDash"
            | "nvdash"
    )
}

#[cfg(test)]
mod tests {
    use super::extract_json_object;

    #[test]
    fn keeps_latex_commands_in_json_strings() {
        let raw = r#"{"explain":"P(H\mid E)=\frac{P(H)\,P(E\mid H)}{\sum_i P(H_i)} \nabla \theta \times \beta \right"}"#;
        let value = extract_json_object(raw).expect("parse");
        let explain = value["explain"].as_str().unwrap();
        assert!(explain.contains(r"\mid"), "{explain}");
        assert!(explain.contains(r"\frac"), "{explain}");
        assert!(explain.contains(r"\sum"), "{explain}");
        assert!(explain.contains(r"\nabla"), "{explain}");
        assert!(explain.contains(r"\theta"), "{explain}");
        assert!(explain.contains(r"\times"), "{explain}");
        assert!(explain.contains(r"\beta"), "{explain}");
        assert!(explain.contains(r"\right"), "{explain}");
        assert!(!explain.contains('\u{000c}'), "{explain}");
    }

    #[test]
    fn keeps_already_escaped_latex_and_real_newlines() {
        let raw = r#"{"explain":"line\n next \\sum_i"}"#;
        let value = extract_json_object(raw).expect("parse");
        let explain = value["explain"].as_str().unwrap();
        assert_eq!(explain, "line\n next \\sum_i");
    }
}
