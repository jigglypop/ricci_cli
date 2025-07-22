use anyhow::Result;
use colored::*;
use std::path::Path;
use std::fs;
use walkdir;
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

pub async fn analyze_file_interactive(
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

pub async fn analyze_project_interactive(
    assistant: &mut DevAssistant,
    _options: &CodeAssistantOptions,
) -> Result<()> {
    println!("\n{}", "🏗️ 프로젝트 전체 분석".bright_cyan().bold());
    println!("{}", "=".repeat(50).dimmed());
    
    // 현재 디렉토리의 프로젝트 구조 분석
    let current_dir = std::env::current_dir()?;
    println!("📁 분석 대상: {}", current_dir.display());
    
    // 프로젝트 타입 감지
    let project_type = detect_project_type(&current_dir)?;
    println!("🔍 프로젝트 타입: {}", project_type.bright_green());
    
    // 프로젝트 메타데이터 읽기
    let mut project_metadata = String::new();
    if project_type == "Rust" {
        if let Ok(cargo_toml) = fs::read_to_string(current_dir.join("Cargo.toml")) {
            // Cargo.toml에서 프로젝트 정보 추출
            if let Ok(toml) = cargo_toml.parse::<toml::Value>() {
                if let Some(package) = toml.get("package") {
                    if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                        project_metadata.push_str(&format!("프로젝트명: {}\n", name));
                    }
                    if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                        project_metadata.push_str(&format!("버전: {}\n", version));
                    }
                    if let Some(desc) = package.get("description").and_then(|v| v.as_str()) {
                        project_metadata.push_str(&format!("설명: {}\n", desc));
                    }
                }
                if let Some(deps) = toml.get("dependencies") {
                    if let Some(deps_table) = deps.as_table() {
                        project_metadata.push_str(&format!("의존성 수: {}\n", deps_table.len()));
                    }
                }
            }
        }
    }
    
    // 파일 구조 수집 및 코드 샘플
    let mut files_info = Vec::new();
    let mut code_samples = Vec::new();
    let mut total_lines = 0;
    let mut file_count = 0;
    let mut language_stats: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for entry in walkdir::WalkDir::new(&current_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // 무시할 디렉토리
        if should_ignore_path(path) {
            continue;
        }
        
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                let lines = content.lines().count();
                total_lines += lines;
                file_count += 1;
                
                let relative_path = path.strip_prefix(&current_dir)
                    .unwrap_or(path)
                    .display()
                    .to_string();
                
                files_info.push(format!("- {} ({} 줄)", relative_path, lines));
                
                // 코드 샘플 추출
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_str().unwrap_or("");
                    if matches!(ext_str, "rs" | "js" | "ts" | "py" | "go" | "java") {
                        // 언어별 통계
                        *language_stats.entry(ext_str.to_string()).or_insert(0) += 1;
                        
                        // 주요 파일의 코드 샘플
                        if code_samples.len() < 5 && lines > 50 {
                            let preview = content.lines()
                                .take(20)
                                .collect::<Vec<_>>()
                                .join("\n");
                            code_samples.push(format!("파일: {}\n```{}\n{}\n```", 
                                relative_path, ext_str, preview));
                        }
                    }
                }
            }
        }
    }
    
    // 프로젝트 통계 출력
    println!("\n📊 프로젝트 통계:");
    if !project_metadata.is_empty() {
        print!("{}", project_metadata);
    }
    println!("  • 총 파일 수: {}", file_count);
    println!("  • 총 코드 라인: {}", total_lines.to_string().bright_yellow());
    
    // 언어별 통계
    if !language_stats.is_empty() {
        println!("\n📈 언어별 파일 수:");
        for (lang, count) in &language_stats {
            println!("  • {}: {} 파일", lang, count);
        }
    }
    
    // 주요 파일 목록 (상위 10개)
    println!("\n📄 주요 파일:");
    for (i, file) in files_info.iter().take(10).enumerate() {
        println!("  {}. {}", i + 1, file);
    }
    if files_info.len() > 10 {
        println!("  ... 외 {} 개 파일", files_info.len() - 10);
    }
    
    // AI에게 프로젝트 구조 분석 요청
    let mut project_summary = format!(
        "=== 프로젝트 정보 ===\n{}\n프로젝트 타입: {}\n총 파일: {}\n총 코드 라인: {}\n",
        project_metadata,
        project_type,
        file_count,
        total_lines
    );
    
    // 언어별 통계 추가
    if !language_stats.is_empty() {
        project_summary.push_str("\n언어별 파일:\n");
        for (lang, count) in &language_stats {
            project_summary.push_str(&format!("- {}: {} 파일\n", lang, count));
        }
    }
    
    // 디렉토리 구조 추가
    project_summary.push_str("\n=== 디렉토리 구조 ===\n");
    let mut dirs = std::collections::HashSet::new();
    for entry in walkdir::WalkDir::new(&current_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_dir() && !should_ignore_path(entry.path()) {
            if let Ok(rel_path) = entry.path().strip_prefix(&current_dir) {
                if !rel_path.as_os_str().is_empty() {
                    dirs.insert(rel_path.display().to_string());
                }
            }
        }
    }
    for dir in dirs.iter().take(10) {
        project_summary.push_str(&format!("- {}/\n", dir));
    }
    
    // 주요 파일 정보
    project_summary.push_str("\n=== 주요 파일 ===\n");
    project_summary.push_str(&files_info.iter().take(15).cloned().collect::<Vec<_>>().join("\n"));
    
    // 코드 샘플 추가
    if !code_samples.is_empty() {
        project_summary.push_str("\n\n=== 코드 샘플 ===\n");
        for sample in &code_samples {
            project_summary.push_str(&format!("\n{}\n", sample));
        }
    }
    
    println!("\n🤖 AI가 프로젝트를 분석하고 있습니다...");
    
    let analysis_prompt = format!(
        "다음 {} 프로젝트의 실제 구조와 코드를 분석하고 구체적인 개선점을 제안해주세요:\n\n{}\n\n\
        구체적으로 다음을 분석해주세요:\n\
        1. 현재 프로젝트 구조의 장단점\n\
        2. 모듈 구성과 관심사 분리\n\
        3. 코드 품질과 일관성\n\
        4. 확장성과 유지보수성\n\
        5. 성능 최적화 기회\n\
        6. 보안 고려사항\n\
        7. 테스트 커버리지\n\
        8. 문서화 수준\n\n\
        위 코드 샘플과 구조를 참고하여 구체적이고 실행 가능한 제안을 해주세요.",
        project_type, project_summary
    );
    
    let _analysis = assistant.stream_response(&analysis_prompt).await?;
    
    // 추가 분석 옵션
    println!("\n\n추가 분석을 원하시나요?");
    println!("1. 특정 디렉토리 심층 분석");
    println!("2. 의존성 분석");
    println!("3. 코드 복잡도 분석");
    println!("4. 완료");
    
    use std::io::{self, Write};
    print!("\n선택: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            print!("분석할 디렉토리 경로: ");
            io::stdout().flush()?;
            let mut dir_path = String::new();
            io::stdin().read_line(&mut dir_path)?;
            let dir_path = dir_path.trim();
            
            if !dir_path.is_empty() {
                analyze_directory_interactive(dir_path, assistant, _options).await?;
            }
        }
        "2" => {
            analyze_dependencies(&current_dir, assistant).await?;
        }
        "3" => {
            analyze_code_complexity(&current_dir, assistant).await?;
        }
        _ => {}
    }
    
    Ok(())
}

