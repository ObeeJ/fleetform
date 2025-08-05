use crate::{terminal, workspace::Workspace};

#[allow(dead_code)]
pub async fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        list_workspaces().await
    } else {
        match args[2].as_str() {
            "new" => {
                if args.len() < 4 {
                    terminal::error("Usage: fleetform workspace new <name>");
                    return Ok(());
                }
                create_workspace(&args[3]).await
            }
            "select" => {
                if args.len() < 4 {
                    terminal::error("Usage: fleetform workspace select <name>");
                    return Ok(());
                }
                select_workspace(&args[3]).await
            }
            "list" => list_workspaces().await,
            _ => {
                terminal::error("Usage: fleetform workspace [new|select|list] <name>");
                Ok(())
            }
        }
    }
}

#[allow(dead_code)]
async fn create_workspace(name: &str) -> anyhow::Result<()> {
    let working_dir = std::env::current_dir()?;
    let workspace = Workspace::new(name, working_dir);
    workspace.create()?;
    
    let state_path = workspace.get_state_path();
    terminal::info(&format!("Workspace state path: {:?}", state_path));
    Ok(())
}

#[allow(dead_code)]
async fn select_workspace(name: &str) -> anyhow::Result<()> {
    let working_dir = std::env::current_dir()?;
    let workspace = Workspace::new(name, working_dir);
    workspace.switch_to()?;
    
    let state_path = workspace.get_state_path();
    terminal::info(&format!("Active workspace state: {:?}", state_path));
    Ok(())
}

#[allow(dead_code)]
async fn list_workspaces() -> anyhow::Result<()> {
    terminal::info("Available workspaces:");
    
    let current = Workspace::get_current()?;
    let workspaces = Workspace::list_workspaces()?;
    
    for workspace in workspaces {
        if workspace == current {
            terminal::success(&format!("* {} (current)", workspace));
        } else {
            terminal::info(&format!("  {}", workspace));
        }
    }
    
    Ok(())
}