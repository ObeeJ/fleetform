use crate::{workspace::Workspace, terminal};
use std::error::Error;

#[derive(clap::Parser)]
pub struct WorkspaceCmd {
    #[clap(subcommand)]
    command: WorkspaceSubCommand,
}

#[derive(clap::Subcommand)]
pub enum WorkspaceSubCommand {
    /// Create a new workspace
    New {
        #[clap(help = "Name of the workspace to create")]
        name: String,
    },
    /// Switch to a different workspace
    Select {
        #[clap(help = "Name of the workspace to switch to")]
        name: String,
    },
    /// Show the current workspace
    Show,
    /// List all workspaces
    List,
}

pub struct Meta {
    pub working_dir: std::path::PathBuf,
    pub streams: Streams,
}

pub struct Streams;

impl Streams {
    pub fn stdout(&self, msg: &str) {
        terminal::info(msg);
    }
}

pub trait Command {
    fn run(&self, meta: Meta) -> Result<(), Box<dyn Error>>;
}

impl Command for WorkspaceCmd {
    fn run(&self, meta: Meta) -> Result<(), Box<dyn Error>> {
        match &self.command {
            WorkspaceSubCommand::New { name } => {
                let workspace = Workspace::new(name, meta.working_dir);
                workspace.create()?;
                meta.streams.stdout(&format!("Created workspace '{}'", name));
                Ok(())
            },
            WorkspaceSubCommand::Select { name } => {
                let workspace = Workspace::new(name, meta.working_dir);
                workspace.switch_to()?;
                meta.streams.stdout(&format!("Switched to workspace '{}'", name));
                Ok(())
            },
            WorkspaceSubCommand::Show => {
                let current = Workspace::get_current()?;
                meta.streams.stdout(&format!("Current workspace: {}", current));
                Ok(())}
            WorkspaceSubCommand::List => {
                let workspaces = Workspace::list_workspaces()?;
                meta.streams.stdout("Available workspaces:");
                for workspace in workspaces {
                    meta.streams.stdout(&format!("  {}", workspace));
                }
                Ok(())
            },
        }
    }
}
