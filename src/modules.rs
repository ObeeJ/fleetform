use std::path::{Path, PathBuf};

pub struct Module {
    pub name: String,
    pub path: PathBuf,
}

impl Module {
    pub fn load(path: &Path) -> Result<Self, anyhow::Error> {
        let module = Module {
            name: path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            path: path.to_path_buf(),
        };
        module.validate()?;
        Ok(module)
    }
    
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if !self.path.exists() {
            return Err(anyhow::anyhow!("Module path does not exist: {:?}", self.path));
        }
        
        // Check for main.tf or main.hcl
        let main_tf = self.path.join("main.tf");
        let main_hcl = self.path.join("main.hcl");
        
        if !main_tf.exists() && !main_hcl.exists() {
            return Err(anyhow::anyhow!("Module missing main configuration file"));
        }
        
        crate::terminal::success(&format!("Module '{}' validated successfully", self.name));
        Ok(())
    }
    
    pub fn list_modules(base_path: &Path) -> Result<Vec<String>, anyhow::Error> {
        let mut modules = Vec::new();
        if base_path.exists() {
            for entry in std::fs::read_dir(base_path)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    modules.push(entry.file_name().to_string_lossy().into());
                }
            }
        }
        Ok(modules)
    }
}