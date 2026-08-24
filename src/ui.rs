use std::path::Path;
use console::{style, Color};
use unicode_width::UnicodeWidthStr;

use crate::apply::ApplyOutcome;
use crate::badges::{format_capabilities, infer_capabilities};
use crate::config::Config;
use crate::settings::CurrentStatus;

/// Renders a boxed card with rounded corners, a styled title, and auto-padded content lines.
pub fn render_box(title: &str, content_lines: &[String], border_color: Color) -> String {
    let raw_title = console::strip_ansi_codes(title);
    let title_width = UnicodeWidthStr::width(raw_title.as_ref());

    let mut max_width = title_width + 4;
    for line in content_lines {
        let raw = console::strip_ansi_codes(line);
        let w = UnicodeWidthStr::width(raw.as_ref());
        if w > max_width {
            max_width = w;
        }
    }
    // Add extra padding to look spacious
    max_width += 4;
    if max_width < 74 {
        max_width = 74;
    }

    let b = |s: &str| style(s).fg(border_color).to_string();

    let mut out = String::new();

    // Top border: ╭  <Title>  ───────╮
    let title_display = if !raw_title.is_empty() {
        format!(" {} ", title)
    } else {
        "".to_string()
    };
    let display_title_width = if !raw_title.is_empty() {
        UnicodeWidthStr::width(raw_title.as_ref()) + 2
    } else {
        0
    };

    let remaining_top = if max_width + 2 > display_title_width + 3 {
        max_width + 2 - display_title_width - 3
    } else {
        2
    };

    out.push_str(&format!(
        " {}{}{}{}\n",
        b(" ╭ "),
        title_display,
        b(&"─".repeat(remaining_top)),
        b("╮")
    ));

    // Empty top line
    out.push_str(&format!(
        " {} {}\n",
        b(" │"),
        b(&format!("{:width$}│", "", width = max_width))
    ));

    // Content lines
    for line in content_lines {
        let raw = console::strip_ansi_codes(line);
        let w = UnicodeWidthStr::width(raw.as_ref());
        let pad = if max_width >= w + 2 {
            max_width - w - 2
        } else {
            0
        };
        out.push_str(&format!(
            " {}  {}{}{}\n",
            b(" │"),
            line,
            " ".repeat(pad),
            b("│")
        ));
    }

    // Empty bottom line
    out.push_str(&format!(
        " {} {}\n",
        b(" │"),
        b(&format!("{:width$}│", "", width = max_width))
    ));

    // Bottom border: ╰────────────╯
    out.push_str(&format!(
        " {}{}{}\n",
        b(" ╰"),
        b(&"─".repeat(max_width)),
        b("╯")
    ));

    out
}

/// Prints the ASCII banner and subtitle with a vibrant orange sunset gradient.
pub fn print_banner() {
    let title = r#"
   ██████╗██╗      █████╗ ██╗   ██╗██████╗ ███████╗   ███████╗██╗  ██╗██╗███████╗████████╗
  ██╔════╝██║     ██╔══██╗██║   ██║██╔══██╗██╔════╝   ██╔════╝██║  ██║██║██╔════╝╚══██╔══╝
  ██║     ██║     ███████║██║   ██║██║  ██║█████╗     ███████╗███████║██║█████╗     ██║   
  ██║     ██║     ██╔══██║██║   ██║██║  ██║██╔══╝     ╚════██║██╔══██║██║██╔══╝     ██║   
  ╚██████╗███████╗██║  ██║╚██████╔╝██████╔╝███████╗   ███████║██║  ██║██║██║        ██║   
   ╚═════╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚══════╝   ╚══════╝╚═╝  ╚═╝╚═╝╚═╝        ╚═╝   "#;

    // Reversed Claude/Anthropic sunset orange gradient steps (Dark burnt amber -> Bright apricot)
    let orange_steps = [
        (185, 40, 0),    // Rich burnt amber (#B92800)
        (215, 55, 0),    // Deep rust orange (#D73700)
        (240, 75, 5),    // Terracotta orange (#F04B05)
        (255, 105, 15),  // Vivid Claude flame orange (#FF690F)
        (255, 140, 40),  // Warm gold orange (#FF8C28)
        (255, 175, 75),  // Bright apricot amber (#FFAF4B)
    ];

    let lines: Vec<&str> = title.trim_matches('\n').lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let (r, g, b) = orange_steps[i.min(orange_steps.len() - 1)];
        println!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, line);
    }

    println!(
        "  \x1b[38;2;230;85;15m⚡ Instant Claude Code Model & Provider Switcher\x1b[0m \x1b[38;2;255;170;80m· 4-Tier Control (Haiku, Medium, Large, Epic)\x1b[0m"
    );
    println!();
}

