use reqwest::Client;
use crate::terminal;

pub struct ModuleRegistry {
    client: Client,
    cache_dir: std::path::PathBuf,
}

impl ModuleRegistry {
    pub fn new(cache_dir: std::path::PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();
        
        ModuleRegistry {
            client: Client::new(),
            cache_dir,
        }
    }
    
    pub async fn fetch_module(&self, url: &str) -> Result<(), anyhow::Error> {
        let response = self.client.get(url).send().await?.text().await?;
        let module_name = url.split('/').last().unwrap_or("unknown");
        std::fs::write(self.cache_dir.join(module_name), response)?;
        terminal::success(&format!("Module fetched: {}", module_name));
        Ok(())
    }
    
    pub fn list_cached_modules(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut modules = Vec::new();
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                modules.push(entry.file_name().to_string_lossy().into());
            }
        }
        Ok(modules)
    }
}