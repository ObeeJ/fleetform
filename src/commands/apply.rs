use crate::{config, state, terminal};
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    run_with_approval(false).await
}

pub async fn run_with_approval(auto_approve: bool) -> anyhow::Result<()> {
    terminal::info("Applying infrastructure...");

    let config = config::load().await?;
    let state_path = Path::new(".fleetform/state.json");

    let mut current_state = if state_path.exists() {
        state::State::read(state_path)?
    } else {
        state::State::new()
    };

    if !auto_approve {
        terminal::info("Do you want to perform these actions? (yes/no)");
        terminal::info("Assuming 'yes' for demo");
    }

    current_state.resources.push("applied-resource".to_string());
    current_state.write(state_path)?;

    // Use provisioner for actual resource creation
    let provisioner = crate::provisioner::Provisioner::new().await;
    provisioner.provision_resources(&config, &mut current_state).await?;
    
    // Use provider functions with Source struct
    let client = reqwest::Client::new();
    let provider_source = crate::provider::Source::new(client, config.clone());
    provider_source
        .apply_changes(&config, &mut current_state)
        .await?;

    // Also use standalone apply_changes function
    crate::provider::apply_changes(&config, &mut current_state).await?;

    // Write to S3 remote state (if configured)
    if std::env::var("FLEETFORM_S3_BUCKET").is_ok() {
        let bucket = std::env::var("FLEETFORM_S3_BUCKET").unwrap_or("fleetform-state".to_string());
        let key = "fleetform.json";
        current_state.write_s3(&bucket, key).await?;
        terminal::info(&format!("State uploaded to s3://{}/{}", bucket, key));
    }

    terminal::success("Apply complete!");
    Ok(())
}
