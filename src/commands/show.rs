use crate::{state, terminal};

pub async fn run() -> anyhow::Result<()> {
    let current = state::load().await?;
    if current.managed.is_empty() && current.resources.is_empty() {
        terminal::info("No managed resources in .fleetform/state.json");
        return Ok(());
    }
    for rec in &current.managed {
        terminal::info(&format!(
            "{} status={} id={} ip={}",
            rec.address,
            rec.status,
            rec.id.clone().unwrap_or_else(|| "-".into()),
            rec.public_ip.clone().unwrap_or_else(|| "-".into())
        ));
    }
    println!("{}", serde_json::to_string_pretty(&current)?);
    Ok(())
}
