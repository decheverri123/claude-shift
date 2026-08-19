# ⚡ Claude Shift (`claude-shift` / `cshift`)

A fast, intuitive, and beautifully styled CLI tool to quickly shift models and providers for **Claude Code** across **4 model tiers** (**Haiku**, **Medium**, **Large**, **Epic**) — inspired by Jan's model selector.

---

## 🚀 Quick Usage

Run globally with `claude-shift`, `cshift`, or `claudeshift`:

```bash
# Open the interactive TUI Wizard
cshift

# View active 4-tier model configuration
cshift --status

# View cross-provider model equivalence guide
cshift --guide

# Reset Claude Code to official Anthropic defaults
cshift --reset

# List all instant presets
cshift --list-presets

# Instant 1-second switch to a preset
cshift --preset openrouter-fable
cshift --preset openrouter-sonnet37
cshift --preset deepseek-r1
cshift --preset gemini-25
cshift --preset ollama-qwen
cshift --preset jan-code
```

---

## 🎛️ 4-Tier Model Architecture & Equivalents

Claude Code workflows divide tasks across 4 tiers of reasoning and speed. `claude-shift` gives you full independent control over each tier:

| Tier | Role | Anthropic Direct | OpenRouter | Google / Gemini | Ollama (Local) | Jan Local |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 👑 **Epic Model** | Frontier autonomous agents, multi-step planning | `claude-fable-5` | `anthropic/claude-fable-5` / `deepseek/deepseek-r1` | `google/gemini-2.5-pro` | `deepseek-r1:70b` / `llama3.3:70b` | `deepseek-r1-distill-qwen-14b` |
| 🦁 **Large Model** | Flagship coding, hybrid reasoning, architecture (Opus tier) | `claude-3-7-sonnet-latest` / `claude-3-opus` | `anthropic/claude-3.7-sonnet` / `qwen/qwen-2.5-coder-32b-instruct` | `google/gemini-2.5-pro` | `qwen2.5-coder:32b` / `deepseek-r1:32b` | `qwen2.5-coder-14b-instruct` |
| ⚡ **Medium Model** | Fast daily driver, refactoring, standard edits (Sonnet tier) | `claude-3-5-sonnet-latest` | `anthropic/claude-3.5-sonnet` / `deepseek/deepseek-chat` | `google/gemini-2.5-flash` | `qwen2.5-coder:14b` / `llama3.1:8b` | `janhq/Jan-code-4b` |
| 🐇 **Haiku Model** | Subagents, file searching, rapid bash commands (Haiku tier) | `claude-3-5-haiku-latest` | `anthropic/claude-3.5-haiku` / `google/gemini-2.5-flash-lite` | `google/gemini-2.5-flash-lite` | `qwen2.5-coder:7b` / `qwen2.5-coder:1.5b` | `janhq/Jan-code-4b` |

---

## 🌐 Supported Providers

- 🌐 **OpenRouter**: Access 200+ frontier models (Claude Fable 5, Claude 3.7 Sonnet, DeepSeek R1/V3, Gemini 2.5 Pro, Qwen 2.5 Coder 32B, Llama 3.3 70B).
- 🦙 **Ollama (Local)**: Dynamic model discovery via local Ollama API (`http://localhost:11434/api/tags`).
- ♊ **Google Gemini**: Gemini 2.5 Pro & Flash via OpenRouter / gateway with 1M context.
- 🤖 **Jan Desktop**: Direct connection to your running Jan desktop local server (`http://127.0.0.1:1337/v1`).
- ✨ **Anthropic Official**: Official Claude models.
- ⚙️ **Custom Gateway**: Any custom endpoint URL, auth token, and model IDs.

---

## 🛡️ Safe & Non-Destructive

- Non-destructive updates to `~/.claude/settings.json`.
- Automatic timestamped backup (`settings.json.cshift-backup-...`) before any modification.
- `cshift --reset` completely restores Claude Code to its pristine initial state.
