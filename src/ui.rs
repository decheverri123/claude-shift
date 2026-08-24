use console::{style, Color};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

use crate::apply::ApplyOutcome;
use crate::badges::{format_capabilities, infer_capabilities};
use crate::config::{Config, Preset};
use crate::settings::CurrentStatus;

/// Best-effort provider label from base URL when the user hasn't told us
/// which preset they came from. Returns the full pretty name (e.g.
/// "Ollama (Local & Cloud)"). Custom endpoints fall through to "Custom Provider".
fn provider_label_from_url(url: &str) -> &'static str {
    if url.contains("11434") {
        "Ollama (Local & Cloud)"
    } else if url.contains("1234") {
        "LM Studio"
    } else if url.contains("openrouter") {
        "OpenRouter"
    } else {
        "Custom Provider"
    }
}

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
    let content_row = |line: &str| -> String {
        let raw = console::strip_ansi_codes(line);
        let w = UnicodeWidthStr::width(raw.as_ref());
        let pad = if max_width >= w + 2 {
            max_width - w - 2
        } else {
            0
        };
        format!(" {}  {}{}{}\n", b(" │"), line, " ".repeat(pad), b("│"))
    };

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

    out.push_str(&content_row(""));

    for line in content_lines {
        out.push_str(&content_row(line));
    }

    out.push_str(&content_row(""));

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
        (185, 40, 0),   // Rich burnt amber (#B92800)
        (215, 55, 0),   // Deep rust orange (#D73700)
        (240, 75, 5),   // Terracotta orange (#F04B05)
        (255, 105, 15), // Vivid Claude flame orange (#FF690F)
        (255, 140, 40), // Warm gold orange (#FF8C28)
        (255, 175, 75), // Bright apricot amber (#FFAF4B)
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

