/// Capability badge inference from model name substrings, plus formatting.
/// Mirrors the old `providers.ts:inferCapabilities` / `formatCapabilities`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Thinking,
    Vision,
    Tools,
    Fast,
    Cloud,
    Local,
}

impl Capability {
    pub fn icon(self) -> &'static str {
        match self {
            Capability::Thinking => "🧠",
            Capability::Vision => "👁️",
            Capability::Tools => "🛠️",
            Capability::Fast => "⚡",
            Capability::Cloud => "🌐",
            Capability::Local => "🔒",
        }
    }
}

/// Infers capability badges from a model name/slug. `is_cloud_hint` is used to
/// mark a model cloud-hosted even when its name doesn't carry a `:cloud` tag.
pub fn infer_capabilities(model_name: &str) -> Vec<Capability> {
    let m = model_name.to_lowercase();
    let mut caps: Vec<Capability> = Vec::new();

    let push = |c: Capability, caps: &mut Vec<Capability>| {
        if !caps.contains(&c) {
            caps.push(c);
        }
    };

    // Thinking / reasoning
    if ["r1", "thinking", "reason", "o1", "o3", "fable", "mythos", "pro"]
        .iter()
        .any(|k| m.contains(k))
        || m.contains("3.7-sonnet")
        || m.contains("sonnet-3.7")
    {
        push(Capability::Thinking, &mut caps);
    }

    // Vision / multimodal
    if ["vision", "gemini", "gemma", "claude-fable", "4o"]
        .iter()
        .any(|k| m.contains(k))
        || m.contains("vl")
        || m.contains("claude-3")
    {
        push(Capability::Vision, &mut caps);
    }

    // Tool use / function calling
    if ["coder", "claude", "qwen", "deepseek", "llama", "codestral", "jan-code", "gpt"]
        .iter()
        .any(|k| m.contains(k))
    {
        push(Capability::Tools, &mut caps);
    }

    // Fast / low-latency
    if ["flash", "haiku", "lite", "mini", "1.5b", "3b", "7b", "8b"]
        .iter()
        .any(|k| m.contains(k))
    {
        push(Capability::Fast, &mut caps);
    }

    // Cloud vs local
    if m.contains(":cloud")
        || m.starts_with("anthropic/")
        || m.starts_with("google/")
        || m.starts_with("meta-llama/")
    {
        push(Capability::Cloud, &mut caps);
    } else if !m.contains("http") && !m.contains('/') {
        push(Capability::Local, &mut caps);
    }

    caps
}

/// Renders capability badges as a spaced icon string, e.g. "🧠 👁️ 🛠️".
pub fn format_capabilities(caps: &[Capability]) -> String {
    caps.iter()
        .map(|c| c.icon())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_thinking_and_tools_for_reasoning_coders() {
        let caps = infer_capabilities("deepseek-r1:32b");
        assert!(caps.contains(&Capability::Thinking));
        assert!(caps.contains(&Capability::Tools));
        assert!(caps.contains(&Capability::Local));
    }

    #[test]
    fn cloud_tag_wins_over_local() {
        let caps = infer_capabilities("deepseek-v4-flash:cloud");
        assert!(caps.contains(&Capability::Cloud));
        assert!(!caps.contains(&Capability::Local));
        assert!(caps.contains(&Capability::Fast));
    }

    #[test]
    fn format_is_space_joined_icons() {
        let s = format_capabilities(&[Capability::Thinking, Capability::Tools]);
        assert_eq!(s, "🧠 🛠️");
    }
}
