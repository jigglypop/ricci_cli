pub mod chat;
pub mod command;
pub mod analysis;
pub mod completion;
pub mod config;
pub mod code_assistant;

// Re-export main handler functions
pub use chat::handle_chat;
pub use command::handle_special_command;
pub use analysis::{handle_analyze, handle_review, handle_doc, handle_plan};
pub use completion::install_completions;
pub use config::handle_config;
pub use code_assistant::run_code_assistant_interactive;

use anyhow::Result;
use crate::{
    assistant::{DevAssistant, SafeFileModifier, FileChange},
    config::Config,
};

// 공통 유틸리티 함수들
pub async fn handle_direct_query(query: &str, config: &Config) -> Result<()> {
    let mut assistant = DevAssistant::new(config.clone())?;
    assistant.stream_response(query).await?;
    println!();
    Ok(())
}

pub async fn handle_code_assist(
    file_path: &str,
    assistant: &mut DevAssistant,
    _config: &Config,
) -> Result<()> {
    use colored::*;
    use std::path::Path;
    
    let path = Path::new(file_path);
    
    if !path.exists() {
        println!("{} 파일을 찾을 수 없습니다: {}", "오류:".red(), file_path);
        return Ok(());
    }
    
    if !path.is_file() {
        println!("{} 파일이 아닙니다: {}", "오류:".red(), file_path);
        return Ok(());
    }
    
    // 파일 읽기
    let content = std::fs::read_to_string(path)?;
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    
    println!("{} {}", "📄 파일 분석 중:".cyan(), file_path);
    println!("{}", "=".repeat(50).dimmed());
    
    // 파일 정보 표시
    println!("📊 파일 정보:");
    println!("  • 크기: {} bytes", content.len());
    println!("  • 줄 수: {}", content.lines().count());
    println!("  • 확장자: {}", extension);
    println!();
    
    // AI 분석 요청
    let prompt = format!(
        "다음 {} 파일을 분석하고 개선점을 제안해주세요:\n\n파일: {}\n\n```{}\n{}\n```\n\n다음 관점에서 분석해주세요:\n1. 코드 품질\n2. 가독성\n3. 성능\n4. 보안\n5. 모범 사례",
        extension, file_path, extension, content
    );
    
    println!("{}", "🤖 AI가 코드를 분석하고 있습니다...".yellow());
    let analysis = assistant.query(&prompt).await?;
    
    println!("\n{}", "📋 분석 결과:".green().bold());
    println!("{}", "=".repeat(50).dimmed());
    println!("{}", analysis);
    println!("{}", "=".repeat(50).dimmed());
    
    // 수정 제안 여부 확인
    println!("\n{}", "코드를 수정하시겠습니까? (y/n)".cyan());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" {
        println!("{}", "🔧 수정사항을 생성하고 있습니다...".yellow());
        
        let fix_prompt = format!(
            "위의 분석을 바탕으로 다음 코드를 개선해주세요. 전체 수정된 코드를 제공해주세요:\n\n```{}\n{}\n```",
            extension, content
        );
        
        let improved_code = assistant.query(&fix_prompt).await?;
        
        // 코드 블록 추출
        let improved_code = extract_code_block(&improved_code, extension);
        
        // SafeFileModifier를 사용하여 사용자 확인 후 수정
        let safe_modifier = SafeFileModifier::new(false);
        let changes = vec![FileChange {
            path: file_path.to_string(),
            original_content: content,
            new_content: improved_code,
            description: "AI가 제안한 코드 개선사항".to_string(),
        }];
        
        safe_modifier.modify_with_backup(changes).await?;
    }
    
    Ok(())
}

fn extract_code_block(text: &str, language: &str) -> String {
    // 코드 블록 추출 (```언어 ... ``` 형식)
    let patterns = vec![
        format!("```{}\n", language),
        "```\n".to_string(),
        format!("```{}", language),
        "```".to_string(),
    ];
    
    for pattern in patterns {
        if let Some(start) = text.find(&pattern) {
            let code_start = start + pattern.len();
            if let Some(end) = text[code_start..].find("```") {
                return text[code_start..code_start + end].trim().to_string();
            }
        }
    }
    
    // 코드 블록을 찾지 못하면 전체 텍스트 반환
    text.trim().to_string()
} 