/// Prints the active status card in a boxed container. When `selected` is
/// `Some`, that preset's tier values are shown alongside the running values
/// so the user can see exactly what switching would change.
pub fn print_status_card(
    current: &CurrentStatus,
    config_opt: Option<&Config>,
    selected: Option<&Preset>,
) {
    let (title, border_color) = if current.is_default {
        (
            format!(
                "{} Active: Anthropic Official (Default)",
                style("✔").fg(Color::Color256(214)).bold()
            ),
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
            .or_else(|| current.base_url.as_deref().map(provider_label_from_url))
            .unwrap_or("Custom Provider");

        (
            format!(
                "{} Active: {}",
                style("★").fg(Color::Color256(208)).bold(),
                provider_name
            ),
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

    if let Some(p) = selected {
        lines.push(format!(
            "{}   {}",
            style("Highlighted:").dim(),
            style(format!("{} → (preview)", p.name)).cyan().bold()
        ));
    }

    let provider_display = current
        .base_url
        .as_deref()
        .map(provider_label_from_url)
        .unwrap_or("ANTHROPIC");

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
    lines.push(
        style("Active 4 Model Tiers:")
            .bold()
            .underlined()
            .to_string(),
    );

    let fmt_tier = |icon_tier: &str,
                    role: &str,
                    model: Option<&str>,
                    default_model: &str,
                    preview: Option<&str>|
     -> String {
        let display_model = model.unwrap_or(default_model);
        let caps = format_capabilities(&infer_capabilities(display_model));
        let caps_str = if !caps.is_empty() {
            format!(" {}", style(format!("[{}]", caps)).fg(Color::Color256(214)))
        } else {
            "".to_string()
        };
        let model_str = style(display_model).white().bold().to_string();
        let preview_str = match preview {
            Some(p) => format!("  {} {}", style("→").dim(), style(p).yellow().bold()),
            None => "".to_string(),
        };

        format!(
            "  {} {}   {}{}{}",
            icon_tier,
            style(role).dim(),
            model_str,
            caps_str,
            preview_str
        )
    };

    let preview_tiers = selected.map(|p| p.tiers());
    let preview_epic = preview_tiers.as_ref().and_then(|t| t.epic.as_deref());
    let preview_large = preview_tiers.as_ref().and_then(|t| t.large.as_deref());
    let preview_medium = preview_tiers.as_ref().and_then(|t| t.medium.as_deref());
    let preview_haiku = preview_tiers.as_ref().and_then(|t| t.haiku.as_deref());

    lines.push(fmt_tier(
        &style("👑 Epic Model  ")
            .magenta()
            .bright()
            .bold()
            .to_string(),
        "(Frontier/Agents):",
        current.epic.as_deref(),
        "claude-fable-5",
        preview_epic,
    ));
    lines.push(fmt_tier(
        &style("🦁 Large Model ").red().bright().bold().to_string(),
        "(Opus/Hybrid):    ",
        current.large.as_deref(),
        "claude-opus-5 / claude-sonnet-5",
        preview_large,
    ));
    lines.push(fmt_tier(
        &style("⚡ Medium Model").cyan().bright().bold().to_string(),
        "(Sonnet/Coding):  ",
        current.medium.as_deref(),
        "claude-sonnet-5",
        preview_medium,
    ));
    lines.push(fmt_tier(
        &style("🐇 Haiku Model ").green().bright().bold().to_string(),
        "(Haiku/Worker):   ",
        current.haiku.as_deref(),
        "claude-haiku-4.5",
        preview_haiku,
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
        "{} Claude Code successfully shifted to {} ({})!",
        style("✔").green().bold(),
        style(&outcome.preset_name).cyan().bold(),
        style(&outcome.provider_name).dim()
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

        format!("  {}    {}{}", icon_tier, model_str, caps_str)
    };

    lines.push(fmt_tier(
        &style("👑 Epic Tier:  ")
            .magenta()
            .bright()
            .bold()
            .to_string(),
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

    let title = style(" Configuration Applied ")
        .fg(Color::Color256(208))
        .bold()
        .to_string();
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
    lines.push(
        style("• Custom ANTHROPIC_BASE_URL and custom tokens cleared.")
            .dim()
            .to_string(),
    );
    lines.push(
        style("• Model overrides and aliases reset to official defaults.")
            .dim()
            .to_string(),
    );
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

    let title = style(" Defaults Restored ")
        .fg(Color::Color256(214))
        .bold()
        .to_string();
    print!("{}", render_box(&title, &lines, Color::Color256(214)));
}

/// Prints the cross-provider model equivalence & capability guide.
pub fn print_equivalence_guide() {
    println!(
        "{}",
        style("\n  📚 Cross-Provider Model Equivalence & Capability Guide:\n")
            .fg(Color::Color256(208))
            .bold()
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
                (
                    "Cloud",
                    "deepseek-v4-pro:cloud · minimax-m3:cloud · kimi-k3:cloud",
                    &["🧠", "🛠️", "🌐"],
                ),
                (
                    "Local (Ollama / LM Studio)",
                    "deepseek-r1:70b · llama3.3:70b",
                    &["🧠", "🛠️", "🔒"],
                ),
            ],
        },
        GuideSection {
            tier_name: "🦁 Large Tier",
            role: "Flagship coding, hybrid reasoning & heavy architecture (Opus 5 / Sonnet 5)",
            badges: &["🧠", "🛠️"],
            equivalents: &[
                (
                    "Anthropic Direct",
                    "claude-opus-5 / claude-sonnet-5",
                    &["🧠", "👁️", "🛠️"],
                ),
                (
                    "Cloud",
                    "deepseek-v4-flash:cloud · kimi-k2.6:cloud · mistral-large-3:cloud",
                    &["⚡", "🛠️", "🌐"],
                ),
                (
                    "Local (Ollama / LM Studio)",
                    "qwen2.5-coder:32b · deepseek-r1:32b",
                    &["🧠", "🛠️", "🔒"],
                ),
            ],
        },
        GuideSection {
            tier_name: "⚡ Medium Tier",
            role: "Fast, reliable daily coding driver & refactoring (Sonnet 5)",
            badges: &["⚡", "🛠️"],
            equivalents: &[
                ("Anthropic Direct", "claude-sonnet-5", &["🧠", "👁️", "🛠️"]),
                (
                    "Cloud",
                    "kimi-k2.7-code:cloud · glm-5.2:cloud · nemotron-3-super:cloud",
                    &["⚡", "🛠️", "🌐"],
                ),
                (
                    "Local (Ollama / LM Studio)",
                    "qwen2.5-coder:14b · llama3.1:8b",
                    &["⚡", "🛠️", "🔒"],
                ),
            ],
        },
        GuideSection {
            tier_name: "🐇 Haiku Tier",
            role:
                "Ultra-fast subagents, background file indexing, git commits & searches (Haiku 4.5)",
            badges: &["⚡", "🛠️"],
            equivalents: &[
                ("Anthropic Direct", "claude-haiku-4.5", &["⚡", "🛠️"]),
                (
                    "Cloud",
                    "gemma4:cloud · qwen3.5:cloud · nemotron-3-nano:cloud",
                    &["⚡", "🌐"],
                ),
                (
                    "Local (Ollama / LM Studio)",
                    "qwen2.5-coder:7b · qwen2.5-coder:1.5b",
                    &["⚡", "🔒"],
                ),
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

    println!(
        "  {}",
        style("Capability Badges Legend:").underlined().bold()
    );
    println!(
        "  {}\n",
        style("🧠 Thinking   👁️ Vision   🛠️ Tools   ⚡ Fast   🌐 Cloud   🔒 Local").dim()
    );
}

/// Prints a list of presets configured in config.json.
pub fn print_presets_list(config: &Config) {
    println!(
        "{}",
        style("\n  Available Claude Shift Presets (4 Tiers):\n")
            .cyan()
            .bold()
    );

    if config.presets.is_empty() {
        println!(
            "  {}",
            style("No presets defined. Run `cshift init` or edit ~/.config/cshift/config.json.")
                .dim()
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

        if let Some(details) = &p.details {
            println!("     {}", style(details).dim().italic());
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn visible_line_widths(rendered: &str) -> Vec<usize> {
        rendered
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| UnicodeWidthStr::width(console::strip_ansi_codes(l).as_ref()))
            .collect()
    }

    #[test]
    fn all_lines_share_the_same_visible_width() {
        let out = render_box(
            "Title",
            &[
                "short".to_string(),
                "a longer content line here".to_string(),
            ],
            Color::White,
        );
        let widths = visible_line_widths(&out);
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "uneven widths: {:?}",
            widths
        );
    }

    #[test]
    fn wide_unicode_title_keeps_lines_aligned() {
        let out = render_box(
            "👑 Epic Model  Status",
            &["plain line".to_string()],
            Color::White,
        );
        let widths = visible_line_widths(&out);
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "uneven widths: {:?}",
            widths
        );
    }

    #[test]
    fn content_survives_rendering() {
        let out = render_box("T", &["unique-content-marker".to_string()], Color::White);
        assert!(console::strip_ansi_codes(&out).contains("unique-content-marker"));
    }

    #[test]
    fn empty_title_does_not_panic_and_stays_aligned() {
        let out = render_box("", &["x".to_string()], Color::White);
        let widths = visible_line_widths(&out);
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "uneven widths: {:?}",
            widths
        );
    }

    #[test]
    fn narrow_content_hits_the_minimum_box_width() {
        let out = render_box("t", &["x".to_string()], Color::White);
        let widths = visible_line_widths(&out);
        assert!(
            widths.iter().all(|&w| w >= 74),
            "expected minimum width of 74: {:?}",
            widths
        );
    }
}
