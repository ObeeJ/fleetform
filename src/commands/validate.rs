use crate::{config, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Validating configuration...");

    let _config = config::load().await?;

    terminal::success("Configuration is valid");
    Ok(())
}
