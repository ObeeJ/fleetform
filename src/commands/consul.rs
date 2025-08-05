use crate::{state::State, terminal};

pub async fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 4 {
        terminal::error("Usage: fleetform consul <endpoint> <operation>");
        return Ok(());
    }
    
    let endpoint = &args[2];
    let operation = &args[3];
    
    match operation.as_str() {
        "write" => write_state(endpoint).await,
        "read" => read_state(endpoint).await,
        _ => {
            terminal::error("Operations: write, read");
            Ok(())
        }
    }
}

async fn write_state(endpoint: &str) -> anyhow::Result<()> {
    let state_path = std::path::Path::new(".fleetform/state.json");
    let state = if state_path.exists() {
        State::read(state_path)?
    } else {
        State::new()
    };
    
    state.write_consul(endpoint, "fleetform/state").await?;
    terminal::success("State written to Consul");
    Ok(())
}

async fn read_state(endpoint: &str) -> anyhow::Result<()> {
    let state = State::read_consul(endpoint, "fleetform/state").await?;
    terminal::info(&format!("Consul state: {} resources", state.resources.len()));
    for resource in &state.resources {
        terminal::success(&format!("  {}", resource));
    }
    Ok(())
}