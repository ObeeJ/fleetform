use crate::{provider, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Listing available providers...");

    let providers = provider::list_available_providers().await?;
    terminal::info(&format!("Available providers: {:?}", providers));

    Ok(())
}
