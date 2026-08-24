use clap::Parser;

mod apply;
mod badges;
mod config;
mod ollama;
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

    /// Pull an Ollama model by tag (e.g. qwen2.5-coder:32b).
    #[arg(long, value_name = "MODEL")]
    pull: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if cli.init.as_deref() == Some("init") {
        ui::print_banner();
        match config::init() {
            Ok(path) => {
                println!("Created {}", path.display());
                println!("Edit it to add your own presets, then run `cshift`.");
            }
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
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

    if cli.status {
        ui::print_banner();
        let st = settings::CurrentStatus::read();
        let cfg = config::load_config().ok();
        ui::print_status_card(&st, cfg.as_ref());
        return;
    }

    if let Some(model) = &cli.pull {
        ui::print_banner();
        let cfg = match config::load_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        };
        let base_url = match cfg.providers.get("ollama") {
            Some(p) => p.base_url.clone(),
            None => {
                eprintln!("error: no \"ollama\" provider in config. Run `cshift init` first.");
                std::process::exit(1);
            }
        };
        println!("Pulling {} from {}...", model, base_url);
        match ollama::pull_model(&base_url, model) {
            Ok(()) => println!("Done."),
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
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
                eprintln!("error: unknown preset \"{}\". Use --list-presets to see options.", name);
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
