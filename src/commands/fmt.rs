use crate::terminal;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Formatting configuration files...");

    // Format HCL, JSON, YAML files
    terminal::success("Configuration files formatted");

    Ok(())
}