/// Prints the active status card in a boxed container.
pub fn print_status_card(current: &CurrentStatus, config_opt: Option<&Config>) {
    let (title, border_color) = if current.is_default {
        (
            format!("{} Active: Anthropic Official (Default)", style("✔").fg(Color::Color256(214)).bold()),
            Color::Color256(214), // Warm golden amber
        )
    } else {
        // Look up preset name if matching
        let matching_preset = config_opt.and_then(|cfg| {
            cfg.presets.iter().find(|p| {
                let tiers = p.tiers();
                tiers.epic == current.epic
                    && tiers.large == current.large
                    && tiers.medium == current.medium
                    && tiers.haiku == current.haiku
            })
        });

        let provider_name = matching_preset
            .map(|p| p.provider.as_str())
            .or_else(|| {
                current.base_url.as_deref().and_then(|u| {
                    if u.contains("11434") {
                        Some("Ollama (Local & Cloud)")
                    } else if u.contains("1234") {
                        Some("LM Studio")
                    } else if u.contains("openrouter") {
                        Some("OpenRouter")
                    } else {
                        None
                    }
                })
            })
            .unwrap_or("Custom Provider");

        (
            format!("{} Active: {}", style("★").fg(Color::Color256(208)).bold(), provider_name),
            Color::Color256(208), // Vivid vibrant orange
        )
    };

    let mut lines: Vec<String> = Vec::new();

    if let Some(cfg) = config_opt {
        if let Some(p) = cfg.presets.iter().find(|p| {
            let tiers = p.tiers();
            tiers.epic == current.epic && tiers.large == current.large
        }) {
            lines.push(format!(
                "{}        {}",
                style("Preset:").dim(),
                style(&p.name).fg(Color::Color256(214)).bold()
            ));
        }
    }

    let provider_display = if let Some(url) = &current.base_url {
        if url.contains("11434") {
            "OLLAMA"
        } else if url.contains("1234") {
            "LM-STUDIO"
        } else if url.contains("openrouter") {
            "OPENROUTER"
        } else {
            "CUSTOM"
        }
    } else {
        "ANTHROPIC"
    };

    lines.push(format!(
        "{}      {}",
        style("Provider:").dim(),
        style(provider_display).fg(Color::Color256(208)).bold()
    ));

    if let Some(url) = &current.base_url {
        lines.push(format!(
            "{}      {}",
            style("Base URL:").dim(),
            style(url).fg(Color::Color256(215))
        ));
    } else {
        lines.push(format!(
            "{}      {}",
            style("Endpoint:").dim(),
            style("api.anthropic.com (Standard)").dim()
        ));
    }

    lines.push(String::new());
    lines.push(style("Active 4 Model Tiers:").bold().underlined().to_string());

    let fmt_tier = |icon_tier: &str, role: &str, model: Option<&str>, default_model: &str| -> String {
        let display_model = model.unwrap_or(default_model);
        let caps = format_capabilities(&infer_capabilities(display_model));
        let caps_str = if !caps.is_empty() {
            format!(" {}", style(format!("[{}]", caps)).fg(Color::Color256(214)))
        } else {
            "".to_string()
        };
        let model_str = style(display_model).white().bold().to_string();

        format!(
            "  {} {}   {}{}",
            icon_tier,
            style(role).dim(),
            model_str,
            caps_str
        )
    };

    lines.push(fmt_tier(
        &style("👑 Epic Model  ").magenta().bright().bold().to_string(),
        "(Frontier/Agents):",
        current.epic.as_deref(),
        "claude-fable-5",
    ));
    lines.push(fmt_tier(
        &style("🦁 Large Model ").red().bright().bold().to_string(),
        "(Opus/Hybrid):    ",
        current.large.as_deref(),
        "claude-3-7-sonnet / claude-3-opus",
    ));
    lines.push(fmt_tier(
        &style("⚡ Medium Model").cyan().bright().bold().to_string(),
        "(Sonnet/Coding):  ",
        current.medium.as_deref(),
        "claude-3-5-sonnet",
    ));
    lines.push(fmt_tier(
        &style("🐇 Haiku Model ").green().bright().bold().to_string(),
        "(Haiku/Worker):   ",
        current.haiku.as_deref(),
        "claude-3-5-haiku",
    ));

    lines.push(String::new());
    lines.push(
        style("Badges: 🧠 Thinking · 👁️ Vision · 🛠️ Tools · ⚡ Fast · 🌐 Cloud · 🔒 Local")
            .dim()
            .to_string(),
    );

    print!("{}", render_box(&title, &lines, border_color));
}

