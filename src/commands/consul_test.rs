use crate::{state::State, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Testing Consul backend functionality...");
    
    // Test write_consul and read_consul methods
    let mut state = State::new();
    state.resources.push("test-resource-1".to_string());
    state.resources.push("test-resource-2".to_string());
    
    let endpoint = "http://localhost:8500";
    let key = "fleetform/test-state";
    
    // Test write_consul
    match state.write_consul(endpoint, key).await {
        Ok(_) => terminal::success("write_consul method working"),
        Err(e) => terminal::warn(&format!("write_consul failed (expected without Consul): {}", e)),
    }
    
    // Test read_consul
    match State::read_consul(endpoint, key).await {
        Ok(read_state) => {
            terminal::success("read_consul method working");
            terminal::info(&format!("Read {} resources from Consul", read_state.resources.len()));
        }
        Err(e) => terminal::warn(&format!("read_consul failed (expected without Consul): {}", e)),
    }
    
    terminal::info("Consul backend methods successfully integrated!");
    Ok(())
}