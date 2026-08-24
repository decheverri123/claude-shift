use std::env;
use std::fs;
use std::path::PathBuf;

use serde_json::{Map, Value};

/// Serializes tests that mutate the process-global env vars (`CSHIFT_CLAUDE_DIR`,
/// `HOME`). Cargo runs unit tests in one process across threads, so env-setting
/// tests must not overlap.
#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Claude Code env var names this tool writes.
pub const ENV_BASE_URL: &str = "ANTHROPIC_BASE_URL";
pub const ENV_AUTH_TOKEN: &str = "ANTHROPIC_AUTH_TOKEN";
pub const ENV_FABLE_MODEL: &str = "ANTHROPIC_DEFAULT_FABLE_MODEL";
pub const ENV_OPUS_MODEL: &str = "ANTHROPIC_DEFAULT_OPUS_MODEL";
pub const ENV_SONNET_MODEL: &str = "ANTHROPIC_DEFAULT_SONNET_MODEL";
pub const ENV_HAIKU_MODEL: &str = "ANTHROPIC_DEFAULT_HAIKU_MODEL";

/// Maps an internal tier name to the env var Claude Code actually reads.
pub fn tier_env_var(tier: &str) -> &'static str {
    match tier {
        "epic" => ENV_FABLE_MODEL,
        "large" => ENV_OPUS_MODEL,
        "medium" => ENV_SONNET_MODEL,
        "haiku" => ENV_HAIKU_MODEL,
        _ => ENV_FABLE_MODEL,
    }
}

/// Location of `~/.claude/settings.json`. Overridable for tests via
/// `CSHIFT_CLAUDE_DIR`.
pub fn claude_dir() -> PathBuf {
    PathBuf::from(env::var("CSHIFT_CLAUDE_DIR").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME is not set");
        format!("{}/.claude", home)
    }))
}

pub fn settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

fn ensure_claude_dir() -> Result<(), String> {
    let dir = claude_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create {}: {}", dir.display(), e))
}

/// Reads settings.json into a JSON object. Missing or malformed file yields an
/// empty object (this file is machine-generated and best-effort to read).
pub fn read_settings() -> Value {
    let path = settings_path();
    if !path.exists() {
        return Value::Object(Map::new());
    }
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| Value::Object(Map::new())),
        Err(_) => Value::Object(Map::new()),
    }
}

/// Writes settings.json atomically (write temp + rename) after creating the
/// `~/.claude` dir.
pub fn write_settings(settings: &Value) -> Result<(), String> {
    ensure_claude_dir()?;
    let path = settings_path();
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("failed to serialize settings: {}", e))?;
    let tmp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    fs::write(&tmp, raw).map_err(|e| format!("failed to write {}: {}", tmp.display(), e))?;
    fs::rename(&tmp, &path).map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
    Ok(())
}

/// Dedicated directory for cshift backups (`~/.claude/backups/cshift`).
pub fn backups_dir() -> PathBuf {
    claude_dir().join("backups").join("cshift")
}

/// Copies the current settings.json to `~/.claude/backups/cshift/settings.json.cshift-backup-<ts>`
/// before any mutation. Retains at most 5 recent backups to prevent disk clutter.
/// Returns None if there is nothing to back up (no file yet). Never leaves the user without a rollback path.
pub fn backup_settings() -> Result<Option<PathBuf>, String> {
    let path = settings_path();
    if !path.exists() {
        return Ok(None);
    }
    let bdir = backups_dir();
    fs::create_dir_all(&bdir)
        .map_err(|e| format!("failed to create backup dir {}: {}", bdir.display(), e))?;

    let ts = chrono_timestamp();
    let backup = bdir.join(format!("settings.json.cshift-backup-{}", ts));
    fs::copy(&path, &backup)
        .map_err(|e| format!("failed to back up {}: {}", path.display(), e))?;

    prune_old_backups(&bdir, 5);
    prune_legacy_backups(&claude_dir());

    Ok(Some(backup))
}

fn prune_old_backups(dir: &std::path::Path, max_keep: usize) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut backups: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("settings.json.cshift-backup-"))
                    .unwrap_or(false)
            })
            .collect();

        backups.sort();
        if backups.len() > max_keep {
            let to_remove = backups.len() - max_keep;
            for p in backups.iter().take(to_remove) {
                let _ = fs::remove_file(p);
            }
        }
    }
}

fn prune_legacy_backups(dir: &std::path::Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("settings.json.cshift-backup-") {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
    }
}

/// Generates the backup filename timestamp. Deterministic and unique enough for
/// backups; the exact wall-clock is not required.
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

/// Current resolved tier assignments, read from settings.json.
#[derive(Debug, Clone, Default)]
pub struct CurrentStatus {
    pub base_url: Option<String>,
    pub epic: Option<String>,
    pub large: Option<String>,
    pub medium: Option<String>,
    pub haiku: Option<String>,
    pub is_default: bool,
}

