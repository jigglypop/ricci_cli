use anyhow::Result;
use colored::*;
use std::io::Write;
use crate::{
    assistant::{DevAssistant, ChatMode},
    analyzer::CodeAnalyzer,
};

pub async fn handle_special_command(command: &str, assistant: &mut DevAssistant) -> Result<()> {
    match command {
        "/clear" => {
            assistant.clear_context();
            println!("{}", "컨텍스트가 초기화되었습니다.".yellow());
        }
        "/cls" | "/new" => {
            // 화면 초기화
            print!("\x1B[2J\x1B[1;1H");
            std::io::stdout().flush()?;
            crate::splash::display_mini_splash();
            if command == "/new" {
                assistant.clear_context();
                println!("{}", "새 대화를 시작합니다.".green());
            }
        }
        "/context" => {
            let context = assistant.get_context_summary();
            println!("{}\n{}", "현재 컨텍스트:".bright_blue(), context);
        }
        "/save" => {
            assistant.save_session("session.json")?;
            println!("{}", "세션이 저장되었습니다.".green());
        }
        "/help" => {
            print_special_commands();
        }
        "/plan" => {
            println!("{}", "작업계획서 템플릿:".bright_blue());
            println!("{}", get_plan_templates());
        }
        "/analyze" => {
            println!("{}", "프로젝트 분석 중...".yellow());
            let analyzer = CodeAnalyzer::new(assistant.get_config().clone())?;
            let report = analyzer.analyze_all(".").await?;
            analyzer.print_full_report(&report);
        }
        cmd if cmd.starts_with("/review ") => {
            let path = cmd.trim_start_matches("/review ").trim();
            println!("{} {}", "코드 리뷰 중:".yellow(), path);
            let review = assistant.review_code(path, "all").await?;
            println!("\n{}", review.format_markdown());
        }
        "/summary" => {
            println!("{}", "작업 계획서를 생성하고 저장하는 중...".yellow());
            let plan = assistant.export_as_plan("markdown").await?;
            let filename = format!("plan_{}.md", chrono::Local::now().format("%Y%m%d_%H%M%S"));
            std::fs::write(&filename, &plan)?;
            println!("{} 작업 계획서가 {} 파일로 저장되었습니다.", "✓".green(), filename.cyan());
        }
        cmd if cmd.starts_with("/mode ") => {
            let mode_str = cmd.trim_start_matches("/mode ").trim();
            let mode = match mode_str {
                "1" => ChatMode::Normal,
                "2" => ChatMode::Concise,
                "3" => ChatMode::Detailed,
                "4" => ChatMode::Code,
                "5" => ChatMode::Planning,
                _ => {
                    println!("{}", "올바른 모드 번호를 입력하세요 (1-5)".red());
                    return Ok(());
                }
            };
            assistant.set_chat_mode(mode);
            println!("{} 모드가 {:?}로 변경되었습니다.", "✓".green(), mode);
        }
        cmd if cmd.starts_with("/doc ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.len() >= 2 {
                let target = parts[1];
                let doc_type = parts.get(2).unwrap_or(&"readme");
                println!("{} {} 문서 생성 중...", doc_type.cyan(), target);
                let doc = assistant.generate_documentation(target, doc_type).await?;
                println!("\n{doc}");
            }
        }
        _ => {
            println!("{}", "알 수 없는 명령어입니다. /help를 입력하세요.".red());
        }
    }
    Ok(())
}

pub fn print_special_commands() {
    println!("{}", "\n주요 명령어 (단축키):".bright_blue().bold());
    println!("  {} ({}, {})    - AI와 대화하는 '대화 모드'로 전환합니다.", "/chat".cyan(), "c".green(), "chat".green());
    println!("  {} ({})            - 이 도움말을 표시합니다.", "/help".cyan(), "h".green());
    println!("  {} ({})        - 현재 대화 내용으로 작업 계획서를 생성하고 파일로 저장합니다.", "/summary".cyan(), "p".green());

    println!("{}", "\n자동완성:".bright_green().bold());
    println!("  {} 또는 {}    - 입력 중 회색으로 표시되는 명령어를 완성합니다.", "Tab".bright_yellow(), "→".bright_yellow());
    println!("  {}         - 가능한 명령어 목록을 확인합니다.", "Ctrl+I".bright_yellow());

    println!("{}", "\n모든 특수 명령어:".bright_blue().bold());
    println!("  {}       - 새 대화 시작 (컨텍스트 초기화)", "/new, /clear".cyan());
    println!("  {}           - 화면을 지웁니다.", "/cls".cyan());
    println!("  {}         - 현재 대화 모드를 확인하고 변경합니다.", "/mode".cyan());
    println!("  {}       - 현재 세션을 파일로 저장합니다.", "/save".cyan());
    println!("  {}     - 현재 프로젝트 구조를 분석합니다.", "/analyze".cyan());
    println!("  {} <file>   - 지정된 파일의 코드를 리뷰합니다.", "/review".cyan());
    println!("  {} <target> - 지정된 대상에 대한 문서를 생성합니다.", "/doc".cyan());
    println!("  {}   - 대화 내용 기반으로 작업계획서를 생성합니다.", "/plan".cyan());
    println!("  {}         - 현재 대화의 컨텍스트 정보를 봅니다.", "/context".cyan());
}

fn get_plan_templates() -> String {
    r#"
1. 웹 애플리케이션:
   ricci plan "React와 Node.js로 소셜 미디어 플랫폼 구축"
   ricci plan "Vue.js와 Django로 전자상거래 사이트 개발"

2. 모바일 앱:
   ricci plan "Flutter로 크로스플랫폼 일정 관리 앱 개발"
   ricci plan "React Native로 피트니스 트래킹 앱 만들기"

3. API 서버:
   ricci plan "GraphQL API 서버 구축 with 인증 시스템"
   ricci plan "마이크로서비스 아키텍처로 RESTful API 설계"

4. 데이터 프로젝트:
   ricci plan "실시간 데이터 파이프라인 구축"
   ricci plan "머신러닝 모델 배포 시스템 개발"

5. DevOps:
   ricci plan "Kubernetes 기반 CI/CD 파이프라인 구축"
   ricci plan "모니터링 및 로깅 시스템 구현"
"#.to_string()
} 