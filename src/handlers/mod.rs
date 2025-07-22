pub mod chat;
pub mod command;
pub mod analysis;
pub mod completion;
pub mod config;

// Re-export main handler functions
pub use chat::handle_chat;
pub use command::handle_special_command;
pub use analysis::{handle_analyze, handle_review, handle_doc, handle_plan};
pub use completion::install_completions;
pub use config::handle_config;

use anyhow::Result;
use crate::config::Config;
use crate::assistant::DevAssistant;

// 공통 유틸리티 함수들
pub async fn handle_direct_query(query: &str, config: &Config) -> Result<()> {
    let mut assistant = DevAssistant::new(config.clone())?;
    assistant.stream_response(query).await?;
    println!();
    Ok(())
} 