fn detect_project_type(path: &Path) -> Result<String> {
    if path.join("Cargo.toml").exists() {
        Ok("Rust".to_string())
    } else if path.join("package.json").exists() {
        Ok("Node.js/JavaScript".to_string())
    } else if path.join("requirements.txt").exists() || path.join("setup.py").exists() {
        Ok("Python".to_string())
    } else if path.join("go.mod").exists() {
        Ok("Go".to_string())
    } else if path.join("pom.xml").exists() {
        Ok("Java (Maven)".to_string())
    } else if path.join("build.gradle").exists() {
        Ok("Java (Gradle)".to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

fn should_ignore_path(path: &Path) -> bool {
    let ignore_dirs = vec![
        ".git", "target", "node_modules", ".venv", "venv", 
        "__pycache__", "dist", "build", ".idea", ".vscode"
    ];
    
    path.components().any(|component| {
        if let Some(name) = component.as_os_str().to_str() {
            ignore_dirs.contains(&name)
        } else {
            false
        }
    })
}

async fn analyze_dependencies(path: &Path, assistant: &mut DevAssistant) -> Result<()> {
    println!("\n📦 의존성 분석 중...");
    
    let mut deps_info = String::new();
    
    // Rust 프로젝트
    if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
        deps_info.push_str("Rust 의존성 (Cargo.toml):\n");
        deps_info.push_str(&content);
    }
    
    // Node.js 프로젝트
    if let Ok(content) = fs::read_to_string(path.join("package.json")) {
        deps_info.push_str("\nNode.js 의존성 (package.json):\n");
        deps_info.push_str(&content);
    }
    
    // Python 프로젝트
    if let Ok(content) = fs::read_to_string(path.join("requirements.txt")) {
        deps_info.push_str("\nPython 의존성 (requirements.txt):\n");
        deps_info.push_str(&content);
    }
    
    if !deps_info.is_empty() {
        let prompt = format!(
            "다음 프로젝트 의존성을 분석하고 다음을 확인해주세요:\n\
            1. 오래된 패키지\n\
            2. 보안 취약점이 있는 패키지\n\
            3. 불필요한 의존성\n\
            4. 버전 충돌 가능성\n\n{}",
            deps_info
        );
        
        assistant.stream_response(&prompt).await?;
    } else {
        println!("의존성 파일을 찾을 수 없습니다.");
    }
    
    Ok(())
}

async fn analyze_code_complexity(path: &Path, assistant: &mut DevAssistant) -> Result<()> {
    println!("\n🔬 코드 복잡도 분석 중...");
    
    let mut complex_files = Vec::new();
    
    for entry in walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        
        if should_ignore_path(file_path) || !file_path.is_file() {
            continue;
        }
        
        if let Some(ext) = file_path.extension() {
            let ext_str = ext.to_str().unwrap_or("");
            if matches!(ext_str, "rs" | "js" | "ts" | "py" | "go" | "java") {
                if let Ok(content) = fs::read_to_string(file_path) {
                    let lines = content.lines().count();
                    let functions = count_functions(&content, ext_str);
                    
                    if lines > 300 || functions > 10 {
                        complex_files.push(format!(
                            "{}: {} 줄, {} 함수",
                            file_path.display(),
                            lines,
                            functions
                        ));
                    }
                }
            }
        }
    }
    
    if !complex_files.is_empty() {
        println!("\n복잡한 파일들:");
        for file in &complex_files {
            println!("  • {}", file);
        }
        
        let prompt = format!(
            "다음 복잡한 파일들을 리팩토링하는 방법을 제안해주세요:\n\n{}\n\n\
            각 파일에 대해:\n\
            1. 함수 분리 방법\n\
            2. 모듈화 전략\n\
            3. 코드 단순화 방안",
            complex_files.join("\n")
        );
        
        assistant.stream_response(&prompt).await?;
    } else {
        println!("특별히 복잡한 파일이 발견되지 않았습니다.");
    }
    
    Ok(())
}

fn count_functions(content: &str, extension: &str) -> usize {
    match extension {
        "rs" => content.matches("fn ").count(),
        "js" | "ts" => content.matches("function").count() + content.matches("=>").count(),
        "py" => content.matches("def ").count(),
        "go" => content.matches("func ").count(),
        "java" => content.matches("public ").count() + content.matches("private ").count(),
        _ => 0,
    }
}

pub async fn analyze_directory_interactive(
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