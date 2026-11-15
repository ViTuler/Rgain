use serde_json::Value;

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
        node_ids.insert(id.to_string());
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
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return Ok(v);
    }
    let start = trimmed
        .find('{')
        .ok_or_else(|| "no_json_object_start".to_string())?;
    let end = trimmed
        .rfind('}')
        .ok_or_else(|| "no_json_object_end".to_string())?;
    if end <= start {
        return Err("json_slice_invalid".into());
    }
    serde_json::from_str(&trimmed[start..=end]).map_err(|e| format!("json_parse:{e}"))
}
