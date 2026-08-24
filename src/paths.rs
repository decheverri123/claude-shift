use std::env;
use std::path::PathBuf;

/// The user's home directory, as seen by both `config_dir()` and `claude_dir()`.
pub fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME is not set"))
}
