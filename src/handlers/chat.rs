use anyhow::Result;
use colored::*;

use std::process::{Command, Stdio};
use crate::{
    assistant::DevAssistant,
    config::Config,
    splash::display_splash,
};
use rustyline::error::ReadlineError;
use rustyline::{Editor, CompletionType, Config as RustyConfig, EditMode, Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{Validator, MatchingBracketValidator};
use rustyline::{Context as RustyContext, Helper};

#[derive(Clone, Copy, PartialEq, Debug)]
enum AppMode {
    Command,
    Chat,
}

pub async fn handle_chat(context: bool, save_path: Option<&str>, config: &Config) -> Result<()> {
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
                    "/clear", "/context", "/save", "/help", "/plan", 
                    "/analyze", "/review", "/doc", "/new", "/cls", 
                    "/mode", "/summary", "/chat",
                ].into_iter().map(String::from).collect(),
            }
        }
    }

    impl Completer for RicciHelper {
        type Candidate = Pair;

        fn complete(&self, line: &str, pos: usize, ctx: &RustyContext<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
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
        fn validate(&self, ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
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
    rl.bind_sequence(
        KeyEvent::from('\t'),
        EventHandler::Simple(Cmd::CompleteHint),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Right, Modifiers::NONE),
        EventHandler::Simple(Cmd::CompleteHint),
    );
    rl.bind_sequence(
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
    
    // 이전 세션 로드 시도
    assistant.load_session().await.ok();
    
    // 컨텍스트 파일 로드
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
                                super::command::print_special_commands();
                                continue;
                            }
                            "p" | "/summary" => {
                                super::command::handle_special_command("/summary", &mut assistant).await?;
                                continue;
                            }
                            // 한글 명령어 처리
                            "폴더분석" | "폴더 분석" | "구조분석" | "구조 분석" => {
                                println!("{}", "📁 현재 폴더 구조를 분석합니다...".green());
                                super::handle_analyze(".", "structure", config).await?;
                                continue;
                            }
                            "파일분석" | "파일 분석" | "코드분석" | "코드 분석" => {
                                println!("{}", "📝 파일 경로를 입력하세요 (예: src/main.rs 또는 . 전체):".cyan());
                                if let Ok(file_path) = rl.readline("파일 경로> ") {
                                    let file_path = file_path.trim();
                                    if !file_path.is_empty() {
                                        super::run_code_assistant_interactive(file_path, &mut assistant, config).await?;
                                    }
                                }
                                continue;
                            }
                            "하위폴더 코드분석" | "하위폴더 분석" | "전체 코드분석" | "전체 코드 분석" => {
                                println!("{}", "📂 하위 폴더의 모든 코드를 분석합니다...".green());
                                super::handle_folder_code_analysis(".", &mut assistant, config).await?;
                                continue;
                            }
                            "작업계획서" | "계획서" | "작업정리" | "작업 정리" => {
                                println!("{}", "📋 대화 내용을 작업계획서로 정리합니다...".green());
                                super::command::handle_special_command("/summary", &mut assistant).await?;
                                continue;
                            }
                            cmd if cmd.starts_with('/') => {
                                super::command::handle_special_command(cmd, &mut assistant).await?;
                                continue;
                            }
                            _ => { // 셸 명령어 실행
                                // 한글 명령어를 직접 처리
                                match input {
                                    "안녕" | "하이" | "헬로" => {
                                        println!("안녕하세요! 무엇을 도와드릴까요? 🙂");
                                        continue;
                                    }
                                    _ => {
                                        // ?나 @로 시작하면 AI와 대화
                                        if input.starts_with('?') || input.starts_with('@') {
                                            let query = input.trim_start_matches(['?', '@']).trim();
                                            if !query.is_empty() {
                                                assistant.stream_response(query).await?;
                                            }
                                        } else {
                                            execute_shell_command(input)?
                                        }
                                    }
                                }
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
                eprintln!("오류: {err:?}");
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
    
    // 세션 자동 저장
    assistant.save_session().await?;
    
    if let Some(path) = save_path {
        assistant.save_conversation(path)?;
        println!("{} {}", "대화 내용 저장됨:".green(), path);
    }
    
    Ok(())
}

fn execute_shell_command(input: &str) -> Result<()> {

    
    // 한글 명령어 처리
    let processed_input = match input {
        "해당 하위 폴더구조 분석좀" | "폴더 분석" | "구조 분석" => {
            println!("{}", "📁 현재 폴더 구조를 분석합니다...".green());
            "ricci analyze ."
        }
        "파일 분석" | "코드 분석" => {
            println!("{}", "📝 코드 분석 모드로 전환합니다. 파일 경로를 입력하세요...".green());
            return Ok(());
        }
        "작업계획서" | "계획서 작성" | "작업 정리" => {
            println!("{}", "📋 대화 내용을 작업계획서로 정리합니다...".green());
            "ricci plan \"현재 대화 내용 정리\""
        }
        _ => input,
    };

    println!("{} {}", "❯ Executing:".dimmed(), processed_input);
    
    // Windows에서는 PowerShell을 사용하여 UTF-8 처리 개선
    let mut command = if cfg!(target_os = "windows") {
        let mut com = Command::new("powershell");
        com.arg("-NoProfile")
            .arg("-Command")
            .arg(&format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; {}", processed_input));
        com
    } else {
        let mut com = Command::new("sh");
        com.arg("-c").arg(processed_input);
        com
    };

    // 표준 입출력 설정
    command.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    match command.output() {
        Ok(output) => {
            // stdout 출력
            if !output.stdout.is_empty() {
                match String::from_utf8(output.stdout.clone()) {
                    Ok(text) => print!("{}", text),
                    Err(_) => {
                        // UTF-8 실패 시 Windows 기본 인코딩 시도
                        if cfg!(target_os = "windows") {
                            // CP949 (Korean Windows) 디코딩 시도
                            let (text, _, _) = encoding_rs::EUC_KR.decode(&output.stdout);
                            print!("{}", text);
                        } else {
                            println!("{}", "출력을 디코딩할 수 없습니다".yellow());
                        }
                    }
                }
            }
            
            // stderr 출력
            if !output.stderr.is_empty() {
                match String::from_utf8(output.stderr.clone()) {
                    Ok(text) => eprint!("{}", text.yellow()),
                    Err(_) => {
                        if cfg!(target_os = "windows") {
                            let (text, _, _) = encoding_rs::EUC_KR.decode(&output.stderr);
                            eprint!("{}", text.yellow());
                        }
                    }
                }
            }
            
            // 종료 코드 확인
            if !output.status.success() {
                if let Some(code) = output.status.code() {
                    eprintln!("{} {}", "명령어 실행 실패. 종료 코드:".red(), code);
                }
            }
        }
        Err(e) => {
            eprintln!("{} {}", "명령어 실행 오류:".red(), e);
        }
    }
    
    Ok(())
} 