impl CurrentStatus {
    /// Reads the current env block + modelOverrides. `is_default` is true when
    /// no custom env or modelOverrides are present.
    pub fn read() -> CurrentStatus {
        let s = read_settings();
        let env = s.get("env").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let overrides = s
            .get("modelOverrides")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let base_url = env_get(&env, ENV_BASE_URL);
        let epic = env_get(&env, ENV_FABLE_MODEL);
        let large = env_get(&env, ENV_OPUS_MODEL);
        let medium = env_get(&env, ENV_SONNET_MODEL);
        let haiku = env_get(&env, ENV_HAIKU_MODEL);

        let is_default = base_url.is_none()
            && epic.is_none()
            && large.is_none()
            && medium.is_none()
            && haiku.is_none()
            && overrides.is_empty();

        CurrentStatus {
            base_url,
            epic,
            large,
            medium,
            haiku,
            is_default,
        }
    }
}

fn env_get(env: &Map<String, Value>, key: &str) -> Option<String> {
    env.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Reverts settings.json to Anthropic defaults: removes all the custom env
/// vars this tool writes, modelOverrides, and `model`. Always backs up first.
/// A failed backup aborts before any mutation.
pub fn reset_settings() -> Result<Option<PathBuf>, String> {
    let backup = backup_settings()?;
    let mut s = read_settings();

    if let Some(env) = s.get_mut("env").and_then(|v| v.as_object_mut()) {
        env.remove(ENV_BASE_URL);
        env.remove(ENV_AUTH_TOKEN);
        env.remove("ANTHROPIC_API_KEY");
        env.remove("ANTHROPIC_API_BASE_URL");
        env.remove("CLAUDE_AGENT_API_BASE_URL");
        env.remove("ANTHROPIC_MODEL");
        env.remove("CCR_CLAUDE_CODE_MCP_CONFIG");
        env.remove("CCR_CLAUDE_CODE_MODEL");
        env.remove("CODEXL_CLAUDE_CODE_MCP_CONFIG");
        env.remove("CODEXL_CLAUDE_CODE_MODEL");
        env.remove("CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY");
        env.remove(ENV_FABLE_MODEL);
        env.remove(ENV_OPUS_MODEL);
        env.remove(ENV_SONNET_MODEL);
        env.remove(ENV_HAIKU_MODEL);
        if env.is_empty() {
            s.as_object_mut().unwrap().remove("env");
        }
    }
    if let Some(obj) = s.as_object_mut() {
        obj.remove("modelOverrides");
        obj.remove("model");
        obj.remove("apiKeyHelper");
    }

    write_settings(&s)?;
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Returns the temp dir plus a held env mutex guard, so `CSHIFT_CLAUDE_DIR`
    /// stays stable while the test body runs.
    fn with_claude_dir() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = crate::settings::TEST_ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        env::set_var("CSHIFT_CLAUDE_DIR", dir.path());
        (dir, guard)
    }

    #[test]
    fn tier_env_var_mapping() {
        assert_eq!(tier_env_var("epic"), ENV_FABLE_MODEL);
        assert_eq!(tier_env_var("large"), ENV_OPUS_MODEL);
        assert_eq!(tier_env_var("medium"), ENV_SONNET_MODEL);
        assert_eq!(tier_env_var("haiku"), ENV_HAIKU_MODEL);
    }

    #[test]
    fn backup_writes_ordered_file() {
        let (_d, _guard) = with_claude_dir();
        let s = serde_json::json!({"env": {"ANTHROPIC_BASE_URL": "http://x"}});
        write_settings(&s).unwrap();
        let backup = backup_settings().unwrap().unwrap();
        assert!(backup.exists());
        assert!(backup
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("settings.json.cshift-backup-"));
    }

    #[test]
    fn backup_returns_none_when_no_file() {
        let (_d, _guard) = with_claude_dir();
        assert!(backup_settings().unwrap().is_none());
    }

    #[test]
    fn reset_clears_env_and_overrides() {
        let (_d, _guard) = with_claude_dir();
        let s = serde_json::json!({
            "env": {
                "ANTHROPIC_BASE_URL": "http://localhost:11434",
                "ANTHROPIC_AUTH_TOKEN": "ollama",
                "ANTHROPIC_DEFAULT_FABLE_MODEL": "qwen2.5-coder:32b"
            },
            "modelOverrides": {"claude-fable-5": "qwen2.5-coder:32b"},
            "model": "qwen2.5-coder:32b"
        });
        write_settings(&s).unwrap();

        let backup = reset_settings().unwrap();
        assert!(backup.is_some());

        let after = read_settings();
        assert_eq!(after.get("env"), None);
        assert_eq!(after.get("modelOverrides"), None);
        assert_eq!(after.get("model"), None);
        assert!(CurrentStatus::read().is_default);
    }
}
