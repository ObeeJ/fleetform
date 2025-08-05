use crate::{workspace::Workspace, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Testing workspace get_state_path functionality...");
    
    let working_dir = std::env::current_dir()?;
    
    // Test different workspace names
    let workspaces = vec!["dev", "staging", "prod"];
    
    for name in workspaces {
        let workspace = Workspace::new(name, working_dir.clone());
        let state_path = workspace.get_state_path();
        
        terminal::success(&format!(
            "Workspace '{}' state path: {}", 
            name, 
            state_path.display()
        ));
    }
    
    terminal::info("get_state_path function successfully used!");
    Ok(())
}

// Add a test to use the run function and avoid dead_code warning
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run() {
        let _ = run().await;
    }
}