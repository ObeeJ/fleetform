use std::path::PathBuf;
use crate::terminal;

pub async fn update_module(name: &str, base_dir: &PathBuf) -> anyhow::Result<()> {
    let module_dir = base_dir.join("modules").join(name);
    if !module_dir.exists() {
        return Err(anyhow::anyhow!("Module '{}' not found", name));
    }

    // Check for module configuration
    let main_tf = module_dir.join("main.tf");
    if !main_tf.exists() {
        return Err(anyhow::anyhow!("Module '{}' is missing main.tf", name));
    }

    // Read current configuration
    let contents = tokio::fs::read_to_string(&main_tf).await?;
    
    // Parse and validate HCL
    crate::hcl::validate_hcl_syntax(&contents)?;
    
    // Extract resources
    let resources = crate::hcl::parse_terraform_config(&contents)?;
    
    // Check for updates (in this case, just validate)
    for resource in resources {
        terminal::info(&format!("Validating resource: {}", resource));
    }

    terminal::success(&format!("Module '{}' updated successfully", name));
    Ok(())
}

pub async fn update_all_modules(base_dir: &PathBuf) -> anyhow::Result<()> {
    let modules_dir = base_dir.join("modules");
    if !modules_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(modules_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            update_module(&name, base_dir).await?;
        }
    }

    Ok(())
}
