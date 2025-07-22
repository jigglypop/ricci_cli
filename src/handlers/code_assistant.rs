use anyhow::Result;
use colored::*;
use std::path::Path;
use std::fs;
use crate::{
    assistant::{DevAssistant, SafeFileModifier, FileChange},
    config::Config,
};

#[derive(Debug, Clone)]
pub struct CodeAssistantOptions {
    pub analyze: bool,        // 코드 분석
    pub refactor: bool,       // 리팩토링 제안
    pub optimize: bool,       // 성능 최적화
    pub security: bool,       // 보안 취약점 검사
    pub test: bool,          // 테스트 코드 생성
    pub docs: bool,          // 문서화 생성
    pub fix_all: bool,       // 모든 문제 자동 수정
}

impl Default for CodeAssistantOptions {
    fn default() -> Self {
        Self {
            analyze: true,
            refactor: true,
            optimize: true,
            security: true,
            test: false,
            docs: false,
            fix_all: false,
        }
    }
}

pub async fn run_code_assistant_interactive(
    path: &str,
    assistant: &mut DevAssistant,
    _config: &Config,
) -> Result<()> {
    println!("{}", "🚀 고급 코드 어시스턴트".bright_cyan().bold());
    println!("{}", "=".repeat(50).dimmed());
    
    // 옵션 선택
    let options = select_options()?;
    
    if path == "." {
        // 프로젝트 전체 분석
        analyze_project_interactive(assistant, &options).await?;
    } else if Path::new(path).is_file() {
        // 단일 파일 분석
        analyze_file_interactive(path, assistant, &options).await?;
    } else if Path::new(path).is_dir() {
        // 디렉토리 분석
        analyze_directory_interactive(path, assistant, &options).await?;
    } else {
        println!("{} 유효하지 않은 경로입니다: {}", "오류:".red(), path);
    }
    
    Ok(())
}

fn select_options() -> Result<CodeAssistantOptions> {
    use std::io::{self, Write};
    
    println!("\n{}", "분석 옵션을 선택하세요:".yellow());
    println!("1. [✓] 코드 분석 (품질, 스타일, 복잡도)");
    println!("2. [✓] 리팩토링 제안");
    println!("3. [✓] 성능 최적화 제안");
    println!("4. [✓] 보안 취약점 검사");
    println!("5. [ ] 테스트 코드 생성");
    println!("6. [ ] 문서화 생성");
    println!("7. [ ] 모든 문제 자동 수정 (위험!)");
    println!("\n번호를 입력하여 옵션을 토글하세요. Enter를 누르면 시작합니다.");
    
    let mut options = CodeAssistantOptions::default();
    
    loop {
        print!("선택 (Enter로 시작): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            break;
        }
        
        match input {
            "1" => options.analyze = !options.analyze,
            "2" => options.refactor = !options.refactor,
            "3" => options.optimize = !options.optimize,
            "4" => options.security = !options.security,
            "5" => options.test = !options.test,
            "6" => options.docs = !options.docs,
            "7" => options.fix_all = !options.fix_all,
            _ => println!("{}", "잘못된 선택입니다.".red()),
        }
        
        // 현재 상태 표시
        println!("\n현재 선택:");
        println!("1. [{}] 코드 분석", if options.analyze { "✓" } else { " " });
        println!("2. [{}] 리팩토링 제안", if options.refactor { "✓" } else { " " });
        println!("3. [{}] 성능 최적화", if options.optimize { "✓" } else { " " });
        println!("4. [{}] 보안 검사", if options.security { "✓" } else { " " });
        println!("5. [{}] 테스트 생성", if options.test { "✓" } else { " " });
        println!("6. [{}] 문서화", if options.docs { "✓" } else { " " });
        println!("7. [{}] 자동 수정", if options.fix_all { "✓" } else { " " });
    }
    
    Ok(options)
}

