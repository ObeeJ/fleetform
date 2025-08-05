use crate::{config, dag, state, terminal};
use reqwest::Client;
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Creating execution plan...");

    // Build resource dependency graph
    let mut graph = dag::ResourceGraph::new();
    graph.add_resource("aws_instance.example");
    graph.add_resource("aws_s3_bucket.my_bucket");
    graph.add_dependency("aws_instance.example", "aws_s3_bucket.my_bucket");
    
    let ordered_resources = graph.get_ordered_resources();
    terminal::info(&format!(
        "Resource graph built with {} resources",
        graph.resource_count()
    ));
    terminal::info(&format!("Ordered resources: {:?}", ordered_resources));

    let _config = config::load().await?;
    let state_path = Path::new(".fleetform/state.json");

    let current_state = if state_path.exists() {
        state::State::read(state_path)?
    } else {
        state::State::new()
    };

    let resource_count = current_state.resources.len();
    terminal::info(&format!(
        "Plan: 1 to add, {} to change, 0 to destroy",
        resource_count
    ));
    terminal::info("+ resource \"new-resource\" will be created");

    // Write plan data to file for Fiber UI
    let plan_data = serde_json::json!({
        "plan": ordered_resources,
        "status": "Planning..."
    });
    std::fs::write("fleetform_plan.json", plan_data.to_string())?;
    
    // Fetch UI data from Fiber endpoint
    let client = Client::new();
    match client.get("http://localhost:3001/ui").send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                terminal::info(&format!("Plan UI: {}", json));
            }
        }
        Err(e) => {
            terminal::error(&format!("Failed to connect to Plan UI server: {}", e));
            terminal::warn("Could not connect to Plan UI server");
        }
    }

    Ok(())
}
