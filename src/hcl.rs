#[derive(Debug)]
pub struct Block {
    pub block_type: String,
    pub labels: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
    #[allow(dead_code)]
    pub blocks: Vec<Block>,
}

pub fn parse_terraform_config(contents: &str) -> Result<Vec<String>, anyhow::Error> {
    let blocks = parse_blocks(contents)?;
    let mut resources = Vec::new();

    for block in blocks {
        if block.block_type == "resource" && block.labels.len() >= 2 {
            resources.push(format!("{}_{}", block.labels[0], block.labels[1]));
        }
    }

    Ok(resources)
}

pub fn parse_blocks(contents: &str) -> Result<Vec<Block>, anyhow::Error> {
    let mut blocks = Vec::new();
    let mut chars = contents.chars().peekable();
    let mut current_block = String::new();
    let mut brace_count = 0;

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                brace_count += 1;
                current_block.push(c);
            }
            '}' => {
                brace_count -= 1;
                current_block.push(c);
                if brace_count == 0 {
                    if !current_block.trim().is_empty() {
                        blocks.push(parse_block(&current_block)?);
                    }
                    current_block.clear();
                }
            }
            '#' => {
                // Skip comments
                while let Some(c) = chars.peek() {
                    if *c == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            _ => {
                if brace_count > 0 || !c.is_whitespace() {
                    current_block.push(c);
                }
            }
        }
    }

    if brace_count != 0 {
        return Err(anyhow::anyhow!("Invalid HCL syntax: unmatched braces"));
    }

    Ok(blocks)
}

fn parse_block(content: &str) -> Result<Block, anyhow::Error> {
    let lines = content.lines().map(|l| l.trim()).collect::<Vec<_>>();
    let header = lines[0].trim_end_matches('{').trim();
    let parts: Vec<_> = header.split_whitespace().collect();

    let block_type = parts[0].to_string();
    let labels = parts[1..].iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect();

    let mut attributes = std::collections::HashMap::new();
    let mut nested_blocks = Vec::new();

    let mut i = 1;
    while i < lines.len() - 1 {
        let line = lines[i].trim();
        if line.starts_with('#') || line.is_empty() {
            i += 1;
            continue;
        }

        if line.contains('=') {
            let parts: Vec<_> = line.splitn(2, '=').collect();
            let key = parts[0].trim().to_string();
            let value = parts[1].trim()
                .trim_matches('"')
                .trim_end_matches(',')
                .to_string();
            attributes.insert(key, value);
        } else if !line.is_empty() {
            // This might be a nested block
            let mut nested_content = String::new();
            let mut brace_count = 0;
            
            while i < lines.len() {
                let line = lines[i].trim();
                nested_content.push_str(line);
                nested_content.push('\n');
                
                brace_count += line.matches('{').count();
                brace_count -= line.matches('}').count();
                
                if brace_count == 0 {
                    break;
                }
                i += 1;
            }
            
            if !nested_content.is_empty() {
                nested_blocks.push(parse_block(&nested_content)?);
            }
        }
        i += 1;
    }

    Ok(Block {
        block_type,
        labels,
        attributes,
        blocks: nested_blocks,
    })
}

pub fn extract_quoted_value(input: &str) -> Option<String> {
    if input.starts_with('"') && input.ends_with('"') {
        Some(input[1..input.len()-1].to_string())
    } else {
        None
    }
}

pub fn validate_hcl_syntax(contents: &str) -> Result<(), anyhow::Error> {
    // First pass: check basic syntax
    let mut brace_count = 0;
    let mut quote_count = 0;
    let mut in_string = false;
    let mut in_comment = false;
    let mut prev_char = None;

    for c in contents.chars() {
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            continue;
        }

        match c {
            '#' if !in_string => in_comment = true,
            '"' if prev_char != Some('\\') => {
                quote_count += 1;
                in_string = !in_string;
            }
            '{' if !in_string => brace_count += 1,
            '}' if !in_string => brace_count -= 1,
            _ => {}
        }

        if brace_count < 0 {
            return Err(anyhow::anyhow!("Invalid HCL syntax: unexpected closing brace"));
        }

        prev_char = Some(c);
    }

    if brace_count != 0 {
        return Err(anyhow::anyhow!("Invalid HCL syntax: unmatched braces"));
    }

    if quote_count % 2 != 0 {
        return Err(anyhow::anyhow!("Invalid HCL syntax: unmatched quotes"));
    }

    // Second pass: try to parse blocks
    parse_blocks(contents)?;

    Ok(())
}