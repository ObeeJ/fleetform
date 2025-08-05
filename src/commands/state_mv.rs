use crate::{state, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Moving state resources...");
    
    let state_path = std::path::Path::new(".fleetform/state.json");
    let mut current_state = if state_path.exists() {
        state::State::read(state_path)?
    } else {
        state::State::new()
    };
    
    // Example state move operation
    if let Some(idx) = current_state.resources.iter().position(|r| r == "old-resource") {
        current_state.resources[idx] = "new-resource".to_string();
        current_state.write(state_path)?;
        terminal::success("Moved old-resource to new-resource");
    } else {
        terminal::info("No resources to move");
    }
    
    Ok(())
}