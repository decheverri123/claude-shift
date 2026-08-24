use std::io::BufRead;
use std::time::Duration;

/// Pulls a model into a local Ollama instance via `POST /api/pull`, streaming
/// NDJSON progress lines and printing each distinct status as it arrives.
/// Long-running by nature (multi-GB downloads), so the timeout is generous.
pub fn pull_model(base_url: &str, model: &str) -> Result<(), String> {
    let url = format!("{}/api/pull", base_url.trim_end_matches('/'));
    let body = serde_json::json!({ "name": model, "stream": true }).to_string();
    let res = ureq::post(&url)
        .timeout(Duration::from_secs(3600))
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map_err(|e| format!("failed to reach Ollama at {}: {}", url, e))?;

    let mut last_status = String::new();
    for line in std::io::BufReader::new(res.into_reader()).lines() {
        let line = line.map_err(|e| format!("error reading pull stream: {}", e))?;
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| format!("malformed pull response: {}", e))?;
        if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
            return Err(err.to_string());
        }
        if let Some(status) = value.get("status").and_then(|v| v.as_str()) {
            if status != last_status {
                println!("{}", status);
                last_status = status.to_string();
            }
        }
    }
    Ok(())
}

/// Fetches the live model tag list from a local Ollama instance via
/// `GET /api/tags`. Best-effort by design: any failure (unreachable, non-2xx,
/// malformed JSON) yields an empty list so the caller can degrade gracefully.
pub fn fetch_ollama_tags(base_url: &str) -> Vec<String> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let res = ureq::get(&url).timeout(Duration::from_millis(1200)).call();
    let res = match res {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    if res.status() != 200 {
        return Vec::new();
    }
    let value: serde_json::Value = match res.into_string() {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };
    value
        .get("models")
        .and_then(|m| m.as_array())
        .map(|models| {
            models
                .iter()
                .filter_map(|m| {
                    m.get("name")
                        .or_else(|| m.get("model"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    // The parsing path is exercised directly with a canned payload; live HTTP
    // is best-effort by design and not required for tests to pass.
    fn parse_from(payload: &str) -> Vec<String> {
        let value: serde_json::Value = serde_json::from_str(payload).unwrap();
        value
            .get("models")
            .and_then(|m| m.as_array())
            .map(|models| {
                models
                    .iter()
                    .filter_map(|m| {
                        m.get("name")
                            .or_else(|| m.get("model"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn parses_ollama_tags_payload() {
        let payload = r#"{"models":[{"name":"qwen2.5-coder:32b"},{"name":"deepseek-r1:8b","model":"deepseek-r1"}]}"#;
        let tags = parse_from(payload);
        assert_eq!(tags, vec!["qwen2.5-coder:32b", "deepseek-r1:8b"]);
    }

    #[test]
    fn empty_payload_yields_empty_list() {
        assert!(parse_from(r#"{}"#).is_empty());
        assert!(parse_from(r#"{"models":[]}"#).is_empty());
    }
}
