use crate::{state, terminal};
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Initializing Fleetform workspace...");
    std::fs::create_dir_all(".fleetform")?;
    let state_path = Path::new(".fleetform/state.json");
    if !state_path.exists() {
        state::State::new().write(state_path)?;
    }
    if !Path::new("main.tf").exists() {
        std::fs::write(
            "main.tf",
            "resource \"aws_instance\" \"example\" {\n  ami           = \"ami-12345678\"\n  instance_type = \"t3.micro\"\n}\n",
        )?;
        terminal::info("Wrote starter main.tf");
    }
    terminal::success(
        "Workspace ready. Next: fleetform plan, then FLEETFORM_LIVE=1 fleetform apply",
    );
    Ok(())
}
