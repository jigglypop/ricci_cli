use anyhow::Result;
use std::path::Path;
use colored::*;
use walkdir::WalkDir;
use crate::analyzer::types::{ComplexityReport, ComplexityInfo};
use crate::analyzer::structure::{IGNORED_DIRS, SOURCE_EXTENSIONS};

const COMPLEXITY_THRESHOLD: u32 = 10;
const LARGE_FILE_LINES: usize = 500;

pub async fn analyze_complexity(path: &str) -> Result<ComplexityReport> {
    let root_path = Path::new(path);
    let mut complex_files = Vec::new();
    let mut total_complexity = 0u32;
    let mut file_count = 0;
    
    for entry in walk_source_files(root_path) {
        let path = entry.path();
        
        if path.is_file() && is_source_file(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let complexity = calculate_complexity(&content);
                let lines = content.lines().count();
                
                if complexity > COMPLEXITY_THRESHOLD || lines > LARGE_FILE_LINES {
                    complex_files.push(ComplexityInfo {
                        file: path.strip_prefix(root_path)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string(),
                        complexity,
                        lines,
                    });
                }
                
                total_complexity += complexity;
                file_count += 1;
            }
        }
    }
    
    let average_complexity = if file_count > 0 {
        total_complexity as f32 / file_count as f32
    } else {
        0.0
    };
    
    Ok(ComplexityReport {
        average_complexity,
        complex_files,
    })
}

pub fn print_complexity_report(complexity: &ComplexityReport) {
    println!("\n{}", "복잡도".bright_cyan().bold());
    println!("평균 복잡도: {:.1}\n", complexity.average_complexity);
    
    if !complexity.complex_files.is_empty() {
        println!("복잡한 파일:");
        for file in complexity.complex_files.iter().take(5) {
            println!("  {} - 복잡도: {}, {} 라인",
                file.file.dimmed(),
                file.complexity.to_string().yellow(),
                file.lines
            );
        }
    }
}

fn walk_source_files(root_path: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let path = entry.path();
            !path.components().any(|c| {
                IGNORED_DIRS.contains(&c.as_os_str().to_string_lossy().as_ref())
            })
        })
}

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SOURCE_EXTENSIONS.contains(&e))
        .unwrap_or(false)
}

fn calculate_complexity(content: &str) -> u32 {
    let mut complexity = 1;
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("#") {
            continue;
        }
        
        complexity += trimmed.matches("if ").count() as u32;
        complexity += trimmed.matches("for ").count() as u32;
        complexity += trimmed.matches("while ").count() as u32;
        complexity += trimmed.matches("match ").count() as u32;
        complexity += trimmed.matches("&&").count() as u32;
        complexity += trimmed.matches("||").count() as u32;
    }
    
    complexity
} 