use crate::{state, terminal};
use std::path::Path;

pub const STARTER_MAIN_TF: &str = "resource \"aws_instance\" \"example\" {\n  ami           = \"ami-12345678\"\n  instance_type = \"t3.micro\"\n\n  # SSH is opened to this machine's public IP only.\n  # Override with a CIDR, or \"0.0.0.0/0\" to allow the internet.\n  # ssh_cidr = \"203.0.113.4/32\"\n}\n";

pub async fn run() -> anyhow::Result<()> {
    terminal::info("Initializing Fleetform workspace...");
    std::fs::create_dir_all(".fleetform")?;
    let state_path = Path::new(".fleetform/state.json");
    if !state_path.exists() {
        state::State::new().write(state_path)?;
    }
    if !Path::new("main.tf").exists() {
        std::fs::write("main.tf", STARTER_MAIN_TF)?;
        terminal::info("Wrote starter main.tf");
    }
    terminal::success(
        "Workspace ready. Next: fleetform plan, then FLEETFORM_LIVE=1 fleetform apply",
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_config_parses_into_one_instance() {
        crate::hcl::validate_hcl_syntax(STARTER_MAIN_TF).expect("starter main.tf must be valid");
        let blocks = crate::hcl::parse_blocks(STARTER_MAIN_TF).unwrap();
        let resources: Vec<_> = blocks
            .iter()
            .filter(|b| b.block_type == "resource")
            .collect();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].labels, vec!["aws_instance", "example"]);
        assert_eq!(
            resources[0]
                .attributes
                .get("instance_type")
                .map(|s| s.as_str()),
            Some("t3.micro")
        );
        // The commented-out ssh_cidr line must stay a comment, not an attribute.
        assert!(!resources[0].attributes.contains_key("ssh_cidr"));
        assert!(!resources[0].attributes.contains_key("# ssh_cidr"));
    }
}
