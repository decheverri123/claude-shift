use serde_json::{json, Map, Value};

use crate::config::{self, Config, Preset};
use crate::settings;

/// Outcome of a successful apply, with enough info for the confirmation print.
#[derive(Debug)]
pub struct ApplyOutcome {
    pub preset_name: String,
    #[allow(dead_code)]
    pub provider_name: String,
    pub base_url: String,
    pub tiers: config::PresetTiers,
    pub backup: Option<std::path::PathBuf>,
}

/// Canonical Claude model names that `modelOverrides` maps onto the tier
/// models, mirroring the old `config.ts:applyShiftConfig`.
const OVERRIDE_KEYS: &[&str] = &[
    "claude-fable-5",
    "claude-mythos-5",
    "claude-3-opus-latest",
    "claude-opus-3-20240229",
    "claude-3-7-sonnet-latest",
    "claude-3-7-sonnet-20250219",
    "claude-3-5-sonnet-latest",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-latest",
    "claude-3-5-haiku-20241022",
];

/// Validates a preset before any write: provider must exist. Tier values may
/// be unset (they fall back to current settings).
fn validate_preset(config: &Config, preset: &Preset) -> Result<(), String> {
    match config.providers.get(&preset.provider) {
        Some(_) => Ok(()),
        None => Err(format!(
            "preset \"{}\" references unknown provider \"{}\"; add it to providers in {}",
            preset.name,
            preset.provider,
            config::config_path().display()
        )),
    }
}

/// Applies a preset to `~/.claude/settings.json`. Validates fully before any
/// write; backs up before writing; a failed backup aborts.
pub fn apply_preset(config: &Config, preset: &Preset) -> Result<ApplyOutcome, String> {
    validate_preset(config, preset)?;
    let provider = config
        .providers
        .get(&preset.provider)
        .expect("validated above");

    let auth_token = config::resolve_auth_token(&provider.auth_token)?;
    let base_url = provider.base_url.clone();

    // Backup-then-write is mandatory and ordered.
    let backup = settings::backup_settings()?;

    let tiers = preset.tiers();
    let mut s = settings::read_settings();
    let obj = s.as_object_mut().expect("settings are an object");

    let env = obj
        .entry("env")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("env is an object");

    env.insert(settings::ENV_BASE_URL.to_string(), json!(base_url));
    env.insert(settings::ENV_AUTH_TOKEN.to_string(), json!(auth_token));

    let set_tier = |tier: &str, value: Option<&str>, env: &mut Map<String, Value>| {
        if let Some(v) = value {
            env.insert(settings::tier_env_var(tier).to_string(), json!(v));
        }
    };
    set_tier("epic", tiers.epic.as_deref(), env);
    set_tier("large", tiers.large.as_deref(), env);
    set_tier("medium", tiers.medium.as_deref(), env);
    set_tier("haiku", tiers.haiku.as_deref(), env);

    // Build modelOverrides only from tiers that are actually set.
    let overrides: Map<String, Value> = OVERRIDE_KEYS
        .iter()
        .filter_map(|key| {
            let model = tier_for_key(key, &tiers)?;
            Some((key.to_string(), json!(model)))
        })
        .collect();
    if overrides.is_empty() {
        obj.remove("modelOverrides");
    } else {
        obj.insert("modelOverrides".to_string(), Value::Object(overrides));
    }

    if let Some(large) = &tiers.large {
        obj.insert("model".to_string(), json!(large));
    }

    settings::write_settings(&s)?;

    Ok(ApplyOutcome {
        preset_name: preset.name.clone(),
        provider_name: preset.provider.clone(),
        base_url,
        tiers,
        backup,
    })
}

