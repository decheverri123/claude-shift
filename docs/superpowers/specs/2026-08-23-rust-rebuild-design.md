# claude-shift Rust rebuild — design spec

## Context

`claude-shift` (TS/Node, `src/*.ts`) is a CLI that switches Claude Code's active
model configuration by rewriting `~/.claude/settings.json`. Current version
ships built-in presets, a `--pull` Ollama wrapper, a `--guide` equivalence
table, and mistakenly writes `ANTHROPIC_DEFAULT_EPIC_MODEL`, an env var Claude
Code does not read (confirmed by extracting real var names from the installed
CLI binary: `ANTHROPIC_DEFAULT_{FABLE,OPUS,SONNET,HAIKU}_MODEL`).

This spec covers a ground-up rebuild in Rust with a narrower, more honest
scope: a fast, zero-daemon CLI where presets are entirely user-defined in a
config file, not hardcoded.

## Goals

- Single static binary, no runtime dependency (no Node/npm required).
- Sub-50ms cold start — startup latency is the actual selling point for a
  tool invoked on every model swap.
- User-owned `~/.config/cshift/config.json` defines providers and presets;
  the tool never ships opinions about which models are "good."
- Keep: interactive TUI wizard, Ollama live model auto-detect, `--reset` +
  timestamped backup, capability badges.