/// Prints a successful preset application card.
pub fn print_success_shift(outcome: &ApplyOutcome) {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!(
        "{} Claude Code successfully shifted to {}!",
        style("✔").green().bold(),
        style(&outcome.preset_name).cyan().bold()
    ));
    lines.push(String::new());

    let fmt_tier = |icon_tier: &str, model: Option<&str>| -> String {
        let caps = model
            .map(|m| format_capabilities(&infer_capabilities(m)))
            .unwrap_or_default();
        let caps_str = if !caps.is_empty() {
            format!(" {}", style(format!("[{}]", caps)).yellow())
        } else {
            "".to_string()
        };
        let model_str = model
            .map(|m| style(m).white().bold().to_string())
            .unwrap_or_else(|| style("(left as-is)").dim().to_string());

        format!(
            "  {}    {}{}",
            icon_tier,
            model_str,
            caps_str
        )
    };

    lines.push(fmt_tier(
        &style("👑 Epic Tier:  ").magenta().bright().bold().to_string(),
        outcome.tiers.epic.as_deref(),
    ));
    lines.push(fmt_tier(
        &style("🦁 Large Tier: ").red().bright().bold().to_string(),
        outcome.tiers.large.as_deref(),
    ));
    lines.push(fmt_tier(
        &style("⚡ Medium Tier:").cyan().bright().bold().to_string(),
        outcome.tiers.medium.as_deref(),
    ));
    lines.push(fmt_tier(
        &style("🐇 Haiku Tier: ").green().bright().bold().to_string(),
        outcome.tiers.haiku.as_deref(),
    ));

    lines.push(String::new());
    lines.push(format!(
        "  {}      {}",
        style("Base URL:").dim(),
        style(&outcome.base_url).fg(Color::Color256(215))
    ));

    if let Some(backup) = &outcome.backup {
        lines.push(format!(
            "  {} {}",
            style("Settings backup saved to:").dim(),
            style(backup.display()).dim()
        ));
    }

    lines.push(String::new());
    lines.push(format!(
        "{} Run {} in your terminal to start coding with this configuration.",
        style("ℹ").fg(Color::Color256(214)),
        style("claude").bold()
    ));

    let title = style(" Configuration Applied ").fg(Color::Color256(208)).bold().to_string();
    print!("{}", render_box(&title, &lines, Color::Color256(208)));
}

