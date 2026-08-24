# ⚡ Claude Shift (`claude-shift` / `cshift`)

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust: 2021](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org)

**A fast, intuitive, zero-overhead CLI tool to instantly switch models and providers for Claude Code across 4 model tiers (Haiku, Medium, Large, Epic).**

[Install](#-install) •
[Quick Start](#-quick-start) •
[4-Tier Architecture](#-4-tier-model-architecture) •
[Capability Badges](#-model-capability-badges) •
[Ollama & Cloud Models](#-ollama-local--cloud-models) •
[Comparison Matrix](#-why-claude-shift-vs-alternatives) •
[Sponsor](#-support--tip-jar)

</div>

---

## 📦 Install

```bash
curl -fsSL https://raw.githubusercontent.com/decheverri123/claude-shift/main/install.sh | sh
```

Downloads the prebuilt `cshift` binary for your OS/arch to `~/.local/bin` — no Rust toolchain required. Covers macOS (Intel/Apple Silicon), Linux (x86_64/aarch64), and Windows under a bash shell (Git Bash, MSYS2, WSL).

### Verifying the download

The release pipeline publishes two artifacts for every tag:

- `cshift-<target>.tar.gz` — the binary
- `SHA256SUMS` — sha256 of every binary tarball

`install.sh` verifies the tarball's hash against `SHA256SUMS` before extracting. Set `CSHIFT_SKIP_VERIFY=1` to skip this check.

---

## 🛠️ Build & Run

Prefer to build from source, or on a platform the installer doesn't cover? `claude-shift` is a single Rust binary — no Node/npm required. Build and run it with Cargo:

```bash
# 1. Build (first run compiles the binary; ~20s)
cargo build --release
./target/release/cshift

# Or run directly via Cargo (recompiles as needed)
cargo run -- init          # scaffold ~/.config/cshift/config.json
cargo run -- --status      # show current settings
cargo run -- --list-presets
cargo run -- --preset "your-preset-name"
cargo run -- --reset       # revert to Anthropic defaults
cargo run                  # interactive wizard
```

First run: `cargo run -- init` creates `~/.config/cshift/config.json`, then edit it to define your own providers and presets. Presets are entirely user-defined — this tool ships no hardcoded model opinions.

---

## 🚀 Quick Start

Run the `cshift` binary (built via `cargo build --release`, or add `./target/release` to your `PATH`):

```bash
# 1. Open the interactive TUI Wizard
cshift

# 2. View current active 4-tier model configuration
cshift --status

# 3. View cross-provider model equivalence guide
cshift --guide

# 4. List presets you've defined in config.json
cshift --list-presets

# 5. Instant 1-second switch to a preset you've defined
cshift --preset my-preset-name

# 6. Open config.json in your default editor/viewer
cshift --open-config

# 7. One-command reset to official Anthropic defaults
cshift --reset
```

Presets aren't shipped out of the box — `cshift init` scaffolds providers only. Add your own presets to `config.json` before `--preset` has anything to switch to.

---

## 🎛️ 4-Tier Model Architecture

Claude Code divides tasks across distinct agent tiers. `claude-shift` gives you full independent control over all 4 tiers:

| Tier                | Claude Code Alias | Role & Workload                                                    | Recommended Models                                                              |
| :------------------ | :---------------- | :----------------------------------------------------------------- | :------------------------------------------------------------------------------ |
| 👑 **Epic Model**   | `opus` / `fable`  | Frontier autonomous agents, long-horizon planning & deep reasoning | **Claude Fable 5**, DeepSeek R1 (671B), Gemini 2.5 Pro, DeepSeek V4 Pro         |
| 🦁 **Large Model**  | `opus` / `sonnet` | Flagship coding driver, heavy architecture & hybrid thinking       | **Claude 3.7 Sonnet**, Claude 3 Opus, Qwen 2.5 Coder 32B, DeepSeek V4 Flash     |
| ⚡ **Medium Model** | `sonnet`          | Fast daily driver, refactoring, code fixes & standard edits        | **Claude 3.5 Sonnet**, DeepSeek V3 (Chat), Gemini 2.5 Flash, Qwen 2.5 Coder 14B |
| 🐇 **Haiku Model**  | `haiku`           | Subagents, file searching, rapid bash commands, git commits        | **Claude 3.5 Haiku**, Gemini 2.5 Flash Lite, Gemma 4, Qwen 2.5 Coder 7B/1.5B    |

---

## 🏷️ Model Capability Badges

Each model in `claude-shift` is tagged with visual capability indicators so you always know what tools and reasoning it supports:

| Badge | Capability               | Description                                                                                |
| :---: | :----------------------- | :----------------------------------------------------------------------------------------- |
|  🧠   | **Thinking / Reasoning** | Deep chain-of-thought logic (`r1`, `thinking`, `pro`, `fable`, `o3`)                       |
|  👁️   | **Vision / Multimodal**  | Screenshot and image comprehension (`vision`, `vl`, `claude`, `gemini`)                    |
|  🛠️   | **Tool & Agent Calling** | High-precision function calling & CLI automation (`coder`, `claude`, `deepseek`, `qwen`)   |
|  ⚡   | **Ultra Fast**           | High-throughput subagents & low-latency execution (`flash`, `haiku`, `lite`, `7b`, `1.5b`) |
|  🌐   | **Cloud Model**          | Remote cloud-accelerated inference (`:cloud`, `openrouter`, `anthropic/`)                  |
|  🔒   | **100% Local**           | Private, offline GPU/CPU inference on your own hardware                                    |

---

## 🦙 Ollama Local & Cloud Models

`claude-shift` dynamically auto-detects your installed Ollama models via `http://localhost:11434/api/tags` and maps them to the appropriate tier:

| Tier          | Ollama Cloud Equivalent (`:cloud`)           | Ollama Local Equivalent (GPU/Local)                         | Why They Match                                         |
| :------------ | :------------------------------------------- | :---------------------------------------------------------- | :----------------------------------------------------- |
| 👑 **Epic**   | `deepseek-v4-pro:cloud`, `deepseek-r1:cloud` | `deepseek-r1:70b`, `llama3.3:70b`                           | Deep CoT & 1M context for frontier reasoning           |
| 🦁 **Large**  | `deepseek-v4-flash:cloud`, `qwen3.5:cloud`   | `qwen2.5-coder:32b`, `deepseek-r1:32b`                      | Flagship syntax, AST reasoning, and test generation    |
| ⚡ **Medium** | `deepseek-v4-flash:cloud`, `qwen3.5:cloud`   | `qwen3.5:4b`, `qwen2.5-coder:14b`, `gemma4:e2b`             | High-speed daily coding with minimal resource cost     |
| 🐇 **Haiku**  | `gemma4:cloud`                               | `llama3.2:latest`, `qwen2.5-coder:7b`, `qwen2.5-coder:1.5b` | Instant sub-millisecond execution for background tools |

---

## 📊 Why Claude Shift? (vs Alternatives)

| Feature / Dimension                                      |    ⚡ **Claude Shift (`cshift`)**    |    🔀 **Claude Code Router (CCR)**    |          🤖 **Jan Desktop**           | 🖥️ **Built-in `/model` in Claude** | 📝 **Manual `.zshrc` / Scripts** |
| :------------------------------------------------------- | :----------------------------------: | :-----------------------------------: | :-----------------------------------: | :--------------------------------: | :------------------------------: |
| **Form Factor**                                          |        **Terminal CLI & TUI**        |      Daemon Middleware + Web UI       |         Heavy GUI Desktop App         |      In-session Slash Command      |         Raw Shell Config         |
| **Zero Daemon / Zero Overhead**                          |    ✅ **Yes** (Direct connection)    | ❌ No (Requires active proxy on port) | ❌ No (Requires running Electron app) |               ✅ Yes               |              ✅ Yes              |
| **4-Tier Model Control** _(Haiku, Medium, Large, Epic)_  |              ✅ **Yes**              |      ❌ No (Rule-based routing)       |       ⚠️ 3 Tiers (Jan UI only)        |   ❌ No (Single model override)    |   ❌ No (Manual JSON hacking)    |
| **Switch Providers** _(OpenRouter, Ollama, Gemini, Jan)_ |           ✅ **1 Second**            |       ✅ Yes (Via proxy config)       |        ⚠️ Primarily Jan Local         |       ❌ No (Anthropic only)       |        ⚠️ Manual exports         |
| **Live Ollama Model Auto-Detection**                     | ✅ **Yes** (`/api/tags` live picker) |           ❌ Manual config            |        ❌ No (Jan models only)        |               ❌ No                |              ❌ No               |
| **Capability Badges** _(🧠 👁️ 🛠️ ⚡ 🌐 🔒)_              |              ✅ **Yes**              |                 ❌ No                 |                 ❌ No                 |               ❌ No                |              ❌ No               |
| **One-Click `--reset` with Auto-Backup**                 |  ✅ **Yes** (`.cshift-backup-...`)   |      ⚠️ Manual shutdown & unlink      |        ⚠️ Reset button in GUI         |               ❌ No                |         ❌ No (Fragile)          |
| **Cross-Provider Equivalence Guide**                     |    ✅ **Yes** (`cshift --guide`)     |                 ❌ No                 |                 ❌ No                 |               ❌ No                |              ❌ No               |

---

## 🛡️ Safe & Non-Destructive

- **Native Settings Modification:** Safely edits `~/.claude/settings.json` (`env` variables and `modelOverrides`).
- **Automated Backups:** Creates timestamped backups (`settings.json.cshift-backup-<timestamp>`) before every change.
- **Factory Reset:** Running `cshift --reset` clears all custom redirects and cleanly restores standard Anthropic defaults.

---

## 💖 Support & Tip Jar

If `claude-shift` saved you money on API tokens or streamlined your workflow, consider supporting ongoing development:

- ☕ **Buy Me a Coffee:** [buymeacoffee.com/decheverri123](https://buymeacoffee.com/decheverri123)
- 💖 **GitHub Sponsors:** [github.com/sponsors/decheverri123](https://github.com/sponsors/decheverri123)

---

## 📄 License

MIT License © 2026 Danny (@decheverri123)
