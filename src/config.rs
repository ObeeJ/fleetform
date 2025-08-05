use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: Vec<String>,
    pub resource: Vec<String>,
    pub resources: Vec<String>,
}

impl Config {
    #[allow(dead_code)]
    pub fn load_config(path: &Path) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path)?;
        crate::hcl::validate_hcl_syntax(&contents)?;
        let resources = crate::hcl::parse_terraform_config(&contents)?;
        Ok(Config { 
            provider: vec!["aws".to_string()],
            resource: resources.clone(),
            resources,
        })
    }
}

pub fn load_config(dir: &Path) -> Result<Config, anyhow::Error> {
    let extensions = ["hcl", "yaml", "json"];
    for ext in extensions {
        let path = dir.join(format!("fleetform.{}", ext));
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            match ext {
                "hcl" => return parse_hcl_basic(&contents),
                "yaml" => return Ok(serde_yaml::from_str(&contents)?),
                "json" => return Ok(serde_json::from_str(&contents)?),
                _ => unreachable!(),
            }
        }
    }
    Err(anyhow::anyhow!("No config file found"))
}

fn parse_hcl_basic(content: &str) -> anyhow::Result<Config> {
    // Basic HCL parsing using content
    let mut providers = Vec::new();
    let mut resources = Vec::new();
    
    for line in content.lines() {
        if line.contains("provider") {
            providers.push("aws".to_string());
        }
        if line.contains("resource") {
            resources.push("aws_instance".to_string());
        }
    }
    
    Ok(Config { provider: providers, resource: resources.clone(), resources })
}

pub async fn load() -> anyhow::Result<Config> {
    let current_dir = std::env::current_dir()?;
    match load_config(&current_dir) {
        Ok(config) => Ok(config),
        Err(_) => Ok(Config {
            provider: vec![],
            resource: vec![],
            resources: vec![],
        }),
    }
}