/// Prints a successful settings reset card.
pub fn print_reset_success(backup_path: Option<&Path>) {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!(
        "{} Claude Code has been restored to default Anthropic configuration!",
        style("✔").fg(Color::Color256(214)).bold()
    ));
    lines.push(String::new());
    lines.push(style("• Custom ANTHROPIC_BASE_URL and custom tokens cleared.").dim().to_string());
    lines.push(style("• Model overrides and aliases reset to official defaults.").dim().to_string());
    lines.push(
        style("• Standard Claude Fable 5 / 3.7 Sonnet / Opus / 3.5 Sonnet / Haiku restored.")
            .dim()
            .to_string(),
    );

    if let Some(backup) = backup_path {
        lines.push(String::new());
        lines.push(format!(
            "{} {}",
            style("Backup saved to:").dim(),
            style(backup.display()).dim()
        ));
    }

    let title = style(" Defaults Restored ").fg(Color::Color256(214)).bold().to_string();
    print!("{}", render_box(&title, &lines, Color::Color256(214)));
}

/// Prints the cross-provider model equivalence & capability guide.
pub fn print_equivalence_guide() {
    println!(
        "{}",
        style("\n  📚 Cross-Provider Model Equivalence & Capability Guide:\n").fg(Color::Color256(208)).bold()
    );

    struct GuideSection {
        tier_name: &'static str,
        role: &'static str,
        badges: &'static [&'static str],
        equivalents: &'static [(&'static str, &'static str, &'static [&'static str])],
    }

    let sections = [
        GuideSection {
            tier_name: "👑 Epic Tier",
            role: "Autonomous multi-step agents, long-horizon planning & frontier reasoning",
            badges: &["🧠", "👁️", "🛠️"],
            equivalents: &[
                ("Anthropic Direct", "claude-fable-5", &["🧠", "👁️", "🛠️"]),
                ("OpenRouter", "deepseek/deepseek-r1 / z-ai/glm-5", &["🧠", "🛠️", "🌐"]),
                ("Ollama Cloud", "deepseek-v4-pro:cloud / deepseek-r1:cloud", &["🧠", "🛠️", "🌐"]),
                ("Local (Ollama / LM Studio)", "deepseek-r1:70b / llama3.3:70b", &["🧠", "🛠️", "🔒"]),
            ],
        },
        GuideSection {
            tier_name: "🦁 Large Tier",
            role: "Flagship coding, hybrid reasoning & heavy architecture (Opus & Sonnet 3.7 tier)",
            badges: &["🧠", "🛠️"],
            equivalents: &[
                ("Anthropic Direct", "claude-3-7-sonnet-latest / claude-3-opus-latest", &["🧠", "👁️", "🛠️"]),
                ("OpenRouter", "minimax/minimax-01 / google/gemini-2.5-pro / z-ai/glm-4.7 / qwen/qwen-2.5-coder-32b-instruct", &["🧠", "👁️", "🛠️", "🌐"]),
                ("Ollama Cloud", "minimax-m3:cloud / deepseek-v4-flash:cloud", &["⚡", "🛠️", "🌐"]),
                ("Local (Ollama / LM Studio)", "qwen2.5-coder:32b / deepseek-r1:32b", &["🧠", "🛠️", "🔒"]),
            ],
        },
        GuideSection {
            tier_name: "⚡ Medium Tier",
            role: "Fast, reliable daily coding driver & refactoring (Sonnet 3.5 tier)",
            badges: &["⚡", "🛠️"],
            equivalents: &[
                ("Anthropic Direct", "claude-3-5-sonnet-latest", &["👁️", "🛠️"]),
                ("OpenRouter", "deepseek/deepseek-chat / google/gemini-2.5-flash / z-ai/glm-4.7-flash", &["⚡", "👁️", "🛠️", "🌐"]),
                ("Ollama Cloud", "qwen3.5:cloud / deepseek-v4-flash:cloud", &["⚡", "🛠️", "🌐"]),
                ("Local (Ollama / LM Studio)", "qwen2.5-coder:14b / llama3.1:8b", &["⚡", "🛠️", "🔒"]),
            ],
        },
        GuideSection {
            tier_name: "🐇 Haiku Tier",
            role: "Ultra-fast subagents, background file indexing, git commits & searches (Haiku tier)",
            badges: &["⚡", "🛠️"],
            equivalents: &[
                ("Anthropic Direct", "claude-3-5-haiku-latest", &["⚡", "🛠️"]),
                ("OpenRouter", "google/gemini-2.5-flash-lite / qwen/qwen-2.5-coder-7b-instruct / meta-llama/llama-3.1-8b-instruct", &["⚡", "🛠️", "🌐"]),
                ("Ollama Cloud", "gemma4:cloud", &["⚡", "🌐"]),
                ("Local (Ollama / LM Studio)", "qwen2.5-coder:7b / qwen2.5-coder:1.5b", &["⚡", "🔒"]),
            ],
        },
    ];

    for (i, s) in sections.iter().enumerate() {
        let badge_str = if !s.badges.is_empty() {
            format!(" [{}]", s.badges.join(" "))
        } else {
            "".to_string()
        };
        let styled_tier = match i {
            0 => style(s.tier_name).magenta().bright().bold(),
            1 => style(s.tier_name).red().bright().bold(),
            2 => style(s.tier_name).cyan().bright().bold(),
            _ => style(s.tier_name).green().bright().bold(),
        };
        println!(
            "  {}{}: {}",
            styled_tier,
            style(badge_str).yellow(),
            style(s.role).dim()
        );
        for (provider, model, caps) in s.equivalents {
            let caps_str = if !caps.is_empty() {
                format!(" [{}]", caps.join(" "))
            } else {
                "".to_string()
            };
            let padded_provider = format!("{:<26}", provider);
            println!(
                "    {} {}: {}{}",
                style("•").dim(),
                style(padded_provider).white().bold(),
                style(model).fg(Color::Color256(215)),
                style(caps_str).dim()
            );
        }
        println!();
    }

    println!("  {}", style("Capability Badges Legend:").underlined().bold());
    println!(
        "  {}\n",
        style("🧠 Thinking   👁️ Vision   🛠️ Tools   ⚡ Fast   🌐 Cloud   🔒 Local").dim()
    );
}

