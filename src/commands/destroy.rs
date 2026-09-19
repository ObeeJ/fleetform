use crate::{engine, state, terminal};

pub async fn run() -> anyhow::Result<()> {
    terminal::warn("Destroy will terminate managed instances listed in local state.");
    let mut current = state::load().await?;
    let engine = if engine::live_enabled() {
        engine::Engine::connect().await?
    } else {
        engine::Engine::dry()
    };
    engine.destroy_all(&mut current).await?;
    state::save(&current).await?;
    terminal::success("Destroy complete.");
    Ok(())
}
