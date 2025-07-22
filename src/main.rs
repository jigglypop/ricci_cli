use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use anyhow::Result;
use std::io;
use ricci_cli::{
    config::Config,
    cli::{Cli, Commands},
    handlers::{
        handle_chat, handle_analyze, handle_review, handle_doc, 
        handle_plan, handle_config, handle_direct_query, install_completions
    },
};
use colored::*;

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
        Some(Commands::CodeAssist { path, fix, test, docs }) => {
            let mut assistant = ricci_cli::assistant::DevAssistant::new(config.clone())?;
            
            // 커맨드라인 옵션이 있으면 직접 처리
            if fix || test || docs {
                println!("{}", "🚀 코드 어시스턴트 직접 모드".bright_cyan().bold());
                // TODO: 직접 실행 로직 구현
            } else {
                // 인터랙티브 모드
                ricci_cli::handlers::run_code_assistant_interactive(&path, &mut assistant, &config).await?;
            }
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

pub fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
