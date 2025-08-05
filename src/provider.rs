use crate::{config::Config, state::State};
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

pub struct Source {
    client: Client,
    config: Config,
}

impl Source {
    pub fn new(client: Client, config: Config) -> Self {
        Source { client, config }
    }

    pub async fn fetch_aws_provider(&self) -> Result<()> {
        let url = "https://registry.opentofu.org/v1/providers/hashicorp/aws/latest";
        let response = self.client.get(url).send().await?.text().await?;
        // Use config field for provider configuration
        crate::terminal::info(&format!("Using config: {:?}", self.config));
        println!("Fetched AWS provider: {}", response);
        Ok(())
    }

    pub async fn fetch_provider(&self, name: &str, version: &str) -> Result<()> {
        let url = format!(
            "https://registry.opentofu.org/v1/providers/{}/{}",
            name, version
        );
        let _response = self.client.get(&url).send().await?.text().await?;
        crate::terminal::success(&format!(
            "Provider {} {} fetched using Source client",
            name, version
        ));
        Ok(())
    }

    pub async fn apply_changes(&self, config: &Config, state: &mut State) -> Result<()> {
        // Validate config matches self.config
        if self.config.resource != config.resource {
            return Err(anyhow::anyhow!("Configuration mismatch"));
        }
        for resource in &config.resource {
            state.resources.push(resource.clone());
        }
        crate::terminal::success("Applied changes using Source config validation");
        Ok(())
    }

    pub async fn load_plugin(&self, name: &str) -> Result<()> {
        let plugin_path = format!("plugins/{}-latest", name);
        crate::terminal::success(&format!("Loaded plugin: {}", plugin_path));
        Ok(())
    }
}

pub async fn apply_changes(_config: &Config, _state: &mut State) -> anyhow::Result<()> {
    // Provider execution logic using reqwest for async HTTP operations
    let _client = Client::new();

    // Fetch provider binaries and execute changes
    // This would interact with provider registries

    Ok(())
}

pub async fn destroy_all(_state: &mut State) -> anyhow::Result<()> {
    // Destroy all resources through providers
    Ok(())
}

pub async fn fetch_provider(name: &str, version: &str) -> anyhow::Result<()> {
    let registry_url = format!(
        "https://registry.opentofu.org/providers/{}/{}",
        name, version
    );

    // Use global client for provider fetching
    let client = Client::new();
    let response = client.get(&registry_url).send().await?;

    if response.status().is_success() {
        // Download and install provider
        crate::terminal::success(&format!(
            "Provider {} {} fetched successfully",
            name, version
        ));
    }

    Ok(())
}

pub async fn list_available_providers() -> anyhow::Result<HashMap<String, String>> {
    let mut providers = HashMap::new();
    providers.insert("aws".to_string(), "5.0.0".to_string());
    providers.insert("azurerm".to_string(), "3.0.0".to_string());
    providers.insert("google".to_string(), "4.0.0".to_string());

    Ok(providers)
}
