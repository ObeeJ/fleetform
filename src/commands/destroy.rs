use crate::{provider, state, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::warn("This will destroy all managed infrastructure!");

    let mut current_state = state::load().await?;

    provider::destroy_all(&mut current_state).await?;

    state::save(&current_state).await?;
    terminal::success("Destroy complete!");

    Ok(())
}
