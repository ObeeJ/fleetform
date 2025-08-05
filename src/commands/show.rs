use crate::{state, terminal};
use reqwest::Client;
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Current state:");

    let state_path = Path::new(".fleetform/state.json");
    let current_state = if state_path.exists() {
        state::State::read(state_path)?
    } else {
        state::State::new()
    };

    println!("{}", serde_json::to_string_pretty(&current_state)?);

    // Show available providers
    let providers = crate::provider::list_available_providers().await?;
    terminal::info(&format!("Available providers: {:?}", providers));

    // Call Fiber UI endpoint
    let client = Client::new();
    match client.get("http://localhost:3001/ui").send().await {
        Ok(response) => {
            if let Ok(text) = response.text().await {
                terminal::info(&format!("Web UI: {}", text));
            }
        }
        Err(_) => terminal::warn("Could not connect to Web UI server"),
    }

    Ok(())
}