/// Prints a list of presets configured in config.json.
pub fn print_presets_list(config: &Config) {
    println!(
        "{}",
        style("\n  Available Claude Shift Presets (4 Tiers):\n").cyan().bold()
    );

    if config.presets.is_empty() {
        println!(
            "  {}",
            style("No presets defined. Run `cshift init` or edit ~/.config/cshift/config.json.").dim()
        );
        return;
    }

    for p in &config.presets {
        let provider_badge = style(format!(" {} ", p.provider.to_uppercase()))
            .black()
            .on_blue()
            .bold();

        println!(
            "  {} {} {}",
            provider_badge,
            style(&p.name).white().bold(),
            style(format!("[--preset \"{}\"]", p.name)).dim()
        );

        let tiers = p.tiers();
        println!(
            "     {} {}  {} {}  {} {}  {} {}",
            style("Epic:").magenta().bright(),
            style(tiers.epic.as_deref().unwrap_or("(unset)")).dim(),
            style("Large:").red().bright(),
            style(tiers.large.as_deref().unwrap_or("(unset)")).dim(),
            style("Medium:").cyan().bright(),
            style(tiers.medium.as_deref().unwrap_or("(unset)")).dim(),
            style("Haiku:").green().bright(),
            style(tiers.haiku.as_deref().unwrap_or("(unset)")).dim(),
        );
        println!();
    }

    println!(
        "  Switch instantly: {}\n",
        style("cshift --preset <preset-name>").white()
    );
}
