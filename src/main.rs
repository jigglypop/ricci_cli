use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use colored::*;
use anyhow::Result;
use std::io::{self, Write};
use ricci_cli::{
    assistant::DevAssistant,
    planner::ProjectPlanner,
    analyzer::CodeAnalyzer,
    config::Config,
};

#[derive(Parser)]
#[clap(name = "ricci")]
#[clap(about = "AI 기반 개발 어시스턴트 CLI", version)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
    
    /// 직접 질문하기 (서브커맨드 없이)
    #[clap(value_name = "QUERY", conflicts_with = "command")]
    query: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// 대화형 모드로 실행
    Chat {
        /// 프로젝트 컨텍스트 포함
        #[clap(short, long)]
        context: bool,
        
        /// 세션 저장 경로
        #[clap(short, long)]
        save: Option<String>,
    },
    
    /// 작업계획서 생성
    Plan {
        /// 프로젝트 설명 또는 요구사항
        description: String,
        
        /// 출력 형식 (markdown, json, yaml)
        #[clap(short, long, default_value = "markdown")]
        format: String,
        
        /// 상세 레벨 (1-5)
        #[clap(short, long, default_value = "3")]
        detail: u8,
        
        /// 일정 추정 포함
        #[clap(short, long)]
        estimate: bool,
    },
    
    /// 프로젝트 분석
    Analyze {
        /// 분석할 디렉토리 경로
        #[clap(default_value = ".")]
        path: String,
        
        /// 분석 유형 (structure, dependencies, complexity, all)
        #[clap(short, long, default_value = "all")]
        type_: String,
    },
    
    /// 코드 리뷰
    Review {
        /// 리뷰할 파일 또는 디렉토리
        path: String,
        
        /// 리뷰 기준 (security, performance, style, all)
        #[clap(short, long, default_value = "all")]
        criteria: String,
    },
    
    /// 문서 생성
    Doc {
        /// 문서화할 대상
        target: String,
        
        /// 문서 유형 (api, guide, readme, architecture)
        #[clap(short, long, default_value = "readme")]
        type_: String,
    },
    
    /// 설정 관리
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
    
    /// 쉘 완성 스크립트 생성
    Completion {
        /// 대상 쉘
        #[clap(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// API 키 설정
    SetKey {
        /// API 제공자 (openai, anthropic, gemini)
        provider: String,
        /// API 키
        key: String,
    },
    /// 설정 보기
    Show,
    /// 설정 초기화
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 설정 로드
    let config = Config::load()?;
    
    match cli.command {
        Some(Commands::Chat { context, save }) => {
            handle_chat(context, save.as_deref(), &config).await?;
        }
        Some(Commands::Plan { description, format, detail, estimate }) => {
            handle_plan(&description, &format, detail, estimate, &config).await?;
        }
        Some(Commands::Analyze { path, type_ }) => {
            handle_analyze(&path, &type_, &config).await?;
        }
        Some(Commands::Review { path, criteria }) => {
            handle_review(&path, &criteria, &config).await?;
        }
        Some(Commands::Doc { target, type_ }) => {
            handle_doc(&target, &type_, &config).await?;
        }
        Some(Commands::Config { action }) => {
            handle_config(action)?;
        }
        Some(Commands::Completion { shell }) => {
            print_completions(shell, &mut Cli::command());
        }
        None => {
            // 직접 질문 모드
            if let Some(query) = cli.query {
                handle_direct_query(&query, &config).await?;
            } else {
                // 기본 대화형 모드
                handle_chat(false, None, &config).await?;
            }
        }
    }
    
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

async fn handle_chat(context: bool, save_path: Option<&str>, config: &Config) -> Result<()> {
    use rustyline::error::ReadlineError;
    use rustyline::{Editor, CompletionType, Config as RustyConfig, EditMode};
    use rustyline::completion::{Completer, FilenameCompleter, Pair};
    use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
    use rustyline::hint::{Hinter, HistoryHinter};
    use rustyline::validate::{Validator, MatchingBracketValidator};
    use rustyline::{Context as RustyContext, Helper};
    
    // 자동완성 헬퍼 구조체
    struct RicciHelper {
        completer: FilenameCompleter,
        highlighter: MatchingBracketHighlighter,
        validator: MatchingBracketValidator,
        hinter: HistoryHinter,
        commands: Vec<String>,
    }
    
    impl RicciHelper {
        fn new() -> Self {
            Self {
                completer: FilenameCompleter::new(),
                highlighter: MatchingBracketHighlighter::new(),
                validator: MatchingBracketValidator::new(),
                hinter: HistoryHinter {},
                commands: vec![
                    "/clear".to_string(),
                    "/context".to_string(),
                    "/save".to_string(),
                    "/help".to_string(),
                    "/plan".to_string(),
                    "/analyze".to_string(),
                    "/review".to_string(),
                    "/doc".to_string(),
                ],
            }
        }
    }
    
    impl Completer for RicciHelper {
        type Candidate = Pair;
        
        fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &RustyContext<'_>,
        ) -> rustyline::Result<(usize, Vec<Pair>)> {
            // 특수 명령어 자동완성
            if line.starts_with('/') {
                let mut candidates = Vec::new();
                let prefix = &line[..pos];
                
                for cmd in &self.commands {
                    if cmd.starts_with(prefix) {
                        candidates.push(Pair {
                            display: cmd.clone(),
                            replacement: cmd.clone(),
                        });
                    }
                }
                
                if !candidates.is_empty() {
                    return Ok((0, candidates));
                }
            }
            
            // 파일명 자동완성
            self.completer.complete(line, pos, ctx)
        }
    }
    
    impl Hinter for RicciHelper {
        type Hint = String;
        
        fn hint(&self, line: &str, pos: usize, ctx: &RustyContext<'_>) -> Option<String> {
            self.hinter.hint(line, pos, ctx)
        }
    }
    
    impl Highlighter for RicciHelper {
        fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
            &'s self,
            prompt: &'p str,
            default: bool,
        ) -> std::borrow::Cow<'b, str> {
            if default {
                std::borrow::Cow::Borrowed(prompt)
            } else {
                std::borrow::Cow::Owned(prompt.bright_green().bold().to_string())
            }
        }
        
        fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
            std::borrow::Cow::Owned(hint.dimmed().to_string())
        }
        
        fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
            self.highlighter.highlight(line, pos)
        }
        
        fn highlight_char(&self, line: &str, pos: usize) -> bool {
            self.highlighter.highlight_char(line, pos)
        }
    }
    
    impl Validator for RicciHelper {
        fn validate(
            &self,
            ctx: &mut rustyline::validate::ValidationContext,
        ) -> rustyline::Result<rustyline::validate::ValidationResult> {
            self.validator.validate(ctx)
        }
        
        fn validate_while_typing(&self) -> bool {
            self.validator.validate_while_typing()
        }
    }
    
    impl Helper for RicciHelper {}
    
    // Rustyline 설정
    let rusty_config = RustyConfig::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    
    let helper = RicciHelper::new();
    let mut rl = Editor::with_config(rusty_config)?;
    rl.set_helper(Some(helper));
    
    // 히스토리 파일 로드
    let history_path = dirs::data_dir()
        .map(|p| p.join("ricci").join("history.txt"));
    
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }
    
    println!("{}", "Ricci 개발 어시스턴트".bright_cyan().bold());
    println!("{}", "대화형 모드로 진입합니다. 'exit' 또는 Ctrl+C로 종료하세요.\n".dimmed());
    println!("{}", "💡 Tab 키로 자동완성을 사용할 수 있습니다.\n".yellow());
    
    let mut assistant = DevAssistant::new(config.clone())?;
    
    if context {
        println!("{}", "프로젝트 컨텍스트 로딩 중...".yellow());
        assistant.load_project_context(".").await?;
        println!("{}", "✓ 프로젝트 컨텍스트 로드 완료\n".green());
    }
    
    loop {
        let readline = rl.readline(&format!("{} ", ">".bright_green().bold()));
        
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                
                if input == "exit" || input == "quit" {
                    println!("{}", "\n대화를 종료합니다.".dimmed());
                    break;
                }
                
                // 특수 명령어 처리
                if input.starts_with('/') {
                    handle_special_command(input, &mut assistant).await?;
                    continue;
                }
                
                // AI 응답 스트리밍
                println!();
                assistant.stream_response(input).await?;
                println!("\n");
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "\n대화가 중단되었습니다.".yellow());
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "\n대화를 종료합니다.".dimmed());
                break;
            }
            Err(err) => {
                eprintln!("오류: {:?}", err);
                break;
            }
        }
    }
    
    // 히스토리 저장
    if let Some(ref path) = history_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = rl.save_history(path);
    }
    
    if let Some(path) = save_path {
        assistant.save_session(path)?;
        println!("{} {}", "세션 저장됨:".green(), path);
    }
    
    Ok(())
}

