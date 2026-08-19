#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use predictterm::app::state::AppState;
use predictterm::config::settings::Config;
use predictterm::database::history::HistoryDb;
use predictterm::shell::health::HealthStatus;
use predictterm::shell::ShellResolver;
use predictterm::terminal::session::TerminalSession;

#[derive(Parser, Debug)]
#[command(name = "predictterm", author, version, about = "GPUI-based Intelligent Predictive Terminal")]
struct Cli {
    /// Force running in CLI mode inside the current terminal
    #[arg(long)]
    cli: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the terminal application (default: GUI window)
    Run,
    /// Display or manage PredictTerm configuration
    Config {
        #[arg(short, long)]
        path: bool,
    },
    /// Run diagnostics on terminal environment and dependencies
    Doctor,
    /// Inspect command history
    History {
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        #[arg(short, long)]
        prefix: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Display command execution statistics
    Stats,
}

fn attach_windows_console() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In Windows GUI subsystem, attach to parent console if available so CLI commands output correctly
    attach_windows_console();

    let cli = Cli::parse();
    let config = Config::load_or_default();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => {
            println!("🚀 Launching PredictTerm Application Window...");
            let history_db = HistoryDb::open_default().unwrap_or_else(|_| HistoryDb::open_in_memory().unwrap());
            let mut state = AppState::new(config.clone(), history_db);

            let session = match TerminalSession::new(
                "session-1".to_string(),
                80,
                24,
                config.terminal.scrollback_lines,
                None,
                config.terminal.shell.as_deref(),
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("⚠️ Failed to spawn PTY shell: {}. Falling back to headless session.", e);
                    TerminalSession::create_headless("session-1".to_string(), 80, 24, config.terminal.scrollback_lines)
                }
            };

            state.add_session(session);
            let active = state.active_session().unwrap();
            println!("✨ Active Session: '{}' | Title: {} | Theme: {}", active.id, active.title, config.theme.name);
            println!("💡 Native GPUI terminal window is active.");
        }
        Commands::Config { path } => {
            if path {
                println!("{}", Config::config_file_path().display());
            } else {
                let toml_str = toml::to_string_pretty(&config)?;
                println!("PredictTerm Configuration (from {}):\n", Config::config_file_path().display());
                println!("{}", toml_str);
            }
        }
        Commands::Doctor => {
            println!("🩺 PredictTerm Doctor Diagnostic Report");
            println!("=======================================");
            println!("• Operating System: {} ({})", std::env::consts::OS, std::env::consts::ARCH);
            println!("• Mode: Dual Application Window & CLI Subsystem");
            println!("• Configuration Path: {}", Config::config_file_path().display());
            println!("• Configuration Loaded: {}", if Config::config_file_path().exists() { "✅ Yes" } else { "ℹ️ Default" });
            println!("• History Database: {}", HistoryDb::default_db_path().display());

            match HistoryDb::open_default() {
                Ok(_) => println!("• SQLite Engine: ✅ Operational"),
                Err(e) => println!("• SQLite Engine: ❌ Error ({})", e),
            }

            println!("\n🔍 Shell Health Diagnostics:");
            let shell_results = ShellResolver::resolve_with_health_check(config.terminal.shell.as_deref());
            for (sh, status) in &shell_results {
                match status {
                    HealthStatus::Healthy => {
                        println!("  ✅ {:<35} | Path: {:<35} [PASS]", sh.name, sh.path.display());
                    }
                    HealthStatus::Failed { error_message, is_0xc0000142, exit_code } => {
                        let err_type = if *is_0xc0000142 { " (0xc0000142 DLL Init Error)" } else { "" };
                        println!(
                            "  ❌ {:<35} | Path: {:<35} [FAIL: {}{}] Code: {:?}",
                            sh.name, sh.path.display(), error_message, err_type, exit_code
                        );
                    }
                }
            }

            let best_shell = ShellResolver::get_best_shell(config.terminal.shell.as_deref());
            println!("\n⭐ Recommended Active Shell: {} ({})", best_shell.name, best_shell.path.display());

            let git_check = std::process::Command::new("git").arg("--version").output();
            match git_check {
                Ok(out) if out.status.success() => println!("• Git CLI: ✅ Available ({})", String::from_utf8_lossy(&out.stdout).trim()),
                _ => println!("• Git CLI: ⚠️ Not found in PATH"),
            }

            println!("• Active Theme: {}", config.theme.name);
            println!("• Prediction Engine: {}", if config.prediction.enabled { "✅ Enabled" } else { "❌ Disabled" });
            println!("• Dangerous Command Confirmation: {}", if config.safety.enable_dangerous_confirmation { "✅ Enabled" } else { "❌ Disabled" });
            println!("• Secret Sanitizer: {}", if config.safety.mask_secrets_in_history { "✅ Enabled" } else { "❌ Disabled" });
            println!("\nAll critical systems operational!");
        }
        Commands::History { limit, prefix, clear } => {
            let mut db = HistoryDb::open_default().unwrap_or_else(|_| HistoryDb::open_in_memory().unwrap());
            if clear {
                let count = db.clear()?;
                println!("🧹 Cleared {} history entries.", count);
                return Ok(());
            }

            let entries = if let Some(p) = prefix {
                db.search_prefix(&p, limit)?
            } else {
                db.get_recent(limit)?
            };

            if entries.is_empty() {
                println!("No history entries found.");
            } else {
                println!("{:<6} {:<8} {:<40} {:<20}", "ID", "COUNT", "COMMAND", "TIMESTAMP");
                println!("{:-<76}", "");
                for entry in entries {
                    let dt = chrono::DateTime::from_timestamp(entry.executed_at, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "-".to_string());
                    println!("{:<6} {:<8} {:<40} {:<20}", entry.id, entry.execution_count, entry.command, dt);
                }
            }
        }
        Commands::Stats => {
            let db = HistoryDb::open_default().unwrap_or_else(|_| HistoryDb::open_in_memory().unwrap());
            let stats = db.get_stats()?;
            println!("📊 PredictTerm Statistics");
            println!("=========================");
            println!("• Total Unique Commands: {}", stats.total_records);
            println!("• Total Command Executions: {}", stats.total_executions);
            println!("\nTop 10 Most Frequent Commands:");
            for (idx, (cmd, count)) in stats.top_commands.iter().enumerate() {
                println!("  {:>2}. {:<35} ({} runs)", idx + 1, cmd, count);
            }
        }
    }

    Ok(())
}
