use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

use crate::apply::apply_preset;
use crate::badges::{format_capabilities, infer_capabilities};
use crate::config::{self, Config, Preset, ProviderConfig};
use crate::ollama;
use crate::settings::{self, CurrentStatus};
use crate::ui;

enum FlowResult {
    Done,
    Back,
}

/// Runs the interactive TUI wizard with the full action menu.
pub fn run_wizard(config: &Config) -> Result<(), String> {
    ui::print_banner();

    loop {
        // Reload current status and config on each loop
        let st = CurrentStatus::read();
        let latest_config = config::load_config().unwrap_or_else(|e| {
            eprintln!(
                "{} {} — using the last known-good config for this screen.",
                style("⚠").yellow().bold(),
                e
            );
            config.clone()
        });

        ui::print_status_card(&st, Some(&latest_config));
        println!();

        let theme = ColorfulTheme::default();

        let main_menu_options = [
            "⚡ Quick Presets",
            "🛠️  Configure 4 Model Tiers (Haiku, Medium, Large, Epic)",
            "📚 Model Equivalence & Capability Guide",
            "🔄 Reset to Claude Code Defaults",
            "🚪 Exit",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(&main_menu_options)
            .default(0)
            .interact_opt()
            .map_err(|e| format!("wizard error: {}", e))?;

        let index = match selection {
            Some(i) => i,
            None => {
                println!("{}", style("Goodbye!").dim());
                return Ok(());
            }
        };

        match index {
            0 => {
                if let FlowResult::Done = handle_presets_flow(&latest_config)? {
                    return Ok(());
                }
            }
            1 => {
                if let FlowResult::Done = handle_configure_flow(&latest_config)? {
                    return Ok(());
                }
            }
            2 => {
                handle_guide_flow()?;
            }
            3 => {
                if let FlowResult::Done = handle_reset_flow()? {
                    return Ok(());
                }
            }
            4 => {
                println!("{}", style("Goodbye!").dim());
                return Ok(());
            }
            _ => return Ok(()),
        }
    }
}

/// Displays the Model Equivalence Guide with an interactive back prompt.
fn handle_guide_flow() -> Result<(), String> {
    ui::print_equivalence_guide();
    let theme = ColorfulTheme::default();
    let options = ["⬅️  Back to Main Menu"];
    let _ = Select::with_theme(&theme)
        .items(&options)
        .default(0)
        .interact_opt();
    println!();
    Ok(())
}

/// Handles the "Quick Presets" flow.
fn handle_presets_flow(config: &Config) -> Result<FlowResult, String> {
    let theme = ColorfulTheme::default();

    let mut items: Vec<String> = config
        .presets
        .iter()
        .map(|p| {
            let tiers = p.tiers();
            let mut summary = Vec::new();
            if let Some(m) = &tiers.epic {
                summary.push(format!("Epic: {}", m));
            }
            if let Some(m) = &tiers.large {
                summary.push(format!("Large: {}", m));
            }
            if summary.is_empty() {
                format!("{:<20} ({})", p.name, p.provider)
            } else {
                format!("{:<20} [{}] ({})", p.name, summary.join(", "), p.provider)
            }
        })
        .collect();

    let create_idx = items.len();
    items.push("➕ Create / Edit Custom Presets (Open config.json)".to_string());
    let back_idx = items.len();
    items.push("⬅️  Back".to_string());

    let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

    let selection = Select::with_theme(&theme)
        .with_prompt("Select a quick configuration preset (4-tier)")
        .items(&item_refs)
        .default(0)
        .interact_opt()
        .map_err(|e| format!("wizard error: {}", e))?;

    let index = match selection {
        Some(i) => i,
        None => return Ok(FlowResult::Back),
    };

    if index == back_idx {
        return Ok(FlowResult::Back);
    }

    if index == create_idx {
        open_or_show_config_file();
        return Ok(FlowResult::Back);
    }

    let preset = &config.presets[index];

    if preset.provider == "ollama" {
        refresh_ollama_warning(config);
    }

    let outcome = apply_preset(config, preset).map_err(|e| {
        eprintln!("failed to apply preset: {}", e);
        e
    })?;

    println!();
    ui::print_success_shift(&outcome);
    Ok(FlowResult::Done)
}

/// Opens config.json in the user's default editor/viewer, or prints the path if unable.
pub fn open_or_show_config_file() {
    let path = config::config_path();
    if !path.exists() {
        if let Ok(p) = config::init() {
            println!("  {} Created starter config at {}", style("✔").green(), p.display());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }

    println!(
        "\n  {} {}\n  {}\n",
        style("Settings file:").bold().cyan(),
        style(path.display()).white().underlined(),
        style("Add or edit custom presets in the JSON file above, then select them in cshift.").dim()
    );
}

/// Handles the "Configure 4 Model Tiers" interactive flow.
fn handle_configure_flow(config: &Config) -> Result<FlowResult, String> {
    let theme = ColorfulTheme::default();

    let providers = [
        ("ollama", "🌐 Ollama (Local & Cloud)"),
        ("openrouter", "🌐 OpenRouter (Cloud Gateway)"),
        ("lmstudio", "🔒 LM Studio (Local)"),
        ("custom", "⚙️  Custom Compatible Endpoint"),
    ];

    let mut provider_labels: Vec<String> = providers.iter().map(|(_, l)| l.to_string()).collect();
    let back_idx = provider_labels.len();
    provider_labels.push("⬅️  Back".to_string());

    let label_refs: Vec<&str> = provider_labels.iter().map(|s| s.as_str()).collect();

    let selection = Select::with_theme(&theme)
        .with_prompt("Select an AI provider / backend")
        .items(&label_refs)
        .default(0)
        .interact_opt()
        .map_err(|e| format!("wizard error: {}", e))?;

    let provider_idx = match selection {
        Some(i) => i,
        None => return Ok(FlowResult::Back),
    };

    if provider_idx == back_idx {
        return Ok(FlowResult::Back);
    }

    let provider_id = providers[provider_idx].0;

    // Dynamic Ollama model detection
    let mut dynamic_ollama: Vec<String> = Vec::new();
    if provider_id == "ollama" {
        let base_url = config
            .providers
            .get("ollama")
            .map(|p| p.base_url.as_str())
            .unwrap_or("http://localhost:11434");
        dynamic_ollama = ollama::fetch_ollama_tags(base_url);
        if !dynamic_ollama.is_empty() {
            println!(
                "{}",
                style(format!("✔ Detected {} local Ollama models!", dynamic_ollama.len())).green()
            );
        }
    }

    let (default_epic, default_large, default_medium, default_haiku) = match provider_id {
        "ollama" => (
            "deepseek-v4-pro:cloud",
            "minimax-m3:cloud",
            "qwen3.5:cloud",
            "gemma4:cloud",
        ),
        "openrouter" => (
            "deepseek/deepseek-r1",
            "google/gemini-2.5-pro",
            "google/gemini-2.5-flash",
            "google/gemini-2.5-flash-lite",
        ),
        "lmstudio" => (
            "qwen2.5-coder-32b-instruct",
            "qwen2.5-coder-32b-instruct",
            "qwen2.5-coder-14b-instruct",
            "qwen2.5-coder-7b-instruct",
        ),
        _ => (
            "claude-fable-5",
            "claude-3-7-sonnet",
            "claude-3-5-sonnet",
            "claude-3-5-haiku",
        ),
    };

    // 1. Epic Model Selection
    let epic_model = match prompt_model_tier(
        &theme,
        provider_id,
        "epic",
        "👑 1/4 Select EPIC Model (Frontier / Autonomous Agents / Mythos Tier)",
        default_epic,
        &dynamic_ollama,
    )? {
        Some(m) => m,
        None => return Ok(FlowResult::Back),
    };

    // 2. Large Model Selection
    let large_model = match prompt_model_tier(
        &theme,
        provider_id,
        "large",
        "🦁 2/4 Select LARGE Model (Opus Tier / Flagship Coding & Reasoning)",
        default_large,
        &dynamic_ollama,
    )? {
        Some(m) => m,
        None => return Ok(FlowResult::Back),
    };

    // 3. Medium Model Selection
    let medium_model = match prompt_model_tier(
        &theme,
        provider_id,
        "medium",
        "⚡ 3/4 Select MEDIUM Model (Sonnet Tier / Daily Coding Driver)",
        default_medium,
        &dynamic_ollama,
    )? {
        Some(m) => m,
        None => return Ok(FlowResult::Back),
    };

    // 4. Haiku Model Selection
    let haiku_model = match prompt_model_tier(
        &theme,
        provider_id,
        "haiku",
        "🐇 4/4 Select HAIKU Model (Haiku Tier / Subagents & Background Tasks)",
        default_haiku,
        &dynamic_ollama,
    )? {
        Some(m) => m,
        None => return Ok(FlowResult::Back),
    };

    // 5. Base URL
    let default_url = match provider_id {
        "ollama" => config
            .providers
            .get("ollama")
            .map(|p| p.base_url.as_str())
            .unwrap_or("http://localhost:11434"),
        "openrouter" => config
            .providers
            .get("openrouter")
            .map(|p| p.base_url.as_str())
            .unwrap_or("https://openrouter.ai/api"),
        "lmstudio" => config
            .providers
            .get("lmstudio")
            .map(|p| p.base_url.as_str())
            .unwrap_or("http://localhost:1234"),
        _ => "http://localhost:11434",
    };

    let base_url: String = Input::with_theme(&theme)
        .with_prompt("Base URL (Endpoint)")
        .default(default_url.to_string())
        .interact_text()
        .map_err(|e| format!("input error: {}", e))?;

    // 6. Auth Token
    let default_auth = match provider_id {
        "ollama" => "ollama",
        "lmstudio" => "lm-studio",
        "openrouter" => "$OPENROUTER_API_KEY",
        _ => "custom",
    };

    let auth_token: String = if provider_id == "openrouter" {
        if std::env::var("OPENROUTER_API_KEY").is_ok() {
            let use_env = Confirm::with_theme(&theme)
                .with_prompt("Found OPENROUTER_API_KEY in environment. Use $OPENROUTER_API_KEY?")
                .default(true)
                .interact()
                .unwrap_or(true);
            if use_env {
                "$OPENROUTER_API_KEY".to_string()
            } else {
                Password::with_theme(&theme)
                    .with_prompt("Enter OpenRouter API Key")
                    .interact()
                    .map_err(|e| format!("password input error: {}", e))?
            }
        } else {
            Password::with_theme(&theme)
                .with_prompt("Enter OpenRouter API Key")
                .interact()
                .map_err(|e| format!("password input error: {}", e))?
        }
    } else {
        Input::with_theme(&theme)
            .with_prompt("Auth Token")
            .default(default_auth.to_string())
            .interact_text()
            .map_err(|e| format!("input error: {}", e))?
    };

    let mut working_config = config.clone();
    working_config.providers.insert(
        provider_id.to_string(),
        ProviderConfig {
            base_url: base_url.clone(),
            auth_token: auth_token.clone(),
        },
    );

    let temp_preset = Preset {
        name: format!("Custom ({})", provider_id),
        provider: provider_id.to_string(),
        epic: Some(epic_model),
        large: Some(large_model),
        medium: Some(medium_model),
        haiku: Some(haiku_model),
    };

    let outcome = apply_preset(&working_config, &temp_preset).map_err(|e| {
        eprintln!("failed to apply configuration: {}", e);
        e
    })?;

    println!();
    ui::print_success_shift(&outcome);
    Ok(FlowResult::Done)
}

fn prompt_model_tier(
    theme: &ColorfulTheme,
    provider_id: &str,
    tier: &str,
    prompt: &str,
    default_model: &str,
    dynamic_models: &[String],
) -> Result<Option<String>, String> {
    let mut raw_candidates: Vec<String> = Vec::new();

    // 1. Add dynamic models first (e.g. from local Ollama)
    for m in dynamic_models {
        if !raw_candidates.contains(m) {
            raw_candidates.push(m.clone());
        }
    }

    // 2. Add provider-specific suggestions
    let suggestions: &[&str] = match (provider_id, tier) {
        ("openrouter", "epic") => &[
            "deepseek/deepseek-r1",
            "z-ai/glm-5",
            "thudm/glm-4-plus",
            "deepseek/deepseek-r1-0528",
            "deepseek/deepseek-r1-distill-llama-70b",
        ],
        ("openrouter", "large") => &[
            "minimax/minimax-01",
            "google/gemini-2.5-pro",
            "z-ai/glm-4.7",
            "qwen/qwen-2.5-coder-32b-instruct",
            "qwen/qwen3-coder",
            "meta-llama/llama-3.3-70b-instruct",
        ],
        ("openrouter", "medium") => &[
            "deepseek/deepseek-chat",
            "google/gemini-2.5-flash",
            "z-ai/glm-4.7-flash",
            "qwen/qwen-2.5-coder-14b-instruct",
            "minimax/minimax-m2.1",
        ],
        ("openrouter", "haiku") => &[
            "google/gemini-2.5-flash-lite",
            "qwen/qwen-2.5-coder-7b-instruct",
            "meta-llama/llama-3.1-8b-instruct",
            "google/gemini-2.5-flash-lite:batch",
        ],
        ("ollama", "epic") => &[
            "deepseek-v4-pro:cloud",
            "deepseek-r1:cloud",
            "deepseek-r1:70b",
            "llama3.3:70b",
            "deepseek-r1:32b",
            "deepseek-r1:14b",
            "deepseek-r1:8b",
        ],
        ("ollama", "large") => &[
            "minimax-m3:cloud",
            "deepseek-v4-flash:cloud",
            "qwen2.5-coder:32b",
            "qwen2.5:32b",
            "codellama:34b",
        ],
        ("ollama", "medium") => &[
            "qwen3.5:cloud",
            "deepseek-v4-flash:cloud",
            "qwen2.5-coder:14b",
            "llama3.1:8b",
            "mistral-nemo:12b",
            "qwen2.5-coder:7b",
        ],
        ("ollama", "haiku") => &[
            "gemma4:cloud",
            "qwen2.5-coder:7b",
            "qwen2.5-coder:1.5b",
            "llama3.2:3b",
            "llama3.2:1b",
            "qwen2.5:0.5b",
        ],
        ("lmstudio", "epic") => &[
            "deepseek-r1-distill-qwen-32b",
            "deepseek-r1-distill-llama-70b",
            "deepseek-r1:70b",
        ],
        ("lmstudio", "large") => &[
            "qwen2.5-coder-32b-instruct",
            "deepseek-r1-distill-qwen-32b",
            "llama-3.3-70b-instruct",
        ],
        ("lmstudio", "medium") => &[
            "qwen2.5-coder-14b-instruct",
            "deepseek-r1-distill-qwen-14b",
            "llama-3.1-8b-instruct",
        ],
        ("lmstudio", "haiku") => &[
            "qwen2.5-coder-7b-instruct",
            "deepseek-r1-distill-qwen-7b",
            "qwen2.5-coder-1.5b-instruct",
        ],
        _ => match tier {
            "epic" => &["claude-fable-5", "deepseek/deepseek-r1", "deepseek-r1:70b", "z-ai/glm-5"],
            "large" => &[
                "claude-3-7-sonnet",
                "claude-3-opus",
                "minimax/minimax-01",
                "google/gemini-2.5-pro",
                "qwen2.5-coder:32b",
            ],
            "medium" => &[
                "claude-3-5-sonnet",
                "deepseek/deepseek-chat",
                "google/gemini-2.5-flash",
                "qwen2.5-coder:14b",
            ],
            _ => &[
                "claude-3-5-haiku",
                "google/gemini-2.5-flash-lite",
                "qwen2.5-coder:7b",
                "gemma4:cloud",
            ],
        },
    };

    // Ensure default_model is at the top if not present
    if !raw_candidates.contains(&default_model.to_string()) {
        raw_candidates.push(default_model.to_string());
    }

    for s in suggestions {
        let string_s = s.to_string();
        if !raw_candidates.contains(&string_s) {
            raw_candidates.push(string_s);
        }
    }

    // Build display strings with capability badges
    let mut options: Vec<String> = Vec::new();
    let mut raw_model_keys: Vec<String> = Vec::new();

    for m in &raw_candidates {
        let caps = format_capabilities(&infer_capabilities(m));
        let badge_str = if !caps.is_empty() {
            format!(" [{}]", caps)
        } else {
            "".to_string()
        };
        let is_def = if m == default_model { " (Default)" } else { "" };
        options.push(format!("{} {}{}", m, badge_str, is_def));
        raw_model_keys.push(m.clone());
    }

    let custom_idx = options.len();
    options.push("✏️  Custom Model ID...".to_string());
    let back_idx = options.len();
    options.push("⬅️  Back".to_string());

    let option_refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

    let search_prompt = format!("{} (Type to search)", prompt);

    let sel = dialoguer::FuzzySelect::with_theme(theme)
        .with_prompt(&search_prompt)
        .items(&option_refs)
        .default(0)
        .interact_opt()
        .map_err(|e| format!("select error: {}", e))?;

    let idx = match sel {
        Some(i) => i,
        None => return Ok(None),
    };

    if idx == back_idx {
        return Ok(None);
    }

    if idx == custom_idx {
        let custom: String = Input::with_theme(theme)
            .with_prompt("Enter custom model string")
            .default(default_model.to_string())
            .interact_text()
            .map_err(|e| format!("input error: {}", e))?;
        Ok(Some(custom))
    } else {
        Ok(Some(raw_model_keys[idx].clone()))
    }
}

/// Handles the "Reset to Claude Code Defaults" flow.
fn handle_reset_flow() -> Result<FlowResult, String> {
    let theme = ColorfulTheme::default();
    let confirm = Confirm::with_theme(&theme)
        .with_prompt("Are you sure you want to reset Claude Code back to official Anthropic defaults?")
        .default(true)
        .interact()
        .map_err(|e| format!("confirm error: {}", e))?;

    if !confirm {
        println!("{}", style("Reset cancelled.").dim());
        return Ok(FlowResult::Back);
    }

    match settings::reset_settings() {
        Ok(backup) => {
            ui::print_reset_success(backup.as_deref());
            Ok(FlowResult::Done)
        }
        Err(e) => {
            eprintln!("error: {}", e);
            Err(e)
        }
    }
}

/// If the Ollama provider is defined, probe `/api/tags`.
fn refresh_ollama_warning(config: &Config) {
    let base_url = match config.providers.get("ollama") {
        Some(p) => p.base_url.clone(),
        None => return,
    };
    let tags = ollama::fetch_ollama_tags(&base_url);
    if tags.is_empty() {
        println!(
            "{}",
            style(format!(
                "warn: Ollama at {} unreachable or returned no tags; using configured preset values.",
                base_url
            ))
            .yellow()
        );
    } else {
        println!(
            "{}",
            style(format!("info: detected {} local Ollama model(s).", tags.len())).green()
        );
    }
}
