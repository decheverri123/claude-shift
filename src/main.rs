use clap::Parser;

mod apply;
mod badges;
mod config;
mod ollama;
mod paths;
mod settings;
mod ui;
mod wizard;

#[derive(Parser)]
#[command(
    name = "cshift",
    version,
    about = "Switch Claude Code model config by rewriting ~/.claude/settings.json",
    long_about = "A fast, zero-daemon CLI that switches Claude Code's active model \
    configuration. Presets are entirely user-defined in ~/.config/cshift/config.json."
)]
struct Cli {
    /// Scaffold ~/.config/cshift/config.json with default providers.
    #[arg(value_name = "SUBCOMMAND")]
    init: Option<String>,

    /// Print current settings.json tier assignments.
    #[arg(short = 's', long)]
    status: bool,

    /// Print preset names from config.json, non-interactive.
    #[arg(long)]
    list_presets: bool,

    /// Apply a preset directly, skipping the wizard.
    #[arg(short = 'p', long)]
    preset: Option<String>,

    /// Revert settings.json to defaults, timestamped backup first.
    #[arg(short = 'r', long)]
    reset: bool,

    /// View cross-provider model equivalence and capability guide.
    #[arg(short = 'g', long)]
    guide: bool,

    /// Open config.json in your default editor/viewer.
    #[arg(long)]
    open_config: bool,

    /// Show current status with a preset's tier values overlaid as a preview.
    /// Lets you see what would change without actually applying.
    #[arg(long, value_name = "PRESET")]
    preview: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.init.as_deref() {
        Some("init") => {}
        Some(other) => {
            eprintln!(
                "error: unknown argument '{}'. Use `cshift init` to scaffold config, or omit subcommand for the wizard.",
                other
            );
            std::process::exit(1);
        }
        None => {}
    }

    if cli.init.is_some() {
        ui::print_banner();
        let path = config::config_path();
        let created = if !path.exists() {
            match config::init() {
                Ok(p) => {
                    println!(
                        "  {} Created starter config at: {}\n  Edit it to add your custom presets & endpoints, then run `cshift`.\n",
                        console::style("✔").green().bold(),
                        console::style(p.display()).cyan().bold()
                    );
                    true
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            println!(
                "  {} Configuration file already exists at: {}\n",
                console::style("ℹ").cyan().bold(),
                console::style(path.display()).cyan().bold()
            );
            false
        };

        if console::user_attended() {
            let prompt_msg = if created {
                "Would you like to open config.json now?"
            } else {
                "Would you like to open your existing config.json now?"
            };

            let open_now = dialoguer::Confirm::new()
                .with_prompt(prompt_msg)
                .default(true)
                .interact()
                .unwrap_or(false);

            if open_now {
                wizard::open_or_show_config_file();
            }
        }
        return;
    }

    if cli.reset {
        ui::print_banner();
        match settings::reset_settings() {
            Ok(backup) => {
                ui::print_reset_success(backup.as_deref());
            }
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if cli.guide {
        ui::print_banner();
        ui::print_equivalence_guide();
        return;
    }

    if let Some(name) = &cli.preview {
        ui::print_banner();
        let st = settings::CurrentStatus::read();
        match config::load_config() {
            Ok(cfg) => match cfg.find_preset(name) {
                Some(p) => ui::print_status_card(&st, Some(&cfg), Some(p)),
                None => {
                    eprintln!(
                        "error: unknown preset \"{}\". Use --list-presets to see options.",
                        name
                    );
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if cli.status {
        ui::print_banner();
        let st = settings::CurrentStatus::read();
        let cfg = config::load_config().ok();
        ui::print_status_card(&st, cfg.as_ref(), None);
        return;
    }

    if cli.open_config {
        ui::print_banner();
        wizard::open_or_show_config_file();
        return;
    }

    if cli.list_presets {
        ui::print_banner();
        match config::load_config() {
            Ok(cfg) => {
                ui::print_presets_list(&cfg);
            }
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Load config; shared by wizard and --preset.
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(name) = &cli.preset {
        let preset = match cfg.find_preset(name) {
            Some(p) => p,
            None => {
                eprintln!(
                    "error: unknown preset \"{}\". Use --list-presets to see options.",
                    name
                );
                std::process::exit(1);
            }
        };
        match apply::apply_preset(&cfg, preset) {
            Ok(outcome) => {
                ui::print_banner();
                ui::print_success_shift(&outcome);
            }
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Default: interactive wizard.
    if let Err(e) = wizard::run_wizard(&cfg) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
