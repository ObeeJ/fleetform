use crate::{config, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Configuration:");

    let current_dir = std::env::current_dir()?;
    let config = config::load_config(&current_dir)?;

    println!("Providers: {:?}", config.provider);
    println!("Resources: {:?}", config.resource);

    Ok(())
}