- Cut: built-in presets, `--pull`, `--guide`.
- No background process, no proxy, no daemon. `ANTHROPIC_BASE_URL` is
  global per Claude Code session, so a preset maps to exactly one provider.
  Cross-provider-per-tier routing would require a local reverse proxy
  (CCR's architecture) — explicitly out of scope; the entire pitch of this
  tool is that it does not run anything in the background.

## Non-goals

- Multi-provider routing within a single preset/session.
- Any HTTP server / proxy / daemon component.
- Bundled/curated preset list.
- `--pull` model downloading, `--guide` reference tables.

## Architecture

Single Rust binary, synchronous I/O only (task is one local file read/write
plus at most one localhost HTTP GET — no async runtime needed).

```
src/
  main.rs        CLI entry, clap parsing, command dispatch
  config.rs      Load/save/validate ~/.config/cshift/config.json, `init` scaffold
  settings.rs    Read/write ~/.claude/settings.json (env block, modelOverrides),
                 backup + reset
  ollama.rs      GET /api/tags, model list + tag parsing
  badges.rs      Capability badge inference from model name substrings
  wizard.rs      Interactive TUI (dialoguer): pick preset, apply
  apply.rs       Preset -> settings.json env vars; orchestrates config.rs + settings.rs
```

Two files, two different owners:

- `~/.config/cshift/config.json` — user-owned, hand-edited. This tool only
  reads it, except `cshift init` which scaffolds a starter file.
- `~/.claude/settings.json` — machine-generated. This tool writes it,
  atomically, always preceded by a timestamped backup.

## Crate stack

`clap` (CLI parsing) + `dialoguer` (interactive select menus — closest Rust
analog to the old `@clack/prompts` UX) + `ureq` (blocking HTTP, only used for
the Ollama `/api/tags` GET) + `serde` / `serde_json`. No async runtime
(`tokio`/`reqwest` rejected — the tool makes at most one HTTP call per run;
an async runtime would be dead weight).

## Data model — `~/.config/cshift/config.json`

```json
{
  "providers": {
    "ollama":     { "base_url": "http://localhost:11434", "auth_token": "ollama" },
    "lmstudio":   { "base_url": "http://localhost:1234",  "auth_token": "lm-studio" },
    "openrouter": { "base_url": "https://openrouter.ai/api", "auth_token": "$OPENROUTER_API_KEY" }
  },
  "presets": [
    {
      "name": "Ollama",
      "provider": "ollama",
      "epic":   "qwen2.5-coder:32b",
      "large":  "qwen2.5-coder:32b",
      "medium": "minimax2:cloud",
      "haiku":  "qwen2.5-coder:7b"
    }
  ]
}
```

Rules:

- A preset belongs to exactly one `provider`. All 4 tiers resolve against
  that provider's `base_url`/`auth_token`. No per-tier provider override.
- Tier values are bare model name strings understood by that provider (e.g.
  Ollama tags like `qwen2.5-coder:32b` — colons inside the value are fine
  since there's no `provider:model` split anymore).
- `auth_token` values starting with `$` are resolved from the environment
  at apply-time. Never written elsewhere on disk, never printed by any
  command.
- A preset need not set all 4 tiers. Unset tiers fall back to whatever's
  currently in `settings.json` (matches old `config.ts`'s
  `savedState?.models.epic || 'claude-fable-5'` merge pattern).
- `cshift init` scaffolds this file pre-filled with the `ollama`,
  `lmstudio`, and `openrouter` providers (verified live to expose real
  Anthropic Messages-compatible `/v1/messages` endpoints; see Provider
  verification below) and zero presets, so first run isn't a blank-file
  error.

### Provider verification (default providers only)

Confirmed live during design (2026-08-23):

- **Ollama** (`localhost:11434`) — native `/v1/messages`, returns
  Anthropic-shaped error JSON (`{"type":"error","error":{"type":"not_found_error",...}}`).
- **LM Studio** (`localhost:1234`) — documented native Anthropic-compatible
  Messages endpoint (`docs/developer/anthropic-compat/messages`).
- **OpenRouter** (`openrouter.ai/api`) — native `/v1/messages`, returned
  `401` (endpoint real, auth rejected) not `404`.

Providers investigated but excluded from the default set (still valid for a
user to hand-add): DeepSeek, Zhipu GLM, Moonshot Kimi, Novita all confirmed
to expose real `/anthropic` Messages routes (401/403, not 404). Groq,
Together AI, Fireworks, Cerebras confirmed **404** — OpenAI-format only, no
Anthropic-compatible endpoint; adding them would silently break. Google
Vertex / AWS Bedrock excluded — real Claude access exists but auth is
SigV4/GCP-signed, doesn't fit the `base_url` + bearer `auth_token` shape.

## CLI command surface

```
cshift                    interactive TUI wizard — pick a preset, apply it
cshift init                scaffold ~/.config/cshift/config.json with default providers
cshift --status            print current settings.json tier assignments
cshift --list-presets      print preset names from config.json, non-interactive
cshift --preset <name>     apply a preset directly, skip wizard
cshift --reset             revert settings.json to defaults, timestamped backup first
```

## Data flow — apply a preset (wizard or `--preset <name>`)

1. Load `~/.config/cshift/config.json`, find preset by name.
2. Look up preset's `provider` in the `providers` map; resolve `$ENV_VAR`
   auth token if present.
3. Validate the whole preset (provider exists, tier model strings present)
   before any write — no partial-apply state.
4. Backup current `~/.claude/settings.json` ->
   `settings.json.cshift-backup-<timestamp>`. If backup fails, abort —
   never leave the user without a rollback path.
5. Write `env.ANTHROPIC_BASE_URL`, `env.ANTHROPIC_AUTH_TOKEN`,
   `env.ANTHROPIC_DEFAULT_{FABLE,OPUS,SONNET,HAIKU}_MODEL`,
   `modelOverrides` — same shape as old `config.ts:applyShiftConfig`, minus
   the `EPIC` typo (uses `FABLE`).
6. Print confirmation with capability badges per tier.

If the preset's provider is `ollama`, refresh its tag list via
`GET /api/tags` before showing the wizard's model picker (mirrors old
`providers.ts:fetchLocalOllamaModels`). If unreachable, warn and fall back
to whatever's already in `config.json` — don't block the wizard.

## Error handling

- Missing/malformed `config.json` -> clear message pointing at `cshift init`,
  never a raw serde panic.
- Preset references unknown `provider` key -> fail validation before any
  write.
- Ollama unreachable during tag-refresh -> warn, degrade gracefully, don't
  block.
- Backup-then-write is mandatory and ordered; a failed backup aborts the
  write.

## Testing

- Unit: config parsing/validation, tier -> env-var name mapping, `$ENV_VAR`
  resolution, backup filename generation.
- Integration: temp `HOME`, run `init` -> `--preset` -> assert resulting
  `settings.json` content, then `--reset` -> assert it matches pre-apply
  state.
- Ollama tag-fetch: mock HTTP server; live fetch is best-effort by design,
  not required for tests to pass.

## Explicit decisions from design discussion

- 4 tiers kept: `epic` (-> `ANTHROPIC_DEFAULT_FABLE_MODEL`), `large` (->
  `OPUS`), `medium` (-> `SONNET`), `haiku` (-> `HAIKU`).
- Mixed-provider-per-tier presets rejected — would require a local proxy
  (CCR's architecture), which contradicts the zero-daemon pitch. One
  provider per preset only.
- Built-in presets, `--pull`, `--guide` all cut — config-file-driven only.
- Sync HTTP (`ureq`) over async (`reqwest`/`tokio`) — the tool's I/O
  profile doesn't justify an async runtime.
