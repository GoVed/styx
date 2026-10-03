use serde_json::Value;

/// Extracts tool name and JSON arguments from an IFM formatted tool call markup block
pub fn parse_ifm_block(block: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let parts: Vec<&str> = if block.contains("<ifm|tool_call>") {
        block.split("<ifm|tool_call>").skip(1).collect()
    } else {
        vec![block]
    };

    for part in parts {
        let content = if let Some(end) = part.find("</ifm|tool_call>") {
            &part[..end]
        } else {
            part
        };

        let (name, body) = if let Some(tag_pos) = content.find('<') {
            (content[..tag_pos].trim(), &content[tag_pos..])
        } else {
            (content.trim(), "")
        };

        if name.is_empty() {
            continue;
        }

        let mut args_map = serde_json::Map::new();
        let mut cur = body;
        while let Some(k_start) = cur.find("<ifm|arg_key>") {
            let after_k = &cur[k_start + "<ifm|arg_key>".len()..];
            let Some(k_end) = after_k.find("</ifm|arg_key>") else { break };
            let key = after_k[..k_end].trim().to_string();
            let after_k_close = &after_k[k_end + "</ifm|arg_key>".len()..];

            if let Some(v_start) = after_k_close.find("<ifm|arg_value>") {
                let after_v = &after_k_close[v_start + "<ifm|arg_value>".len()..];
                let Some(v_end) = after_v.find("</ifm|arg_value>") else { break };
                let val_str = after_v[..v_end].trim();

                let val = if let Ok(parsed) = serde_json::from_str::<Value>(val_str) {
                    parsed
                } else {
                    Value::String(val_str.to_string())
                };

                args_map.insert(key, val);
                cur = &after_v[v_end + "</ifm|arg_value>".len()..];
            } else {
                cur = after_k_close;
            }
        }

        results.push((name.to_string(), Value::Object(args_map).to_string()));
    }

    results
}

/// Extracts tool name and JSON arguments from generic <function_calls> or <tool_call> blocks
pub fn parse_json_block(block: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let trimmed = block.trim();

    if let (Some(start), Some(end)) = (trimmed.find('['), trimmed.rfind(']')) {
        if end > start {
            if let Ok(val) = serde_json::from_str::<Value>(&trimmed[start..=end]) {
                if let Some(arr) = val.as_array() {
                    for item in arr {
                        if let Some((name, args)) = extract_tool_from_json(item) {
                            results.push((name, args));
                        }
                    }
                    if !results.is_empty() {
                        return results;
                    }
                }
            }
        }
    }

    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            if let Ok(val) = serde_json::from_str::<Value>(&trimmed[start..=end]) {
                if let Some((name, args)) = extract_tool_from_json(&val) {
                    results.push((name, args));
                    return results;
                }
            }
        }
    }

    results
}

fn extract_tool_from_json(val: &Value) -> Option<(String, String)> {
    let name = val
        .get("name")
        .or_else(|| val.get("function").and_then(|f| f.get("name")))
        .or_else(|| val.get("tool"))
        .and_then(|n| n.as_str())?;

    let args_val = val
        .get("arguments")
        .or_else(|| val.get("function").and_then(|f| f.get("arguments")))
        .or_else(|| val.get("parameters"));

    let args_str = match args_val {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => "{}".to_string(),
    };

    Some((name.to_string(), args_str))
}
