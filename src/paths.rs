use std::env;
use std::path::PathBuf;

#[cfg(windows)]
fn home_from_env() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn home_from_env() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

/// The user's home directory. Prefers `$HOME` on Unix, `$USERPROFILE` on
/// Windows, falls back across both. Returns a default under `/tmp` only as a
/// last resort so we never panic — callers that need a real home must validate.
pub fn home_dir() -> PathBuf {
    home_from_env().unwrap_or_else(|| {
        // Last-resort fallback. Avoids panicking in containers or CI images
        // that omit `$HOME`. Callers (config_dir, claude_dir) treat this as
        // ephemeral; nothing is written here on a healthy machine.
        PathBuf::from("/tmp")
    })
}
