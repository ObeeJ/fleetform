use std::path::PathBuf;
use crate::{state::State, terminal};

pub struct Workspace {
    name: String,
    working_dir: PathBuf,
}

#[allow(dead_code)]
impl Workspace {
    pub fn new(name: &str, working_dir: PathBuf) -> Self {
        Workspace {
            name: name.to_string(),
            working_dir,
        }
    }
    
    pub fn create(&self) -> anyhow::Result<()> {
        let workspace_dir = self.working_dir.join(".fleetform").join("workspaces").join(&self.name);
        std::fs::create_dir_all(&workspace_dir)?;
        
        let state_file = self.get_state_path();
        if !state_file.exists() {
            let initial_state = State::new();
            initial_state.write(&state_file)?;
        }
        
        terminal::success(&format!("Workspace '{}' created", self.name));
        Ok(())
    }
    
    pub fn switch_to(&self) -> anyhow::Result<()> {
        let current_workspace_file = PathBuf::from(".fleetform").join("current_workspace");
        std::fs::write(&current_workspace_file, &self.name)?;
        
        terminal::info(&format!("Switched to workspace: {}", self.name));
        Ok(())
    }
    
    pub fn get_current() -> anyhow::Result<String> {
        let current_workspace_file = PathBuf::from(".fleetform").join("current_workspace");
        
        if current_workspace_file.exists() {
            Ok(std::fs::read_to_string(&current_workspace_file)?.trim().to_string())
        } else {
            Ok("default".to_string())
        }
    }
    
    pub fn list_workspaces() -> anyhow::Result<Vec<String>> {
        let workspaces_dir = PathBuf::from(".fleetform").join("workspaces");
        let mut workspaces = vec!["default".to_string()];
        
        if workspaces_dir.exists() {
            for entry in std::fs::read_dir(&workspaces_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    workspaces.push(entry.file_name().to_string_lossy().into());
                }
            }
        }
        
        Ok(workspaces)
    }
    
    pub fn get_state_path(&self) -> PathBuf {
        self.working_dir.join(format!("fleetform-{}.json", self.name))
    }
}