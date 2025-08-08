// Placeholder implementation for OpenTofu plugin system
// This will be properly implemented when protobuf generation is working

#[derive(Default)]
pub struct ProviderServer;

impl ProviderServer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
    
    #[allow(dead_code)]
    pub async fn start(&self) -> anyhow::Result<()> {
        crate::terminal::info("OpenTofu provider server placeholder started");
        Ok(())
    }
}
