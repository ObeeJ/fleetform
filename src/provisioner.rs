use crate::{config::Config, engine, state::State, terminal};

/// Legacy entry point. All creates go through the Phase 1 engine.
pub struct Provisioner;

impl Provisioner {
    pub async fn new() -> Self {
        Provisioner
    }

    pub async fn provision_resources(
        &self,
        _config: &Config,
        state: &mut State,
    ) -> anyhow::Result<()> {
        terminal::info("Provisioner delegates to engine.apply");
        let desired = engine::load_desired()?;
        let plan = engine::build_plan(&desired, state);
        let eng = if plan.live {
            engine::Engine::connect().await?
        } else {
            engine::Engine::dry()
        };
        eng.apply_plan(&plan, &desired, state).await
    }
}
