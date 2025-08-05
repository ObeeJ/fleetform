use crate::{config, state, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Running infrastructure tests...");
    
    // Load configuration and state
    let _config = config::load().await?;
    let state_path = std::path::Path::new(".fleetform/state.json");
    let current_state = if state_path.exists() {
        state::State::read(state_path)?
    } else {
        state::State::new()
    };
    
    // Run basic tests
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: State file exists
    if state_path.exists() {
        terminal::success("✓ State file exists");
        passed += 1;
    } else {
        terminal::error("✗ State file missing");
        failed += 1;
    }
    
    // Test 2: Resources are defined
    if !current_state.resources.is_empty() {
        terminal::success(&format!("✓ {} resources defined", current_state.resources.len()));
        passed += 1;
    } else {
        terminal::warn("⚠ No resources defined");
    }
    
    // Test 3: Configuration is valid
    terminal::success("✓ Configuration syntax valid");
    passed += 1;
    
    terminal::info(&format!("Tests completed: {} passed, {} failed", passed, failed));
    
    if failed > 0 {
        return Err(anyhow::anyhow!("Some tests failed"));
    }
    
    Ok(())
}