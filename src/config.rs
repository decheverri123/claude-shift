use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Default providers scaffolded by `cshift init`. All expose a real
/// Anthropic Messages-compatible `/v1/messages` endpoint (verified live).
pub const DEFAULT_PROVIDERS: &[(&str, &str, &str)] = &[
    ("ollama", "http://localhost:11434", "ollama"),
    ("lmstudio", "http://localhost:1234", "lm-studio"),
    ("openrouter", "https://openrouter.ai/api", "$OPENROUTER_API_KEY"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetTiers {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub haiku: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub epic: Option<String>,
    #[serde(default)]
    pub large: Option<String>,
    #[serde(default)]
    pub medium: Option<String>,
    #[serde(default)]
    pub haiku: Option<String>,
}

impl Preset {
    pub fn tiers(&self) -> PresetTiers {
        PresetTiers {
            epic: self.epic.clone(),
            large: self.large.clone(),
            medium: self.medium.clone(),
            haiku: self.haiku.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub presets: Vec<Preset>,
}

impl Config {
    /// Returns the default scaffolded config: known providers, zero presets.
    pub fn default_config() -> Self {
        let mut providers = HashMap::new();
        for (name, base_url, auth_token) in DEFAULT_PROVIDERS {
            providers.insert(
                name.to_string(),
                ProviderConfig {
                    base_url: base_url.to_string(),
                    auth_token: auth_token.to_string(),
                },
            );
        }
        Config {
            providers,
            presets: Vec::new(),
        }
    }

    pub fn find_preset(&self, name: &str) -> Option<&Preset> {
        self.presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }
}

/// Resolves `$ENV_VAR` auth-token values from the environment at apply-time.
/// A token starting with `$` must reference a set env var, otherwise it is an
/// error. Plain tokens (e.g. "ollama") pass through unchanged.
pub fn resolve_auth_token(token: &str) -> Result<String, String> {
    if let Some(rest) = token.strip_prefix('$') {
        env::var(rest).map_err(|_| format!("env var {} is not set", rest))
    } else {
        Ok(token.to_string())
    }
}

pub fn config_dir() -> PathBuf {
    PathBuf::from(env::var("CSHIFT_CONFIG_DIR").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME is not set");
        format!("{}/.config/cshift", home)
    }))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Loads and validates the user-owned config file. Errors are user-facing,
/// never raw serde panics.
pub fn load_config() -> Result<Config, String> {
    let path = config_path();
    if !path.exists() {
        return Err(format!(
            "config file not found at {}\nRun `cshift init` to scaffold it.",
            path.display()
        ));
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {}", path.display(), e))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("malformed config at {}: {}", path.display(), e))
}

/// Writes a config to disk, creating the parent dir if needed.
pub fn write_config(config: &Config) -> Result<PathBuf, String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
    }
    let raw = serde_json::to_string_pretty(config)
        .map_err(|e| format!("failed to serialize config: {}", e))?;
    fs::write(&path, raw).map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
    Ok(path)
}

/// Scaffolds a starter config file with default providers and zero presets.
pub fn init() -> Result<PathBuf, String> {
    let path = config_path();
    if path.exists() {
        return Err(format!("{} already exists; leaving it untouched", path.display()));
    }
    let path = write_config(&Config::default_config())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_env_auth_token() {
        // plain token passes through
        assert_eq!(resolve_auth_token("ollama").unwrap(), "ollama");
        // `$FOO` resolves from env
        env::set_var("CSHIFT_TEST_TOKEN", "sk-test");
        assert_eq!(resolve_auth_token("$CSHIFT_TEST_TOKEN").unwrap(), "sk-test");
        env::remove_var("CSHIFT_TEST_TOKEN");
    }

    #[test]
    fn missing_env_var_is_an_error() {
        env::remove_var("CSHIFT_DOES_NOT_EXIST");
        assert!(resolve_auth_token("$CSHIFT_DOES_NOT_EXIST").is_err());
    }

    #[test]
    fn default_config_has_known_providers_no_presets() {
        let c = Config::default_config();
        assert_eq!(c.providers.len(), 3);
        assert!(c.providers.contains_key("ollama"));
        assert!(c.providers.contains_key("lmstudio"));
        assert!(c.providers.contains_key("openrouter"));
        assert!(c.presets.is_empty());
    }

    #[test]
    fn preset_lookup_is_case_insensitive() {
        let c = Config {
            providers: HashMap::new(),
            presets: vec![Preset {
                name: "Ollama".into(),
                provider: "ollama".into(),
                epic: None,
                large: None,
                medium: None,
                haiku: None,
            }],
        };
        assert!(c.find_preset("ollama").is_some());
        assert!(c.find_preset("OLLAMA").is_some());
        assert!(c.find_preset("nope").is_none());
    }
}
