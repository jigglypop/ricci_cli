mod types;
mod file_modifier;
mod review;

pub use types::*;
pub use file_modifier::{FileModifier, FileChange, SafeFileModifier};
pub use review::review_code;

use anyhow::{Result, Context};
use crate::config::Config;
use crate::api::OpenAIClient;
use crate::renderer::MarkdownRenderer;
use std::path::Path;
use colored::*;
use chrono::Utc;
use rustyline::Editor;
use rustyline::error::ReadlineError;

pub struct DevAssistant {
    client: OpenAIClient,
    renderer: MarkdownRenderer,
    context: AssistantContext,
    config: Config,
    chat_mode: ChatMode,
}

impl DevAssistant {
    pub fn new(config: Config) -> Result<Self> {
        let client = OpenAIClient::new(&config)?;
        let renderer = MarkdownRenderer::new();
        
        Ok(Self {
            client,
            renderer,
            context: AssistantContext::default(),
            config,
            chat_mode: ChatMode::Normal,
        })
    }
    
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    
    pub fn set_mode(&mut self, mode: ChatMode) {
        self.chat_mode = mode;
    }
    
    pub fn get_mode(&self) -> ChatMode {
        self.chat_mode
    }
    
    pub fn add_context_file(&mut self, file_path: &str) -> Result<()> {
        if !self.context.current_files.contains(&file_path.to_string()) {
            self.context.current_files.push(file_path.to_string());
        }
        Ok(())
    }
    
    pub fn clear_context(&mut self) {
        self.context.messages.clear();
        self.context.current_files.clear();
    }
    
    pub async fn generate_documentation(&self, target: &str, doc_type: &str) -> Result<String> {
        let prompt = self.build_doc_prompt(target, doc_type)?;
        self.client.query(&prompt).await
    }
    
    fn build_doc_prompt(&self, target: &str, doc_type: &str) -> Result<String> {
        let content = if Path::new(target).exists() {
            std::fs::read_to_string(target)?
        } else {
            target.to_string()
        };
        
        let prompt = match doc_type {
            "api" => format!(
                "다음 코드에 대한 API 문서를 작성해주세요:\n\n{}\n\n\
                각 public 함수/메서드에 대해 설명, 매개변수, 반환값, 예제를 포함해주세요.",
                content
            ),
            "readme" => format!(
                "다음 프로젝트/코드에 대한 README.md를 작성해주세요:\n\n{}\n\n\
                프로젝트 설명, 설치 방법, 사용법, 예제를 포함해주세요.",
                content
            ),
            "tutorial" => format!(
                "다음 코드를 사용하는 방법에 대한 튜토리얼을 작성해주세요:\n\n{}\n\n\
                단계별 설명과 실제 사용 예제를 포함해주세요.",
                content
            ),
            _ => format!("다음에 대한 {} 문서를 작성해주세요:\n\n{}", doc_type, content),
        };
        
        Ok(prompt)
    }
    
    pub async fn chat_interactive(&mut self) -> Result<()> {
        println!("{}", "대화형 모드를 시작합니다. 'exit'를 입력하면 종료됩니다.".bright_cyan());
        println!("{}", "명령어: /clear, /mode [normal|concise|detailed|code], /save [파일명]".dimmed());
        
        let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()?;
        let history_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ricci")
            .join("history.txt");
        
        let _ = rl.load_history(&history_path);
        
        loop {
            let prompt = format!("{} ", "You:".green().bold());
            
            match rl.readline(&prompt) {
                Ok(input) => {
                    let input = input.trim();
                    
                    if input.is_empty() {
                        continue;
                    }
                    
                    let _ = rl.add_history_entry(input);
                    
                    if input == "exit" {
                        break;
                    }
                    
                    if let Some(command) = input.strip_prefix('/') {
                        self.handle_command(command)?;
                        continue;
                    }
                    
                    self.add_message("user", input);
                    
                    println!("\n{} ", "Assistant:".blue().bold());
                    
                    let system_prompt = self.get_system_prompt();
                    let mut stream = self.client.stream_chat(&system_prompt, &self.context.messages).await?;
                    
                    let mut response = String::new();
                    while let Some(chunk) = stream.recv().await {
                        match chunk {
                            Ok(text) => {
                                response.push_str(&text);
                                self.renderer.render_chunk(&text)?;
                            }
                            Err(e) => {
                                eprintln!("\n{}: {}", "스트림 오류".red(), e);
                                break;
                            }
                        }
                    }
                    
                    println!("\n");
                    self.add_message("assistant", &response);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("\n{}", "중단됨. 계속하려면 Enter를 누르세요.".yellow());
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("\n{}", "종료합니다.".yellow());
                    break;
                }
                Err(err) => {
                    eprintln!("{}: {:?}", "입력 오류".red(), err);
                    break;
                }
            }
        }
        
        let _ = rl.save_history(&history_path);
        Ok(())
    }
    
