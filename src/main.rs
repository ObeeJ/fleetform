use clap::{Parser, Subcommand};

#[cfg(not(windows))]
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::process::Command;
use std::{env, thread};
use crate::commands::module::Command as ModuleCommand;
use crate::commands::workspace::Command as WorkspaceCommand;

mod commands;
mod config;
mod dag;
mod hcl;
mod modules;
mod provisioner;
mod provider;
mod registry;
mod state;
mod terminal;
mod workspace;

// OpenTofu integration modules
mod tofu {
    pub mod plugin;
    pub mod provider;
    pub mod schema;
}

use std::error::Error;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
use futures_util::StreamExt;

// Generated OpenTofu protobuf types will be added here
pub mod tfplugin6 {
    tonic::include_proto!("tfplugin6");
}

use tfplugin6::provider_server::{Provider, ProviderServer};
use tfplugin6::{
    GetProviderSchema_Response,
    Configure_Response,
    PlanResourceChange_Response,
    ApplyResourceChange_Response,
};

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
    /// Initialize a new Fleetform configuration
    Init,
    /// Validate the configuration files
    Validate,
    /// Create an execution plan
    Plan,
    /// Apply the configuration changes
    Apply,
    /// Destroy managed infrastructure
    Destroy,
    /// Format configuration files
    Fmt,
    /// Show current state
    Show,
    /// Manage workspaces
    Workspace(commands::workspace::WorkspaceCmd),
    /// Show configuration
    Config,
    /// List available providers
    Providers,
    /// Move state resources
    StateMv,
    /// Run infrastructure tests
    Test,
    /// Manage modules
    Module(commands::module::ModuleCmd),
    /// Consul backend operations
    Consul,
    /// Provision resources
    Provision,
    /// Validate HCL syntax
    HclValidate,
    /// Test workspace functionality
    WorkspaceTest,
    /// Test Consul backend
    ConsulTest,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();

    // Setup signal handling
    setup_signal_handling();

    // Start UI server
    if let Err(e) = start_ui_server() {
        terminal::warn(&format!("Failed to start UI server: {}", e));
    }

    // Parse CLI args, including from environment variable
    let mut args = std::env::args().collect::<Vec<_>>();

    // Check for FLEETFORM_CLI_ARGS environment variable
    if let Ok(env_args) = env::var("FLEETFORM_CLI_ARGS") {
        let parsed_args = shellwords::split(&env_args)?;
        args.extend(parsed_args);
    }

    let cli = Cli::parse_from(args);

    // Handle chdir option
    if let Some(dir) = &cli.chdir {
        env::set_current_dir(dir)?;
        terminal::info(&format!("Changed directory to: {}", dir));
    }

    // Execute command
    match cli.command {
        None => {
            terminal::info("Fleetform - Infrastructure as Code CLI");
            terminal::info("Use --help for available commands");
            Ok(())
        },
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
        },
        Some(Commands::Config) => commands::config::run().await,
        Some(Commands::Providers) => commands::providers::run().await,
        Some(Commands::StateMv) => commands::state_mv::run().await,
        Some(Commands::Test) => commands::test::run().await,
        Some(Commands::Module(cmd)) => {
            let meta = commands::module::Meta {
                working_dir: std::env::current_dir()?,
                streams: commands::module::Streams,
            };
            cmd.run(meta).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
            Ok(())
        },
        Some(Commands::Consul) => commands::consul::run().await,
        Some(Commands::Provision) => commands::provision::run().await,
        Some(Commands::HclValidate) => commands::hcl_validate::run().await,
        Some(Commands::WorkspaceTest) => commands::workspace_test::run().await,
        Some(Commands::ConsulTest) => commands::consul_test::run().await
    }
}

fn start_ui_server() -> Result<(), anyhow::Error> {
    thread::spawn(|| {
        Command::new("go")
            .args(["run", "main.go"])
            .current_dir("fiber")
            .spawn()
            .expect("Failed to start Fiber server")
            .wait()
            .expect("Fiber server crashed");
    });
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for server to start
    Ok(())
}

fn setup_signal_handling() {
    #[cfg(not(windows))]
    thread::spawn(|| {
        let mut signals = Signals::new(&[SIGINT]).expect("Failed to register signal handler");
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    crate::terminal::warn("Received interrupt signal, shutting down gracefully...");
                    std::process::exit(130);
                }
                _ => {}
            }
        }
    });
}
