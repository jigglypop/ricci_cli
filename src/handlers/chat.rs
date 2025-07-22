use anyhow::Result;
use colored::*;
use std::io::{BufReader, BufRead};
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
                            cmd if cmd.starts_with('/') => {
                                super::command::handle_special_command(cmd, &mut assistant).await?;
                                continue;
                            }
                            _ => { // 셸 명령어 실행
                                execute_shell_command(input)?;
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
    
    if let Some(path) = save_path {
        assistant.save_session(path)?;
        println!("{} {}", "세션 저장됨:".green(), path);
    }
    
    Ok(())
}

fn execute_shell_command(input: &str) -> Result<()> {
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
    Ok(())
} 