pub async fn handle_folder_code_analysis(
    folder_path: &str,
    assistant: &mut DevAssistant,
    _config: &Config,
) -> Result<()> {
    use colored::*;
    use std::path::Path;
    use walkdir::WalkDir;
    
    let path = Path::new(folder_path);
    
    if !path.exists() {
        println!("{} 폴더를 찾을 수 없습니다: {}", "오류:".red(), folder_path);
        return Ok(());
    }
    
    if !path.is_dir() {
        println!("{} 디렉토리가 아닙니다: {}", "오류:".red(), folder_path);
        return Ok(());
    }
    
    println!("{} {}", "📂 폴더 분석 시작:".cyan(), folder_path);
    println!("{}", "=".repeat(50).dimmed());
    
    // 소스 파일 확장자 목록
    let code_extensions = vec![
        "rs", "py", "js", "ts", "jsx", "tsx", "java", "cpp", "c", "h", "hpp",
        "cs", "go", "rb", "php", "swift", "kt", "scala", "r", "dart", "vue"
    ];
    
    let mut files_analyzed = 0;
    let mut total_issues = Vec::new();
    
    // 하위 폴더의 모든 파일 순회
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        
        // 디렉토리는 건너뛰기
        if entry_path.is_dir() {
            continue;
        }
        
        // 숨김 파일이나 특정 폴더 제외
        let path_str = entry_path.to_string_lossy();
        if path_str.contains("/.git/") || path_str.contains("\\.git\\") ||
           path_str.contains("/node_modules/") || path_str.contains("\\node_modules\\") ||
           path_str.contains("/target/") || path_str.contains("\\target\\") ||
           path_str.contains("/.idea/") || path_str.contains("\\.idea\\") ||
           path_str.contains("/__pycache__/") || path_str.contains("\\__pycache__\\") {
            continue;
        }
        
        // 코드 파일인지 확인
        if let Some(ext) = entry_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if code_extensions.contains(&ext_str.to_lowercase().as_str()) {
                    // 파일 크기 확인 (너무 큰 파일은 건너뛰기)
                    if let Ok(metadata) = entry_path.metadata() {
                        if metadata.len() > 1_000_000 { // 1MB 이상
                            println!("{} {} (너무 큼)", "⏩ 건너뛰기:".yellow(), path_str);
                            continue;
                        }
                    }
                    
                    println!("\n{} {}", "🔍 분석 중:".blue(), path_str);
                    
                    // 파일 읽기
                    if let Ok(content) = std::fs::read_to_string(entry_path) {
                        let lines = content.lines().count();
                        println!("  • 줄 수: {}", lines);
                        
                        // 간단한 코드 품질 체크
                        let mut issues = Vec::new();
                        
                        // TODO 주석 찾기
                        let todo_count = content.matches("TODO").count() + content.matches("FIXME").count();
                        if todo_count > 0 {
                            issues.push(format!("TODO/FIXME 주석 {} 개 발견", todo_count));
                        }
                        
                        // 긴 줄 체크
                        let long_lines = content.lines().filter(|line| line.len() > 100).count();
                        if long_lines > 0 {
                            issues.push(format!("100자 이상 긴 줄 {} 개", long_lines));
                        }
                        
                        // 중복 코드 패턴 간단 체크
                        let lines_vec: Vec<&str> = content.lines().collect();
                        let mut duplicate_count = 0;
                        for i in 0..lines_vec.len().saturating_sub(3) {
                            for j in i+10..lines_vec.len().saturating_sub(3) {
                                if lines_vec[i..i+3] == lines_vec[j..j+3] &&
                                   !lines_vec[i].trim().is_empty() {
                                    duplicate_count += 1;
                                    break;
                                }
                            }
                        }
                        if duplicate_count > 5 {
                            issues.push(format!("중복 코드 패턴 {} 개 발견", duplicate_count));
                        }
                        
                        if !issues.is_empty() {
                            println!("  • 발견된 이슈:");
                            for issue in &issues {
                                println!("    - {}", issue.yellow());
                            }
                            total_issues.push((path_str.to_string(), issues));
                        } else {
                            println!("  • {}", "이슈 없음 ✓".green());
                        }
                        
                        files_analyzed += 1;
                    }
                }
            }
        }
    }
    
    // 전체 요약
    println!("\n{}", "=".repeat(50).dimmed());
    println!("{}", "📊 분석 요약".green().bold());
    println!("  • 분석된 파일 수: {}", files_analyzed);
    println!("  • 이슈가 있는 파일 수: {}", total_issues.len());
    
    if !total_issues.is_empty() {
        println!("\n{}", "📋 이슈 요약:".yellow().bold());
        for (file, issues) in &total_issues {
            println!("\n  {}:", file.cyan());
            for issue in issues {
                println!("    - {}", issue);
            }
        }
    }
    
    // AI 분석 요청 여부
    if files_analyzed > 0 {
        println!("\n{}", "AI로 전체 코드베이스를 분석하시겠습니까? (y/n)".cyan());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            println!("{}", "🤖 AI가 전체 코드베이스를 분석하고 있습니다...".yellow());
            
            let prompt = format!(
                "다음은 프로젝트의 코드 분석 결과입니다:\n\n\
                분석된 파일 수: {}\n\
                이슈가 있는 파일 수: {}\n\n\
                주요 이슈:\n{}\n\n\
                이 프로젝트의 전반적인 코드 품질을 평가하고, 개선 방안을 제시해주세요.",
                files_analyzed,
                total_issues.len(),
                total_issues.iter()
                    .map(|(f, issues)| format!("{}: {}", f, issues.join(", ")))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            
            let analysis = assistant.query(&prompt).await?;
            
            println!("\n{}", "📋 AI 분석 결과:".green().bold());
            println!("{}", "=".repeat(50).dimmed());
            println!("{}", analysis);
            println!("{}", "=".repeat(50).dimmed());
        }
    }
    
    Ok(())
} 