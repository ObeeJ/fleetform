use crate::{hcl, terminal};

pub async fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        terminal::error("Usage: fleetform hcl-validate <file>");
        return Ok(());
    }
    
    let file_path = std::path::Path::new(&args[2]);
    
    // Use HCL functions
    let content = std::fs::read_to_string(file_path)?;
    
    match hcl::validate_hcl_syntax(&content) {
        Ok(()) => {
            terminal::success("HCL syntax is valid");
            
            // Also parse the config
            let resources = hcl::parse_terraform_config(&content)?;
            terminal::info(&format!("Found {} resources", resources.len()));
                
            // Use extract_quoted_value for additional parsing
            for line in content.lines() {
                if line.contains("name") {
                    if let Some(value) = hcl::extract_quoted_value(line) {
                        terminal::info(&format!("Found quoted value: {}", value));
                    }
                }
            }
        }
        Err(e) => terminal::error(&format!("Validation error: {}", e)),
    }
    
    Ok(())
}