async fn handle_special_command(command: &str, assistant: &mut DevAssistant) -> Result<()> {
    match command {
        "/clear" => {
            assistant.clear_context();
            println!("{}", "컨텍스트가 초기화되었습니다.".yellow());
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
        cmd if cmd.starts_with("/doc ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.len() >= 2 {
                let target = parts[1];
                let doc_type = parts.get(2).unwrap_or(&"readme");
                println!("{} {} 문서 생성 중...", doc_type.cyan(), target);
                let doc = assistant.generate_documentation(target, doc_type).await?;
                println!("\n{}", doc);
            }
        }
        _ => {
            println!("{}", "알 수 없는 명령어입니다. /help를 입력하세요.".red());
        }
    }
    Ok(())
}

fn print_special_commands() {
    println!("{}", "\n특수 명령어:".bright_blue().bold());
    println!("  {} - 컨텍스트 초기화", "/clear".cyan());
    println!("  {} - 현재 컨텍스트 보기", "/context".cyan());
    println!("  {} - 세션 저장", "/save".cyan());
    println!("  {} - 작업계획서 템플릿", "/plan".cyan());
    println!("  {} - 프로젝트 분석", "/analyze".cyan());
    println!("  {} <path> - 코드 리뷰", "/review".cyan());
    println!("  {} <target> [type] - 문서 생성", "/doc".cyan());
    println!("  {} - 도움말\n", "/help".cyan());
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

async fn handle_plan(
    description: &str,
    format: &str,
    detail: u8,
    estimate: bool,
    config: &Config,
) -> Result<()> {
    println!("{}", "작업계획서 생성 중...".yellow());
    
    let planner = ProjectPlanner::new(config.clone())?;
    let plan = planner.create_plan(description, detail, estimate).await?;
    
    match format {
        "markdown" => {
            println!("\n{}", plan.to_markdown());
        }
        "json" => {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&plan)?);
        }
        _ => {
            anyhow::bail!("지원하지 않는 형식: {}", format);
        }
    }
    
    Ok(())
}

