use crate::{registry::ModuleRegistry, modules::Module, terminal};
use std::error::Error;

#[derive(clap::Parser)]
pub struct ModuleCmd {
    #[command(subcommand)]
    command: ModuleSubCommand,
}

#[derive(clap::Subcommand)]
pub enum ModuleSubCommand {
    /// Initialize a new module
    Init {
        #[clap(help = "Name of the module to create")]
        name: String,
    },
    /// Fetch a module from a remote URL
    Get {
        #[clap(help = "URL of the module to fetch")]
        url: String,
    },
    /// List all available modules (both local and cached)
    List,
    /// Validate a module
    Validate {
        #[clap(help = "Name of the module to validate")]
        name: String,
    },
    /// Update module dependencies
    Update {
        /// Name of the module to update
        #[clap(help = "Name of the module to update, or all modules if not specified")]
        name: Option<String>,
    },
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
    
    pub fn error(&self, msg: &str) {
        terminal::error(msg);
    }
}

pub trait Command {
    async fn run(&self, meta: Meta) -> Result<(), Box<dyn Error>>;
}

impl Command for ModuleCmd {
    async fn run(&self, meta: Meta) -> Result<(), Box<dyn Error>> {
        let modules_dir = meta.working_dir.join("modules");
        let registry = ModuleRegistry::new(modules_dir.clone());

        match &self.command {
            ModuleSubCommand::Init { name } => {
                let module_dir = modules_dir.join(name);
                std::fs::create_dir_all(&module_dir)?;
                
                // Create main.tf
                let main_tf = module_dir.join("main.tf");
                std::fs::write(&main_tf, "# Main module configuration\n")?;
                
                // Create variables.tf
                let vars_tf = module_dir.join("variables.tf");
                std::fs::write(&vars_tf, "# Module variables\n")?;
                
                // Create outputs.tf
                let outputs_tf = module_dir.join("outputs.tf");
                std::fs::write(&outputs_tf, "# Module outputs\n")?;
                
                meta.streams.stdout(&format!("Initialized new module '{}'", name));
                Ok(())
            },
            ModuleSubCommand::Update { name: _ } => {
                meta.streams.stdout("Module update functionality coming soon");
                Ok(())
            },
            ModuleSubCommand::Get { url } => {
                registry.fetch_module(url).await?;
                meta.streams.stdout(&format!("Successfully fetched module from {}", url));
                Ok(())
            },
            ModuleSubCommand::List => {
                let cached = registry.list_cached_modules()?;
                let local = Module::list_modules(&modules_dir)?;
                
                meta.streams.stdout("Available modules:");
                meta.streams.stdout("\nLocal modules:");
                for module in local {
                    meta.streams.stdout(&format!("  {}", module));
                }
                meta.streams.stdout("\nCached modules:");
                for module in cached {
                    meta.streams.stdout(&format!("  {}", module));
                }
                Ok(())
            },
            ModuleSubCommand::Validate { name } => {
                let _module = Module::load(&modules_dir.join(name))?;
                meta.streams.stdout(&format!("Module '{}' validation complete", name));
                Ok(())
            },
        }
    }
}

// Legacy run function removed - now using clap integration