//! Integration tests: run the compiled binary against a temp HOME and assert
//! the resulting `~/.claude/settings.json` content.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

fn bin() -> PathBuf {
    // Path to the integration test binary (env!("CARGO_BIN_EXE_<name>")).
    PathBuf::from(env!("CARGO_BIN_EXE_cshift"))
}

/// Serializes integration tests that share process-global env vars.
static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Returns a temp HOME plus a temp CLAUDE dir, keeping them separate so
/// `~/.config/cshift` and `~/.claude` can be pointed independently. Holds the
/// env lock for the whole test so env vars stay stable.
fn env_for_test() -> (TempDir, TempDir, std::sync::MutexGuard<'static, ()>) {
    let _guard = TEST_ENV_LOCK.lock().unwrap();
    let home = TempDir::new().unwrap();
    let claude = TempDir::new().unwrap();
    std::env::set_var("CSHIFT_CLAUDE_DIR", claude.path());
    std::env::set_var("CSHIFT_CONFIG_DIR", home.path().join(".config/cshift"));
    std::env::set_var("HOME", home.path());
    (home, claude, _guard)
}

fn run_cmd(args: &[&str]) -> std::process::Output {
    Command::new(bin()).args(args).output().unwrap()
}

fn write_config(dir: &Path, content: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("config.json"), content).unwrap();
}

#[test]
fn init_then_preset_then_reset_roundtrip() {
    let (home, claude, _guard) = env_for_test();
    let config_dir = home.path().join(".config/cshift");
    let settings_file = claude.path().join("settings.json");

    // 1. init scaffolds config with default providers.
    let out = run_cmd(&["init"]);
    assert!(
        out.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let cfg: Value =
        serde_json::from_str(&fs::read_to_string(config_dir.join("config.json")).unwrap()).unwrap();
    assert_eq!(
        cfg["providers"]["ollama"]["base_url"],
        "http://localhost:11434"
    );
    assert!(cfg["presets"].as_array().unwrap().is_empty());

    // 2. Add a preset, then apply it via --preset.
    write_config(
        &config_dir,
        r#"{
            "providers": {
                "ollama": {"base_url": "http://localhost:11434", "auth_token": "ollama"}
            },
            "presets": [{
                "name": "TestOllama",
                "provider": "ollama",
                "epic": "qwen2.5-coder:32b",
                "large": "qwen2.5-coder:32b",
                "medium": "minimax2:cloud",
                "haiku": "qwen2.5-coder:7b"
            }]
        }"#,
    );

    let out = run_cmd(&["--preset", "TestOllama"]);
    assert!(
        out.status.success(),
        "--preset failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let s: Value = serde_json::from_str(&fs::read_to_string(&settings_file).unwrap()).unwrap();
    let env = s["env"].as_object().unwrap();
    assert_eq!(env["ANTHROPIC_BASE_URL"], "http://localhost:11434");
    assert_eq!(env["ANTHROPIC_AUTH_TOKEN"], "ollama");
    assert_eq!(env["ANTHROPIC_DEFAULT_FABLE_MODEL"], "qwen2.5-coder:32b");
    assert_eq!(env["ANTHROPIC_DEFAULT_OPUS_MODEL"], "qwen2.5-coder:32b");
    assert_eq!(env["ANTHROPIC_DEFAULT_SONNET_MODEL"], "minimax2:cloud");
    assert_eq!(env["ANTHROPIC_DEFAULT_HAIKU_MODEL"], "qwen2.5-coder:7b");
    assert_eq!(s["modelOverrides"]["claude-fable-5"], "qwen2.5-coder:32b");

    // 3. Reset; settings.json should be reverted (env/modelOverrides cleared).
    let out = run_cmd(&["--reset"]);
    assert!(
        out.status.success(),
        "--reset failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let s: Value = serde_json::from_str(&fs::read_to_string(&settings_file).unwrap()).unwrap();
    assert_eq!(s.get("env"), None);
    assert_eq!(s.get("modelOverrides"), None);
    assert_eq!(s.get("model"), None);

    // 4. A backup was created during --preset.
    let backups: Vec<_> = fs::read_dir(claude.path().join("backups/cshift"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("settings.json.cshift-backup-")
        })
        .collect();
    assert_eq!(
        backups.len(),
        1,
        "expected one backup from the preset apply"
    );
}

#[test]
fn list_presets_prints_names() {
    let (home, _claude, _guard) = env_for_test();
    let config_dir = home.path().join(".config/cshift");
    write_config(
        &config_dir,
        r#"{
            "providers": {"ollama": {"base_url":"http://x","auth_token":"ollama"}},
            "presets": [{"name":"Alpha","provider":"ollama"},{"name":"Beta","provider":"ollama"}]
        }"#,
    );
    let out = run_cmd(&["--list-presets"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Alpha"));
    assert!(stdout.contains("Beta"));
}

#[test]
fn unknown_preset_fails_cleanly() {
    let (home, _claude, _guard) = env_for_test();
    let config_dir = home.path().join(".config/cshift");
    write_config(
        &config_dir,
        r#"{"providers":{"ollama":{"base_url":"http://x","auth_token":"ollama"}},"presets":[{"name":"Alpha","provider":"ollama"}]}"#,
    );
    let out = run_cmd(&["--preset", "Nope"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown preset"));
}

#[test]
fn missing_config_guides_to_init() {
    let (_home, _claude, _guard) = env_for_test();
    let out = run_cmd(&["--list-presets"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("cshift init"));
}