async fn handle_analyze(path: &str, type_: &str, config: &Config) -> Result<()> {
    println!("{} {}", "분석 중:".yellow(), path);
    
    let analyzer = CodeAnalyzer::new(config.clone())?;
    
    match type_ {
        "structure" => {
            let structure = analyzer.analyze_structure(path).await?;
            analyzer.print_structure_report(&structure);
        }
        "dependencies" => {
            let deps = analyzer.analyze_dependencies(path).await?;
            analyzer.print_dependency_report(&deps);
        }
        "complexity" => {
            let complexity = analyzer.analyze_complexity(path).await?;
            analyzer.print_complexity_report(&complexity);
        }
        "all" => {
            let report = analyzer.analyze_all(path).await?;
            analyzer.print_full_report(&report);
        }
        _ => {
            anyhow::bail!("지원하지 않는 분석 유형: {}", type_);
        }
    }
    
    Ok(())
}

async fn handle_review(path: &str, criteria: &str, config: &Config) -> Result<()> {
    println!("{} {}", "코드 리뷰 중:".yellow(), path);
    
    let assistant = DevAssistant::new(config.clone())?;
    let review = assistant.review_code(path, criteria).await?;
    
    println!("\n{}", review.format_markdown());
    
    Ok(())
}

async fn handle_doc(target: &str, type_: &str, config: &Config) -> Result<()> {
    println!("{} {} 문서 생성 중...", type_.cyan(), target);
    
    let assistant = DevAssistant::new(config.clone())?;
    let doc = assistant.generate_documentation(target, type_).await?;
    
    println!("\n{}", doc);
    
    Ok(())
}

fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::SetKey { provider, key } => {
            Config::set_api_key(&provider, &key)?;
            println!("{} API 키가 설정되었습니다.", provider.green());
        }
        ConfigAction::Show => {
            let config = Config::load()?;
            println!("{}", config.display());
        }
        ConfigAction::Reset => {
            Config::reset()?;
            println!("{}", "설정이 초기화되었습니다.".yellow());
        }
    }
    Ok(())
}

async fn handle_direct_query(query: &str, config: &Config) -> Result<()> {
    let assistant = DevAssistant::new(config.clone())?;
    assistant.stream_response(query).await?;
    println!();
    Ok(())
}
