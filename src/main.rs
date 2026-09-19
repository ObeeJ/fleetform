use clap::{Parser, Subcommand};

use crate::commands::module::Command as ModuleCommand;
use crate::commands::workspace::Command as WorkspaceCommand;
#[cfg(not(windows))]
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::process::Command;
use std::{env, thread};

mod commands;
mod config;
mod dag;
mod engine;
mod hcl;
mod modules;
mod proto;
mod provider;
mod provisioner;
mod registry;
mod state;
mod terminal;
mod workspace;

mod tofu {
    pub mod plugin;
    pub mod provider;
    pub mod schema;
}

#[derive(Default)]
pub struct OpenTofuProvider {}

#[derive(Parser)]
#[command(name = "fleetform")]
#[command(about = "A Rust-based Infrastructure as Code CLI tool")]
#[command(version = "0.1.0")]
struct Cli {
    /// Change to directory before executing any operations
    #[arg(short = 'C', long = "chdir", value_name = "DIR")]
    chdir: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Validate,
    Plan,
    Apply,
    Destroy,
    Fmt,
    Show,
    Workspace(commands::workspace::WorkspaceCmd),
    Config,
    Providers,
    StateMv,
    Test,
    Module(commands::module::ModuleCmd),
    Consul,
    Provision,
    HclValidate,
    WorkspaceTest,
    ConsulTest,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    setup_signal_handling();

    if env::var("FLEETFORM_UI").ok().as_deref() == Some("1") {
        if let Err(e) = start_ui_server() {
            terminal::warn(&format!("Failed to start UI server: {}", e));
        }
    }

    let cli = Cli::parse();

    if let Some(dir) = &cli.chdir {
        env::set_current_dir(dir)?;
        terminal::info(&format!("Changed directory to: {}", dir));
    }

    match cli.command {
        None => {
            terminal::info("Fleetform - Infrastructure as Code CLI");
            terminal::info("Use --help for available commands");
            Ok(())
        }
        Some(Commands::Init) => commands::init::run().await,
        Some(Commands::Validate) => commands::validate::run().await,
        Some(Commands::Plan) => commands::plan::run().await,
        Some(Commands::Apply) => commands::apply::run().await,
        Some(Commands::Destroy) => commands::destroy::run().await,
        Some(Commands::Fmt) => commands::fmt::run().await,
        Some(Commands::Show) => commands::show::run().await,
        Some(Commands::Workspace(cmd)) => {
            let meta = commands::workspace::Meta {
                working_dir: std::env::current_dir()?,
                streams: commands::workspace::Streams,
            };
            cmd.run(meta).map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        }
        Some(Commands::Config) => commands::config::run().await,
        Some(Commands::Providers) => commands::providers::run().await,
        Some(Commands::StateMv) => commands::state_mv::run().await,
        Some(Commands::Test) => commands::test::run().await,
        Some(Commands::Module(cmd)) => {
            let meta = commands::module::Meta {
                working_dir: std::env::current_dir()?,
                streams: commands::module::Streams,
            };
            cmd.run(meta)
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        }
        Some(Commands::Consul) => commands::consul::run().await,
        Some(Commands::Provision) => commands::provision::run().await,
        Some(Commands::HclValidate) => commands::hcl_validate::run().await,
        Some(Commands::WorkspaceTest) => commands::workspace_test::run().await,
        Some(Commands::ConsulTest) => commands::consul_test::run().await,
    }
}

fn start_ui_server() -> Result<(), anyhow::Error> {
    thread::spawn(|| {
        let _ = Command::new("go")
            .args(["run", "main.go"])
            .current_dir("fiber")
            .spawn();
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}

fn setup_signal_handling() {
    #[cfg(not(windows))]
    thread::spawn(|| {
        let mut signals = Signals::new([SIGINT]).expect("Failed to register signal handler");
        for sig in signals.forever() {
            if sig == SIGINT {
                crate::terminal::warn("Received interrupt signal, shutting down gracefully...");
                std::process::exit(130);
            }
        }
    });
}