    fn handle_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0).map(|s| *s) {
            Some("clear") => {
                self.clear_context();
                println!("{}", "대화 기록이 삭제되었습니다.".green());
            }
            Some("mode") => {
                if let Some(mode_str) = parts.get(1) {
                    let mode = match *mode_str {
                        "normal" => ChatMode::Normal,
                        "concise" => ChatMode::Concise,
                        "detailed" => ChatMode::Detailed,
                        "code" => ChatMode::Code,
                        "planning" => ChatMode::Planning,
                        _ => {
                            println!("{}", "알 수 없는 모드입니다.".red());
                            return Ok(());
                        }
                    };
                    self.set_mode(mode);
                    println!("{} {:?}", "모드 변경:".green(), mode);
                } else {
                    println!("{} {:?}", "현재 모드:".blue(), self.chat_mode);
                }
            }
            Some("save") => {
                let filename = parts.get(1).unwrap_or(&"chat_history.md");
                self.save_conversation(filename)?;
            }
            _ => {
                println!("{}", "알 수 없는 명령어입니다.".red());
            }
        }
        
        Ok(())
    }
    
    fn get_system_prompt(&self) -> String {
        match self.chat_mode {
            ChatMode::Normal => "You are a helpful development assistant.".to_string(),
            ChatMode::Concise => "You are a concise assistant. Keep responses brief and to the point.".to_string(),
            ChatMode::Detailed => "You are a detailed assistant. Provide comprehensive explanations with examples.".to_string(),
            ChatMode::Code => "You are a code-focused assistant. Prioritize code examples and technical details.".to_string(),
            ChatMode::Planning => "You are a project planning assistant. Focus on architecture, design, and planning.".to_string(),
        }
    }
    
    fn add_message(&mut self, role: &str, content: &str) {
        self.context.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }
    
    fn save_conversation(&self, filename: &str) -> Result<()> {
        let mut content = String::new();
        content.push_str(&format!("# 대화 기록\n\n"));
        content.push_str(&format!("생성일: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S")));
        
        for msg in &self.context.messages {
            content.push_str(&format!("## {} ({})\n\n", 
                msg.role.to_uppercase(), 
                msg.timestamp.format("%H:%M:%S")
            ));
            content.push_str(&format!("{}\n\n", msg.content));
        }
        
        std::fs::write(filename, content)?;
        println!("{} {}", "대화 내용이 저장되었습니다:".green(), filename);
        Ok(())
    }
    
    pub async fn query(&self, prompt: &str) -> Result<String> {
        self.client.query(prompt).await
    }
    
    pub async fn review_code(&self, path: &str, criteria: &str) -> Result<CodeReview> {
        review_code(&self.client, path, criteria).await
    }
    
    pub async fn apply_code_suggestions(&self, suggestions: Vec<CodeSuggestion>) -> Result<()> {
        let modifier = FileModifier::new(false);
        
        let changes: Vec<FileChange> = suggestions
            .into_iter()
            .map(|s| FileChange {
                path: s.file_path,
                original_content: s.original_code,
                new_content: s.suggested_code,
                description: s.reason,
            })
            .collect();
        
        modifier.apply_changes(changes).await?;
        Ok(())
    }
    
    pub async fn safe_modify_files(&self, changes: Vec<FileChange>) -> Result<()> {
        let safe_modifier = SafeFileModifier::new(false);
        safe_modifier.modify_with_backup(changes).await
    }
    
    async fn analyze_project(&self, path: &str) -> Result<ProjectInfo> {
        let prompt = format!(
            "다음 프로젝트 구조를 분석하고 주요 정보를 추출해주세요:\n{}\n\n\
            JSON 형식으로 응답해주세요: {{\"name\": \"\", \"language\": \"\", \"framework\": \"\", \"dependencies\": [], \"structure\": \"\"}}",
            path
        );
        
        let response = self.client.query(&prompt).await?;
        let info: ProjectInfo = serde_json::from_str(&response)
            .context("프로젝트 정보 파싱 실패")?;
        
        Ok(info)
    }
    
    pub async fn stream_response(&mut self, query: &str) -> Result<()> {
        self.add_message("user", query);
        let system_prompt = self.get_system_prompt();
        let mut stream = self.client.stream_chat(&system_prompt, &self.context.messages).await?;
        
        let mut response = String::new();
        while let Some(chunk) = stream.recv().await {
            match chunk {
                Ok(text) => {
                    response.push_str(&text);
                    self.renderer.render_chunk(&text)?;
                }
                Err(e) => {
                    eprintln!("\n{}: {}", "스트림 오류".red(), e);
                    break;
                }
            }
        }
        
        self.add_message("assistant", &response);
        Ok(())
    }
    
    pub async fn load_project_context(&mut self, path: &str) -> Result<()> {
        let project_info = self.analyze_project(path).await?;
        self.context.project_info = Some(project_info);
        Ok(())
    }
    
    pub fn save_session(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.context)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn get_context_summary(&self) -> String {
        let mut summary = String::new();
        
        if let Some(ref info) = self.context.project_info {
            summary.push_str(&format!("프로젝트: {} ({})\n", info.name, info.language));
            if let Some(ref framework) = info.framework {
                summary.push_str(&format!("프레임워크: {}\n", framework));
            }
        }
        
        summary.push_str(&format!("대화 기록: {} 개\n", self.context.messages.len()));
        summary
    }
    
    pub async fn export_as_plan(&self, format: &str) -> Result<String> {
        let mut content = String::new();
        
        for msg in &self.context.messages {
            content.push_str(&format!("{}: {}\n\n", msg.role, msg.content));
        }
        
        match format {
            "markdown" => Ok(format!("# 작업 계획\n\n{}", content)),
            _ => Ok(content),
        }
    }
    
    pub fn set_chat_mode(&mut self, mode: ChatMode) {
        self.set_mode(mode);
    }
} 