use anyhow::Result;
use std::path::Path;
use colored::*;
use crate::analyzer::types::{DependencyAnalysis, Dependency};

pub async fn analyze_dependencies(path: &str) -> Result<DependencyAnalysis> {
    let root_path = Path::new(path);
    let mut direct_dependencies = Vec::new();
    let mut dev_dependencies = Vec::new();
    
    // Cargo.toml
    if let Ok(content) = std::fs::read_to_string(root_path.join("Cargo.toml")) {
        parse_cargo_toml(&content, &mut direct_dependencies, &mut dev_dependencies)?;
    }
    
    // package.json
    if let Ok(content) = std::fs::read_to_string(root_path.join("package.json")) {
        parse_package_json(&content, &mut direct_dependencies, &mut dev_dependencies)?;
    }
    
    Ok(DependencyAnalysis {
        direct_dependencies,
        dev_dependencies,
    })
}

pub fn print_dependency_report(deps: &DependencyAnalysis) {
    println!("\n{}", "의존성".bright_cyan().bold());
    println!("직접: {} | 개발: {}", 
        deps.direct_dependencies.len().to_string().yellow(),
        deps.dev_dependencies.len().to_string().yellow()
    );
}

fn parse_cargo_toml(content: &str, deps: &mut Vec<Dependency>, dev_deps: &mut Vec<Dependency>) -> Result<()> {
    let mut section = "";
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed == "[dependencies]" {
            section = "deps";
        } else if trimmed == "[dev-dependencies]" {
            section = "dev";
        } else if trimmed.starts_with('[') {
            section = "";
        } else if !section.is_empty() && trimmed.contains('=') {
            if let Some((name, version)) = trimmed.split_once('=') {
                let dep = Dependency {
                    name: name.trim().to_string(),
                    version: version.trim().trim_matches('"').to_string(),
                };
                
                match section {
                    "deps" => deps.push(dep),
                    "dev" => dev_deps.push(dep),
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

fn parse_package_json(content: &str, deps: &mut Vec<Dependency>, dev_deps: &mut Vec<Dependency>) -> Result<()> {
    let json: serde_json::Value = serde_json::from_str(content)?;
    
    if let Some(obj) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, version) in obj {
            deps.push(Dependency {
                name: name.clone(),
                version: version.as_str().unwrap_or("").to_string(),
            });
        }
    }
    
    if let Some(obj) = json.get("devDependencies").and_then(|d| d.as_object()) {
        for (name, version) in obj {
            dev_deps.push(Dependency {
                name: name.clone(),
                version: version.as_str().unwrap_or("").to_string(),
            });
        }
    }
    
    Ok(())
} 