use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use colored::*;
use anyhow::{Result, Context};
use std::io::{self, Write, BufReader, BufRead};
use std::process::{Command, Stdio};
use ricci_cli::{
    assistant::DevAssistant,
    planner::ProjectPlanner,
    analyzer::CodeAnalyzer,
    config::Config,
    splash::{display_splash},
};
use rustyline::error::ReadlineError;
use rustyline::{Editor, CompletionType, Config as RustyConfig, EditMode, Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{Validator, MatchingBracketValidator};
use rustyline::{Context as RustyContext, Helper};

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
    
    /// 자동완성 설치
    Install {
        /// 대상 쉘 (자동 감지하려면 비워두세요)
        #[clap(value_enum)]
        shell: Option<Shell>,
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

#[derive(Clone, Copy, PartialEq, Debug)]
enum AppMode {
    Command,
    Chat,
}

#[tokio::main]
async fn main() -> Result<()> {
    // .env 파일 로드
    dotenv::dotenv().ok();
    
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
        Some(Commands::Install { shell }) => {
            install_completions(shell)?;
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
    // 자동완성 헬퍼 구조체
    struct RicciHelper {
        completer: FilenameCompleter,
        highlighter: MatchingBracketHighlighter,
        validator: MatchingBracketValidator,
        hinter: HistoryHinter, // 표준 히스토리 힌터 사용
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
                    "/clear", "/context", "/save", "/help", "/plan", 
                    "/analyze", "/review", "/doc", "/new", "/cls", 
                    "/mode", "/summary", "/chat",
                ].into_iter().map(String::from).collect(),
            }
        }
    }

    impl Completer for RicciHelper {
        type Candidate = Pair;

        fn complete( &self, line: &str, pos: usize, ctx: &RustyContext<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
            if line.starts_with('/') {
                let mut matches = Vec::new();
                for cmd in &self.commands {
                    if cmd.starts_with(line) {
                        matches.push(Pair {
                            display: cmd.clone(),
                            replacement: cmd.clone(),
                        });
                    }
                }
                return Ok((0, matches));
            }
            self.completer.complete(line, pos, ctx)
        }
    }

    impl Hinter for RicciHelper {
        type Hint = String;
        fn hint(&self, line: &str, pos: usize, ctx: &RustyContext<'_>) -> Option<String> {
            if pos < line.len() { return None; }

            // 명령어 힌트
            if line.starts_with('/') {
                for cmd in &self.commands {
                    if cmd.starts_with(line) && cmd.len() > line.len() {
                        return Some(cmd[pos..].to_string());
                    }
                }
            }
            
            // 그 외에는 히스토리 기반 힌트
            self.hinter.hint(line, pos, ctx)
        }
    }

    impl Highlighter for RicciHelper {
        fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _default: bool) -> std::borrow::Cow<'b, str> {
            if prompt == "ricci (chat)> " {
                std::borrow::Cow::Owned(format!("{} {}", "ricci".bright_blue().bold(), "(chat)>".yellow()))
            } else {
                std::borrow::Cow::Owned(prompt.bright_blue().bold().to_string())
            }
        }

        fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
            std::borrow::Cow::Owned(hint.dimmed().to_string())
        }

        fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
            self.highlighter.highlight(line, pos)
        }

        fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
            self.highlighter.highlight_char(line, pos, forced)
        }
    }

    impl Validator for RicciHelper {
        fn validate( &self, ctx: &mut rustyline::validate::ValidationContext, ) -> rustyline::Result<rustyline::validate::ValidationResult> {
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
    rl.bind_sequence( // Tab 키는 힌트 완성
        KeyEvent::from('\t'),
        EventHandler::Simple(Cmd::CompleteHint),
    );
    rl.bind_sequence( // 오른쪽 화살표도 힌트 완성
        KeyEvent(KeyCode::Right, Modifiers::NONE),
        EventHandler::Simple(Cmd::CompleteHint),
    );
    rl.bind_sequence( // Ctrl+I는 목록 표시
        KeyEvent(KeyCode::Char('i'), Modifiers::CTRL),
        EventHandler::Simple(Cmd::Complete),
    );
    
    // 히스토리 파일 로드
    let history_path = dirs::data_dir()
        .map(|p| p.join("ricci").join("history.txt"));
    
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }
    
    // Splash 화면 표시
    display_splash()?;
    
    let mut assistant = DevAssistant::new(config.clone())?;
    
    if context {
        println!("{}", "프로젝트 컨텍스트 로딩 중...".yellow());
        assistant.load_project_context(".").await?;
        println!("{}", "✓ 프로젝트 컨텍스트 로드 완료\n".green());
    }
    
    let mut mode = AppMode::Command;

    loop {
        let prompt = match mode {
            AppMode::Command => "ricci> ",
            AppMode::Chat => "ricci (chat)> ",
        };

        let readline = rl.readline(prompt);
        
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                match mode {
                    AppMode::Command => {
                        // 단축키 및 명령어 처리
                        match input {
                            "c" | "chat" | "/chat" => {
                                mode = AppMode::Chat;
                                println!("{}", "대화 모드로 전환합니다. 'exit' 또는 'quit'으로 나올 수 있습니다.".green());
                                continue;
                            }
                            "h" | "/help" => {
                                print_special_commands();
                                continue;
                            }
                            "p" | "/summary" => {
                                handle_special_command("/summary", &mut assistant).await?;
                                continue;
                            }
                            cmd if cmd.starts_with('/') => {
                                handle_special_command(cmd, &mut assistant).await?;
                                continue;
                            }
                            _ => { // 셸 명령어 실행
                                println!("{} {}", "❯ Executing:".dimmed(), input);
                                let mut command = if cfg!(target_os = "windows") {
                                    let mut com = Command::new("cmd");
                                    com.arg("/C").arg(input);
                                    com
                                } else {
                                    let mut com = Command::new("sh");
                                    com.arg("-c").arg(input);
                                    com
                                };

                                let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
                                
                                if let Some(stdout) = child.stdout.take() {
                                    let reader = BufReader::new(stdout);
                                    for line in reader.lines() {
                                        println!("{}", line?);
                                    }
                                }
                                if let Some(stderr) = child.stderr.take() {
                                    let reader = BufReader::new(stderr);
                                    for line in reader.lines() {
                                        eprintln!("{}", line?.yellow());
                                    }
                                }

                                child.wait()?;
                            }
                        }
                    }
                    AppMode::Chat => {
                        if input == "exit" || input == "quit" {
                            mode = AppMode::Command;
                            println!("{}", "명령어 모드로 돌아갑니다.".yellow());
                            continue;
                        }
                        assistant.stream_response(input).await?;
                    }
                }
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
        "/cls" | "/new" => {
            // 화면 초기화
            print!("\x1B[2J\x1B[1;1H");
            std::io::stdout().flush()?;
            ricci_cli::splash::display_mini_splash();
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
            use ricci_cli::assistant::ChatMode;
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
    let mut assistant = DevAssistant::new(config.clone())?;
    assistant.stream_response(query).await?;
    println!();
    Ok(())
}

fn install_completions(shell: Option<Shell>) -> Result<()> {
    // 쉘 자동 감지
    let detected_shell = if let Some(shell) = shell {
        shell
    } else {
        detect_shell()?
    };
    
    println!("{} {}", 
        "자동완성 설치 중:".bright_green(), 
        format!("{:?}", detected_shell).cyan()
    );
    
    // 완성 스크립트 생성
    let mut cmd = Cli::command();
    let mut script = Vec::new();
    generate(detected_shell, &mut cmd, "ricci", &mut script);
    let script_content = String::from_utf8(script)?;
    
    // 설치 경로 결정
    match detected_shell {
        Shell::Bash => install_bash_completion(&script_content)?,
        Shell::Zsh => install_zsh_completion(&script_content)?,
        Shell::PowerShell => install_powershell_completion(&script_content)?,
        Shell::Fish => install_fish_completion(&script_content)?,
        _ => anyhow::bail!("지원하지 않는 쉘입니다: {:?}", detected_shell),
    }
    
    println!("{}", "✓ 자동완성 설치 완료!".green().bold());
    println!("\n다음 중 하나를 실행하여 적용하세요:");
    
    match detected_shell {
        Shell::Bash => println!("  source ~/.bashrc"),
        Shell::Zsh => println!("  source ~/.zshrc"),
        Shell::PowerShell => println!("  . $PROFILE"),
        Shell::Fish => println!("  source ~/.config/fish/config.fish"),
        _ => {}
    }
    
    println!("\n{}", "사용 예시:".yellow());
    println!("  ricci <Tab>        # 사용 가능한 명력어 보기");
    println!("  ricci plan <Tab>   # plan 옵션 보기");
    
    Ok(())
}

fn detect_shell() -> Result<Shell> {
    // Windows
    if cfg!(windows) {
        return Ok(Shell::PowerShell);
    }
    
    // Unix-like systems
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("bash") {
            return Ok(Shell::Bash);
        } else if shell.contains("zsh") {
            return Ok(Shell::Zsh);
        } else if shell.contains("fish") {
            return Ok(Shell::Fish);
        }
    }
    
    // 기본값
    Ok(Shell::Bash)
}

fn install_bash_completion(script: &str) -> Result<()> {
    let home = dirs::home_dir().context("홈 디렉토리를 찾을 수 없습니다")?;
    let completion_dir = home.join(".local").join("share").join("bash-completion").join("completions");
    std::fs::create_dir_all(&completion_dir)?;
    
    let completion_file = completion_dir.join("ricci");
    std::fs::write(&completion_file, script)?;
    
    // .bashrc에 추가
    let bashrc = home.join(".bashrc");
    if bashrc.exists() {
        let content = std::fs::read_to_string(&bashrc)?;
        if !content.contains("bash-completion/completions") {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&bashrc)?;
            writeln!(file, "\n# Ricci CLI 자동완성")?;
            writeln!(file, "[ -f ~/.local/share/bash-completion/completions/ricci ] && source ~/.local/share/bash-completion/completions/ricci")?;
        }
    }
    
    Ok(())
}

