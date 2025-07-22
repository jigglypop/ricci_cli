use anyhow::Result;
use colored::*;
use crate::config::Config;

#[derive(clap::Subcommand)]
pub enum ConfigAction {
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

pub fn handle_config(action: ConfigAction) -> Result<()> {
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