/// Maps a canonical override key to the tier model it should resolve to. Only
/// returns Some when that tier is actually set in the preset.
fn tier_for_key<'a>(key: &str, tiers: &'a config::PresetTiers) -> Option<&'a str> {
    if key.starts_with("claude-fable") || key.starts_with("claude-mythos") {
        tiers.epic.as_deref()
    } else if key.contains("opus") || key.contains("3-7-sonnet") {
        tiers.large.as_deref()
    } else if key.contains("3-5-sonnet") {
        tiers.medium.as_deref()
    } else {
        tiers.haiku.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        serde_json::from_str(
            r#"{
                "providers": {
                    "ollama": {"base_url": "http://localhost:11434", "auth_token": "ollama"},
                    "openrouter": {"base_url": "https://openrouter.ai/api", "auth_token": "$FAKE_OR_KEY"}
                },
                "presets": [{
                    "name": "Ollama",
                    "provider": "ollama",
                    "epic": "qwen2.5-coder:32b",
                    "large": "qwen2.5-coder:32b",
                    "medium": "minimax2:cloud",
                    "haiku": "qwen2.5-coder:7b"
                }]
            }"#,
        )
        .unwrap()
    }

    fn with_claude_dir() -> (tempfile::TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = crate::settings::TEST_ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        std::env::set_var("CSHIFT_CLAUDE_DIR", dir.path());
        (dir, guard)
    }

    #[test]
    fn apply_writes_expected_env_and_overrides() {
        let (_d, _guard) = with_claude_dir();
        let config = test_config();
        let preset = config.find_preset("Ollama").unwrap();
        let outcome = apply_preset(&config, preset).unwrap();
        assert!(outcome.backup.is_none()); // no prior file to back up

        let s = settings::read_settings();
        let env = s["env"].as_object().unwrap();
        assert_eq!(env[settings::ENV_BASE_URL], "http://localhost:11434");
        assert_eq!(env[settings::ENV_AUTH_TOKEN], "ollama");
        assert_eq!(env[settings::ENV_FABLE_MODEL], "qwen2.5-coder:32b");
        assert_eq!(env[settings::ENV_OPUS_MODEL], "qwen2.5-coder:32b");
        assert_eq!(env[settings::ENV_SONNET_MODEL], "minimax2:cloud");
        assert_eq!(env[settings::ENV_HAIKU_MODEL], "qwen2.5-coder:7b");
        let overrides = s["modelOverrides"].as_object().unwrap();
        assert_eq!(overrides["claude-fable-5"], "qwen2.5-coder:32b");
        assert_eq!(s["model"], "qwen2.5-coder:32b");
    }

    #[test]
    fn backup_happens_before_write_when_dir_exists() {
        let (_d, _guard) = with_claude_dir();
        // First write a pre-existing settings file.
        let before = serde_json::json!({"env": {"ANTHROPIC_BASE_URL": "http://old"}});
        settings::write_settings(&before).unwrap();

        let config = test_config();
        let preset = config.find_preset("Ollama").unwrap();
        let outcome = apply_preset(&config, preset).unwrap();
        assert!(outcome.backup.is_some());

        // The backup preserves the pre-apply state.
        let backup_raw = std::fs::read_to_string(outcome.backup.unwrap()).unwrap();
        let backup: Value = serde_json::from_str(&backup_raw).unwrap();
        assert_eq!(backup["env"]["ANTHROPIC_BASE_URL"], "http://old");
    }

    #[test]
    fn unknown_provider_fails_validation_before_write() {
        let (_d, _guard) = with_claude_dir();
        let mut config = test_config();
        config.providers.remove("ollama");
        let preset = config.find_preset("Ollama").unwrap();
        let err = apply_preset(&config, preset).unwrap_err();
        assert!(err.contains("unknown provider"));
    }

    #[test]
    fn unset_tier_preserves_existing_value() {
        let (_d, _guard) = with_claude_dir();
        let existing = serde_json::json!({"env": {"ANTHROPIC_DEFAULT_HAIKU_MODEL": "keep-me"}});
        settings::write_settings(&existing).unwrap();

        let config = test_config();
        let mut preset = config.find_preset("Ollama").unwrap().clone();
        preset.haiku = None; // tier unset -> fall back
        let outcome = apply_preset(&config, &preset).unwrap();
        assert_eq!(outcome.tiers.haiku, None);

        let s = settings::read_settings();
        assert_eq!(s["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"], "keep-me");
        // Other tiers were overwritten by the preset.
        assert_eq!(s["env"]["ANTHROPIC_DEFAULT_FABLE_MODEL"], "qwen2.5-coder:32b");
    }

    #[test]
    fn env_var_auth_token_resolves_at_apply_time() {
        let (_d, _guard) = with_claude_dir();
        std::env::set_var("TEST_OR_KEY", "sk-resolved");
        let config = serde_json::from_str::<Config>(
            r#"{
                "providers": {"openrouter": {"base_url": "https://openrouter.ai/api", "auth_token": "$TEST_OR_KEY"}},
                "presets": [{"name": "OR", "provider": "openrouter", "large": "x"}]
            }"#,
        )
        .unwrap();
        let preset = config.find_preset("OR").unwrap();
        apply_preset(&config, preset).unwrap();
        let s = settings::read_settings();
        assert_eq!(s["env"]["ANTHROPIC_AUTH_TOKEN"], "sk-resolved");
    }

    #[test]
    fn unset_env_var_auth_fails_before_write() {
        let (_d, _guard) = with_claude_dir();
        std::env::remove_var("TEST_MISSING_KEY");
        let config = serde_json::from_str::<Config>(
            r#"{
                "providers": {"openrouter": {"base_url": "https://x", "auth_token": "$TEST_MISSING_KEY"}},
                "presets": [{"name":"OR","provider":"openrouter","epic":"a"}]
            }"#,
        )
        .unwrap();
        let preset = config.find_preset("OR").unwrap();
        assert!(apply_preset(&config, preset).is_err());
    }
}