fn install_zsh_completion(script: &str) -> Result<()> {
    let home = dirs::home_dir().context("홈 디렉토리를 찾을 수 없습니다")?;
    let completion_dir = home.join(".local").join("share").join("zsh").join("completions");
    std::fs::create_dir_all(&completion_dir)?;
    
    let completion_file = completion_dir.join("_ricci");
    std::fs::write(&completion_file, script)?;
    
    // .zshrc에 fpath 추가
    let zshrc = home.join(".zshrc");
    if zshrc.exists() {
        let content = std::fs::read_to_string(&zshrc)?;
        if !content.contains(".local/share/zsh/completions") {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&zshrc)?;
            writeln!(file, "\n# Ricci CLI 자동완성")?;
            writeln!(file, "fpath=(~/.local/share/zsh/completions $fpath)")?;
            writeln!(file, "autoload -Uz compinit && compinit")?;
        }
    }
    
    Ok(())
}

fn install_powershell_completion(script: &str) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("설정 디렉토리를 찾을 수 없습니다")?
        .join("ricci");
    std::fs::create_dir_all(&config_dir)?;
    
    let completion_file = config_dir.join("ricci-completion.ps1");
    std::fs::write(&completion_file, script)?;
    
    // PowerShell 프로필에 추가
    if let Ok(profile) = std::env::var("PROFILE") {
        let profile_path = std::path::Path::new(&profile);
        if let Some(parent) = profile_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        if profile_path.exists() {
            let content = std::fs::read_to_string(&profile_path)?;
            let import_line = format!(". \"{}\"", completion_file.display());
            
            if !content.contains(&import_line) {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&profile_path)?;
                writeln!(file, "\n# Ricci CLI 자동완성")?;
                writeln!(file, "{}", import_line)?;
            }
        }
    }
    
    Ok(())
}

fn install_fish_completion(script: &str) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("설정 디렉토리를 찾을 수 없습니다")?
        .join("fish")
        .join("completions");
    std::fs::create_dir_all(&config_dir)?;
    
    let completion_file = config_dir.join("ricci.fish");
    std::fs::write(&completion_file, script)?;
    
    Ok(())
}
