use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;
use colored::*;
use walkdir::WalkDir;
use crate::analyzer::types::{ProjectStructure, LanguageStats};

pub const IGNORED_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", "build", "vendor"];
pub const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "js", "ts", "jsx", "tsx", "py", "java", "go", "c", "cpp", "cs", "rb", "php"
];

pub async fn analyze_structure(path: &str) -> Result<ProjectStructure> {
    let root_path = Path::new(path).canonicalize()?;
    let mut languages = HashMap::new();
    let mut total_files = 0;
    let mut total_lines = 0;
    
    for entry in walk_source_files(&root_path) {
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if SOURCE_EXTENSIONS.contains(&ext_str.as_ref()) {
                    total_files += 1;
                    
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let line_count = content.lines().count();
                        total_lines += line_count;
                        
                        let lang = detect_language(&ext_str);
                        let stats = languages.entry(lang.to_string()).or_insert(LanguageStats {
                            file_count: 0,
                            line_count: 0,
                            percentage: 0.0,
                        });
                        stats.file_count += 1;
                        stats.line_count += line_count;
                    }
                }
            }
        }
    }
    
    // 언어별 비율 계산
    for stats in languages.values_mut() {
        stats.percentage = (stats.line_count as f32 / total_lines.max(1) as f32) * 100.0;
    }
    
    Ok(ProjectStructure {
        root_path,
        total_files,
        total_lines,
        languages,
    })
}

pub fn print_structure_report(structure: &ProjectStructure) {
    println!("\n{}", "프로젝트 구조".bright_cyan().bold());
    println!("총 파일: {} | 총 라인: {}\n", 
        structure.total_files.to_string().yellow(),
        structure.total_lines.to_string().yellow()
    );
    
    if !structure.languages.is_empty() {
        for (lang, stats) in &structure.languages {
            println!("  {} - {} 파일, {} 라인 ({:.1}%)",
                lang.green(),
                stats.file_count,
                stats.line_count,
                stats.percentage
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

fn detect_language(ext: &str) -> &'static str {
    match ext {
        "rs" => "Rust",
        "js" | "jsx" => "JavaScript",
        "ts" | "tsx" => "TypeScript",
        "py" => "Python",
        "java" => "Java",
        "go" => "Go",
        "c" | "cpp" | "cc" => "C/C++",
        "cs" => "C#",
        "rb" => "Ruby",
        "php" => "PHP",
        _ => "Other",
    }
} 