use crate::{engine, state, terminal};
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Creating execution plan from main.tf...");
    let desired = engine::load_desired()?;
    let current = state::load().await?;
    let plan = engine::build_plan(&desired, &current);

    terminal::info(&format!(
        "Plan: {} to add, {} to change, {} to destroy",
        plan.add, plan.change, plan.destroy
    ));
    if !plan.live {
        terminal::warn("Dry plan. Set FLEETFORM_LIVE=1 before apply to launch a real machine.");
    }
    for change in &plan.changes {
        let mark = match change.action {
            engine::Action::Create => "+",
            engine::Action::Destroy => "-",
            engine::Action::NoOp => "~",
        };
        terminal::info(&format!("{} {} ({})", mark, change.address, change.note));
    }

    let plan_path = Path::new("fleetform_plan.json");
    std::fs::write(plan_path, serde_json::to_string_pretty(&plan)?)?;
    terminal::success(&format!("Plan written to {}", plan_path.display()));
    Ok(())
}
