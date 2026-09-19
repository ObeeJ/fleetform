use crate::{engine, state, terminal};
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    run_with_approval(false).await
}

pub async fn run_with_approval(auto_approve: bool) -> anyhow::Result<()> {
    terminal::info("Applying infrastructure...");
    let desired = engine::load_desired()?;
    let mut current = state::load().await?;
    let plan = engine::build_plan(&desired, &current);

    terminal::info(&format!(
        "Plan: {} to add, {} to change, {} to destroy",
        plan.add, plan.change, plan.destroy
    ));
    if plan.is_empty() {
        terminal::success("Nothing to apply.");
        return Ok(());
    }
    if !auto_approve && !engine::live_enabled() {
        terminal::info("Dry apply (no FLEETFORM_LIVE). Recording planned resources only.");
    }

    let engine = engine::Engine::new().await;
    engine.apply_plan(&plan, &desired, &mut current).await?;
    state::save(&current).await?;

    if std::env::var("FLEETFORM_S3_BUCKET").is_ok() {
        let bucket = std::env::var("FLEETFORM_S3_BUCKET").unwrap_or_else(|_| "fleetform-state".into());
        current.write_s3(&bucket, "fleetform.json").await?;
        terminal::info(&format!("State uploaded to s3://{}/fleetform.json", bucket));
    }

    let running = current
        .managed
        .iter()
        .filter(|r| r.status == "running")
        .count();
    if running > 0 {
        terminal::success(&format!("Apply complete. {} machine(s) running.", running));
    } else if engine::live_enabled() {
        return Err(anyhow::anyhow!(
            "Apply finished without a running machine. Check AWS credentials, AMI, and subnet/VPC defaults."
        ));
    } else {
        terminal::success("Apply recorded locally. Not live.");
    }
    let _ = std::fs::write(
        Path::new("fleetform_plan.json"),
        serde_json::to_string_pretty(&plan)?,
    );
    Ok(())
}
