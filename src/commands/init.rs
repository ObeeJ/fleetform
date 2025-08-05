use crate::{config, modules, provider, terminal};
use reqwest::Client;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Initializing Fleetform workspace...");

    // Load module if exists
    let module_path = std::path::Path::new("module");
    if module_path.exists() {
        let module = modules::Module::load(module_path)?;
        terminal::info(&format!("Loaded module: {}", module.name));
    }

    // Create .fleetform directory
    std::fs::create_dir_all(".fleetform")?;

    // Load config and initialize provider source
    let config = config::load().await?;
    let client = Client::new();
    let provider_source = provider::Source::new(client, config);

    // Fetch AWS provider using Source client
    provider_source.fetch_aws_provider().await?;
    provider_source.fetch_provider("aws", "5.0.0").await?;
    provider_source.load_plugin("aws").await?;

    // Also use standalone fetch_provider function
    crate::provider::fetch_provider("google", "4.0.0").await?;

    terminal::success("Fleetform has been successfully initialized!");
    Ok(())
}
