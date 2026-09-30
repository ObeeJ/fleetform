use crate::{engine, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Validating main.tf...");
    let desired = engine::load_desired()?;
    if desired.is_empty() {
        return Err(anyhow::anyhow!("main.tf parsed but contained no blocks"));
    }
    terminal::success(&format!(
        "Configuration is valid ({} blocks)",
        desired.len()
    ));
    Ok(())
}