async fn analyze_file_interactive(
    file_path: &str,
    assistant: &mut DevAssistant,
    options: &CodeAssistantOptions,
) -> Result<()> {
    let content = fs::read_to_string(file_path)?;
    let extension = Path::new(file_path).extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    println!("\n{} {}", "📄 파일 분석:".cyan(), file_path);
    println!("{}", "=".repeat(50).dimmed());
    
    let mut analysis_results = Vec::new();
    let mut suggested_changes = Vec::new();
    
    // 1. 코드 분석
    if options.analyze {
        println!("\n{}", "🔍 코드 품질 분석 중...".yellow());
        let analysis = analyze_code_quality(assistant, &content, extension).await?;
        println!("{}", analysis);
        analysis_results.push(("코드 품질", analysis));
    }
    
    // 2. 리팩토링 제안
    if options.refactor {
        println!("\n{}", "🔧 리팩토링 기회 찾는 중...".yellow());
        let (suggestions, code) = suggest_refactoring(assistant, &content, extension).await?;
        println!("{}", suggestions);
        if !code.is_empty() {
            suggested_changes.push(FileChange {
                path: file_path.to_string(),
                original_content: content.clone(),
                new_content: code,
                description: "리팩토링 제안".to_string(),
            });
        }
    }
    
    // 3. 성능 최적화
    if options.optimize {
        println!("\n{}", "⚡ 성능 최적화 분석 중...".yellow());
        let (optimization, code) = analyze_performance(assistant, &content, extension).await?;
        println!("{}", optimization);
        if !code.is_empty() {
            suggested_changes.push(FileChange {
                path: file_path.to_string(),
                original_content: content.clone(),
                new_content: code,
                description: "성능 최적화".to_string(),
            });
        }
    }
    
    // 4. 보안 검사
    if options.security {
        println!("\n{}", "🔒 보안 취약점 검사 중...".yellow());
        let security = check_security(assistant, &content, extension).await?;
        println!("{}", security);
        analysis_results.push(("보안 검사", security));
    }
    
    // 5. 테스트 코드 생성
    if options.test {
        println!("\n{}", "🧪 테스트 코드 생성 중...".yellow());
        let test_code = generate_tests(assistant, &content, extension, file_path).await?;
        if !test_code.is_empty() {
            let test_file = format!("{}_test.{}", 
                file_path.trim_end_matches(&format!(".{}", extension)), 
                extension
            );
            suggested_changes.push(FileChange {
                path: test_file,
                original_content: String::new(),
                new_content: test_code,
                description: "테스트 코드".to_string(),
            });
        }
    }
    
    // 6. 문서화 생성
    if options.docs {
        println!("\n{}", "📚 문서 생성 중...".yellow());
        let docs = generate_documentation(assistant, &content, extension).await?;
        println!("{}", docs);
        analysis_results.push(("문서화", docs));
    }
    
    // 변경사항 적용
    if !suggested_changes.is_empty() {
        println!("\n{}", "💡 제안된 변경사항:".green().bold());
        for (idx, change) in suggested_changes.iter().enumerate() {
            println!("{}. {} - {}", idx + 1, change.path, change.description);
        }
        
        if options.fix_all {
            println!("\n{}", "자동 수정 모드가 활성화되어 있습니다.".yellow());
            apply_all_changes(suggested_changes, assistant).await?;
        } else {
            println!("\n변경사항을 검토하고 적용하시겠습니까? (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" {
                let safe_modifier = SafeFileModifier::new(false);
                safe_modifier.modify_with_backup(suggested_changes).await?;
            }
        }
    }
    
    // 최종 요약
    print_analysis_summary(&analysis_results);
    
    Ok(())
}

async fn analyze_code_quality(
    assistant: &DevAssistant,
    content: &str,
    extension: &str
) -> Result<String> {
    let prompt = format!(
        "다음 {} 코드의 품질을 분석해주세요. 다음 관점에서 평가해주세요:\n\
        1. 코드 가독성과 명확성\n\
        2. 변수/함수 네이밍\n\
        3. 코드 구조와 조직화\n\
        4. 복잡도\n\
        5. 주석의 적절성\n\
        6. 에러 처리\n\n\
        코드:\n```{}\n{}\n```",
        extension, extension, content
    );
    
    assistant.query(&prompt).await
}

async fn suggest_refactoring(
    assistant: &DevAssistant,
    content: &str,
    extension: &str
) -> Result<(String, String)> {
    let prompt = format!(
        "다음 {} 코드를 리팩토링해주세요. 다음을 개선해주세요:\n\
        1. 중복 코드 제거\n\
        2. 함수/메서드 분리\n\
        3. 더 나은 추상화\n\
        4. SOLID 원칙 적용\n\
        5. 디자인 패턴 적용\n\n\
        먼저 개선점을 설명하고, 그 다음 전체 리팩토링된 코드를 제공해주세요.\n\n\
        코드:\n```{}\n{}\n```",
        extension, extension, content
    );
    
    let response = assistant.query(&prompt).await?;
    
    // 응답에서 설명과 코드 분리
    if let Some(code_start) = response.find("```") {
        let explanation = response[..code_start].trim().to_string();
        let code = extract_code_block(&response[code_start..], extension);
        Ok((explanation, code))
    } else {
        Ok((response, String::new()))
    }
}

async fn analyze_performance(
    assistant: &DevAssistant,
    content: &str,
    extension: &str
) -> Result<(String, String)> {
    let prompt = format!(
        "다음 {} 코드의 성능을 분석하고 최적화해주세요:\n\
        1. 시간 복잡도 분석\n\
        2. 공간 복잡도 분석\n\
        3. 불필요한 연산 찾기\n\
        4. 캐싱 기회\n\
        5. 병렬 처리 가능성\n\
        6. 메모리 사용 최적화\n\n\
        분석 결과와 최적화된 코드를 제공해주세요.\n\n\
        코드:\n```{}\n{}\n```",
        extension, extension, content
    );
    
    let response = assistant.query(&prompt).await?;
    
    if let Some(code_start) = response.find("```") {
        let explanation = response[..code_start].trim().to_string();
        let code = extract_code_block(&response[code_start..], extension);
        Ok((explanation, code))
    } else {
        Ok((response, String::new()))
    }
}

async fn check_security(
    assistant: &DevAssistant,
    content: &str,
    extension: &str
) -> Result<String> {
    let prompt = format!(
        "다음 {} 코드의 보안 취약점을 검사해주세요:\n\
        1. SQL 인젝션\n\
        2. XSS 취약점\n\
        3. 인증/인가 문제\n\
        4. 민감한 정보 노출\n\
        5. 안전하지 않은 함수 사용\n\
        6. 입력 검증 부족\n\
        7. 암호화 문제\n\n\
        발견된 취약점과 수정 방법을 설명해주세요.\n\n\
        코드:\n```{}\n{}\n```",
        extension, extension, content
    );
    
    assistant.query(&prompt).await
}

async fn generate_tests(
    assistant: &DevAssistant,
    content: &str,
    extension: &str,
    file_path: &str
) -> Result<String> {
    let prompt = format!(
        "다음 {} 코드에 대한 단위 테스트를 생성해주세요:\n\
        1. 정상 케이스 테스트\n\
        2. 엣지 케이스 테스트\n\
        3. 에러 케이스 테스트\n\
        4. 성능 테스트 (필요시)\n\n\
        파일명: {}\n\
        코드:\n```{}\n{}\n```\n\n\
        테스트 코드만 제공해주세요.",
        extension, file_path, extension, content
    );
    
    let response = assistant.query(&prompt).await?;
    Ok(extract_code_block(&response, extension))
}

async fn generate_documentation(
    assistant: &DevAssistant,
    content: &str,
    extension: &str
) -> Result<String> {
    let prompt = format!(
        "다음 {} 코드에 대한 문서를 생성해주세요:\n\
        1. 파일/모듈 개요\n\
        2. 주요 함수/클래스 설명\n\
        3. 파라미터와 반환값\n\
        4. 사용 예제\n\
        5. 주의사항\n\n\
        코드:\n```{}\n{}\n```",
        extension, extension, content
    );
    
    assistant.query(&prompt).await
}

async fn analyze_project_interactive(
    assistant: &mut DevAssistant,
    options: &CodeAssistantOptions,
) -> Result<()> {
    println!("\n{}", "🏗️ 프로젝트 전체 분석".bright_cyan().bold());
    
    // 프로젝트 구조 분석
    let structure_analysis = assistant.query(
        "현재 프로젝트의 구조를 분석하고 아키텍처 개선점을 제안해주세요."
    ).await?;
    
    println!("\n{}", "📊 프로젝트 구조 분석:".green());
    println!("{}", structure_analysis);
    
    // 코드 품질 메트릭
    if options.analyze {
        let metrics = assistant.query(
            "프로젝트의 전반적인 코드 품질 메트릭을 평가해주세요: \
            복잡도, 중복도, 테스트 커버리지, 문서화 수준 등"
        ).await?;
        
        println!("\n{}", "📈 코드 품질 메트릭:".green());
        println!("{}", metrics);
    }
    
    Ok(())
}

async fn analyze_directory_interactive(
    path: &str,
    assistant: &mut DevAssistant,
    options: &CodeAssistantOptions,
) -> Result<()> {
    println!("\n{} {}", "📁 디렉토리 분석:".cyan(), path);
    
    // 디렉토리 내 파일들 분석
    super::handle_folder_code_analysis(path, assistant, &Config::default()).await?;
    
    Ok(())
}

async fn apply_all_changes(
    changes: Vec<FileChange>,
    assistant: &DevAssistant,
) -> Result<()> {
    println!("\n{}", "🔄 모든 변경사항을 적용하는 중...".yellow());
    
    for change in changes {
        println!("  • {} 수정 중...", change.path);
        fs::write(&change.path, &change.new_content)?;
    }
    
    println!("{}", "✓ 모든 변경사항이 적용되었습니다!".green());
    Ok(())
}

fn print_analysis_summary(results: &[(&str, String)]) {
    if results.is_empty() {
        return;
    }
    
    println!("\n{}", "📊 분석 요약".green().bold());
    println!("{}", "=".repeat(50).dimmed());
    
    for (category, _) in results {
        println!("  ✓ {} 완료", category);
    }
}

fn extract_code_block(text: &str, language: &str) -> String {
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
    
    text.trim().to_string()
} 