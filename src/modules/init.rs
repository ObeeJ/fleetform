use std::path::PathBuf;
use crate::terminal;

pub fn init_module(name: &str, base_dir: &PathBuf) -> anyhow::Result<()> {
    let module_dir = base_dir.join("modules").join(name);
    std::fs::create_dir_all(&module_dir)?;

    // Create main.tf
    let main_tf = "# Module configuration\nvariable \"environment\" {\n  description = \"Target environment\"\n  type        = string\n}\n\n# Resource declarations go here\n";
    std::fs::write(module_dir.join("main.tf"), main_tf)?;

    // Create outputs.tf
    let outputs_tf = "# Module outputs\n";
    std::fs::write(module_dir.join("outputs.tf"), outputs_tf)?;

    // Create variables.tf
    let variables_tf = "# Module variables\n";
    std::fs::write(module_dir.join("variables.tf"), variables_tf)?;

    // Create README.md
    let readme = format!("# {}\n\nModule documentation\n\n## Usage\n\n```hcl\nmodule \"{}\" {{\n  source = \"./modules/{}\"\n  environment = \"dev\"\n}}\n```\n", 
        name, name, name);
    std::fs::write(module_dir.join("README.md"), readme)?;

    terminal::success(&format!("Module '{}' initialized successfully", name));
    Ok(())
}
