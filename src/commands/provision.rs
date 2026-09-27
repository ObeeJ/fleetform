use crate::commands::apply;
use crate::terminal;

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Provision is an alias of apply.");
    apply::run().await
}
