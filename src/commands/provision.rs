use crate::{config, provisioner::Provisioner, state, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Provisioning resources...");
    
    let config = config::load().await?;
    let state_path = std::path::Path::new(".fleetform/state.json");
    
    let mut current_state = if state_path.exists() {
        state::State::read(state_path)?
    } else {
        state::State::new()
    };
    
    // Use provisioner directly
    let provisioner = Provisioner::new().await;
    provisioner.provision_resources(&config, &mut current_state).await?;
    
    current_state.write(state_path)?;
    terminal::success("Provisioning complete!");
    
    Ok(())
}