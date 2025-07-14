use anyhow::{Result, Context};
use crate::config::Config;
use crate::api::OpenAIClient;
use crate::renderer::MarkdownRenderer;
use serde::{Serialize, Deserialize};
use std::path::Path;
use colored::*;

pub struct DevAssistant {
    client: OpenAIClient,
    renderer: MarkdownRenderer,
    context: AssistantContext,
    config: Config,
    chat_mode: ChatMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatMode {
    Normal,     // 일반 대화
    Concise,    // 간결한 응답
    Detailed,   // 상세한 응답
    Code,       // 코드 중심
    Planning,   // 계획 수립 모드
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssistantContext {
    messages: Vec<Message>,
    project_info: Option<ProjectInfo>,
    current_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectInfo {
    name: String,
    language: String,
    framework: Option<String>,
    dependencies: Vec<String>,
    structure: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeReview {
    pub overall_score: f32,
    pub issues: Vec<ReviewIssue>,
    pub suggestions: Vec<String>,
    pub positive_aspects: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub location: String,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    Style,
    BestPractice,
    Bug,
    Documentation,
}

impl DevAssistant {
    pub fn new(config: Config) -> Result<Self> {
        let client = OpenAIClient::new(&config)?;
        let renderer = MarkdownRenderer::new();
        
        Ok(Self {
            client,
            renderer,
            context: AssistantContext {
                messages: Vec::new(),
                project_info: None,
                current_files: Vec::new(),
            },
            config,
            chat_mode: ChatMode::Normal,
        })
    }
    
    pub fn set_chat_mode(&mut self, mode: ChatMode) {
        self.chat_mode = mode;
    }
    
    pub fn get_chat_mode(&self) -> ChatMode {
        self.chat_mode
    }
    
    pub async fn summarize_conversation(&self) -> Result<String> {
        if self.context.messages.is_empty() {
            return Ok("대화 내용이 없습니다.".to_string());
        }
        
        let conversation = self.context.messages.iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let prompt = format!(
            "다음 대화 내용을 요약하고 작업계획서로 정리해주세요:\n\n{}\n\n\
            다음 형식으로 작성해주세요:\n\
            1. 논의된 주요 작업\n\
            2. 구현 우선순위\n\
            3. 기술적 고려사항\n\
            4. 예상 일정",
            conversation
        );
        
        self.client.query(&prompt).await
    }
    
    pub async fn export_as_plan(&self, format: &str) -> Result<String> {
        let summary = self.summarize_conversation().await?;
        
        match format {
            "markdown" => Ok(format!("# 작업계획서\n\n{}", summary)),
            "json" => {
                let plan = serde_json::json!({
                    "title": "대화 기반 작업계획서",
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "content": summary,
                    "message_count": self.context.messages.len(),
                });
                Ok(serde_json::to_string_pretty(&plan)?)
            }
            _ => Ok(summary),
        }
    }
    
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    
    pub async fn stream_response(&mut self, query: &str) -> Result<()> {
        use colored::*;
        use std::io::Write;
        
        // 사용자 입력 표시
        println!("\n{} {}", "나 :".bright_cyan().bold(), query.white());
        println!("{}", "─".repeat(80).bright_black());
        
        // 사용자 메시지 추가
        self.add_message("user", query);
        
        // 시스템 프롬프트 생성
        let system_prompt = self.create_system_prompt();
        
        // AI 응답 시작 표시
        println!("{} {}", "Ricci 봇:".bright_green().bold(), "Thinking...".dimmed());
        print!("\r{} ", "Ricci: 봇".bright_green().bold());
        std::io::stdout().flush()?;
        
        // 스트리밍 응답 받기
        let mut stream = self.client.stream_chat(&system_prompt, &self.context.messages).await?;
        
        let mut response = String::new();
        let mut buffer = String::new();
        let mut first_chunk = true;
        
        // 스트리밍 출력
        while let Some(chunk) = stream.recv().await {
            match chunk {
                Ok(content) => {
                    if first_chunk {
                        // "Thinking..." 텍스트를 지우고 시작
                        print!("\r{} ", "Ricci 봇:".bright_green().bold());
                        first_chunk = false;
                    }
                    
                    buffer.push_str(&content);
                    response.push_str(&content);
                    
                    // 마크다운 블록이 완성되면 렌더링
                    if self.should_render(&buffer) {
                        self.renderer.render_chunk(&buffer)?;
                        buffer.clear();
                    }
                }
                Err(e) => {
                    eprintln!("\n스트리밍 오류: {}", e);
                    break;
                }
            }
        }
        
        // 남은 버퍼 렌더링
        if !buffer.is_empty() {
            self.renderer.render_chunk(&buffer)?;
        }
        
        // 응답 끝 구분선
        println!("\n{}", "─".repeat(80).bright_black());
        
        // 어시스턴트 응답 저장
        self.add_message("assistant", &response);
        
        Ok(())
    }
    
    pub async fn load_project_context(&mut self, path: &str) -> Result<()> {
        let project_info = self.analyze_project(path).await?;
        self.context.project_info = Some(project_info);
        Ok(())
    }
    
    pub fn clear_context(&mut self) {
        self.context.messages.clear();
        self.context.current_files.clear();
    }
    
    pub fn get_context_summary(&self) -> String {
        let mut summary = String::new();
        
        if let Some(ref info) = self.context.project_info {
            summary.push_str(&format!("프로젝트: {} ({})\n", 
                info.name.bright_blue(), 
                info.language.cyan()
            ));
            
            if let Some(ref framework) = info.framework {
                summary.push_str(&format!("프레임워크: {}\n", framework.green()));
            }
            
            summary.push_str(&format!("의존성: {} 개\n", info.dependencies.len()));
        }
        
        summary.push_str(&format!("\n대화 기록: {} 개 메시지\n", self.context.messages.len()));
        
        if !self.context.current_files.is_empty() {
            summary.push_str(&format!("\n열린 파일:\n"));
            for file in &self.context.current_files {
                summary.push_str(&format!("  - {}\n", file));
            }
        }
        
        summary
    }
    
    pub fn save_session(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.context)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub async fn review_code(&self, path: &str, criteria: &str) -> Result<CodeReview> {
        let code = std::fs::read_to_string(path)
            .context("코드 파일 읽기 실패")?;
        
        let prompt = format!(
            "다음 코드를 {} 기준으로 리뷰해주세요:\n\n```\n{}\n```\n\n\
            JSON 형식으로 응답해주세요.",
            criteria, code
        );
        
        let response = self.client.query(&prompt).await?;
        let review: CodeReview = serde_json::from_str(&response)
            .context("리뷰 결과 파싱 실패")?;
        
        Ok(review)
    }
    
    pub async fn generate_documentation(&self, target: &str, doc_type: &str) -> Result<String> {
        let prompt = match doc_type {
            "api" => format!("{}에 대한 API 문서를 생성해주세요.", target),
            "guide" => format!("{}에 대한 사용 가이드를 작성해주세요.", target),
            "readme" => format!("{}에 대한 README.md 파일을 작성해주세요.", target),
            "architecture" => format!("{}의 아키텍처 문서를 작성해주세요.", target),
            _ => format!("{}에 대한 문서를 작성해주세요.", target),
        };
        
        self.client.query(&prompt).await
    }
    
    async fn analyze_project(&self, path: &str) -> Result<ProjectInfo> {
        let path = Path::new(path);
        
        // 프로젝트 이름
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // 언어 감지
        let (language, framework) = self.detect_language_and_framework(path).await?;
        
        // 의존성 추출
        let dependencies = self.extract_dependencies(path, &language).await?;
        
        // 구조 분석
        let structure = self.analyze_structure(path).await?;
        
        Ok(ProjectInfo {
            name,
            language,
            framework,
            dependencies,
            structure,
        })
    }
    
    async fn detect_language_and_framework(&self, path: &Path) -> Result<(String, Option<String>)> {
        // 파일 확장자와 설정 파일로 언어와 프레임워크 감지
        if path.join("Cargo.toml").exists() {
            Ok(("Rust".to_string(), None))
        } else if path.join("package.json").exists() {
            let content = std::fs::read_to_string(path.join("package.json"))?;
            let framework = if content.contains("\"react\"") {
                Some("React".to_string())
            } else if content.contains("\"vue\"") {
                Some("Vue".to_string())
            } else if content.contains("\"@angular/core\"") {
                Some("Angular".to_string())
            } else {
                None
            };
            Ok(("JavaScript/TypeScript".to_string(), framework))
        } else if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
            Ok(("Python".to_string(), None))
        } else {
            Ok(("Unknown".to_string(), None))
        }
    }
    
    async fn extract_dependencies(&self, path: &Path, language: &str) -> Result<Vec<String>> {
        let mut deps = Vec::new();
        
        match language {
            "Rust" => {
                if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
                    // 간단한 의존성 추출 (실제로는 toml 파싱이 필요)
                    for line in content.lines() {
                        if line.contains(" = ") && !line.starts_with('#') {
                            if let Some(dep) = line.split(" = ").next() {
                                deps.push(dep.trim().to_string());
                            }
                        }
                    }
                }
            }
            "JavaScript/TypeScript" => {
                if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(dependencies) = json.get("dependencies") {
                            if let Some(obj) = dependencies.as_object() {
                                deps.extend(obj.keys().map(|k| k.to_string()));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(deps)
    }
    
    async fn analyze_structure(&self, path: &Path) -> Result<String> {
        let mut structure = String::new();
        self.walk_directory(path, &mut structure, 0, 3)?;
        Ok(structure)
    }
    
    fn walk_directory(&self, path: &Path, output: &mut String, depth: usize, max_depth: usize) -> Result<()> {
        if depth > max_depth {
            return Ok(());
        }
        
        let entries = std::fs::read_dir(path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // 숨김 파일과 일반적인 무시 패턴 스킵
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            
            output.push_str(&"  ".repeat(depth));
            
            if path.is_dir() {
                output.push_str(&format!("📁 {}/\n", name));
                self.walk_directory(&path, output, depth + 1, max_depth)?;
            } else {
                output.push_str(&format!("📄 {}\n", name));
            }
        }
        
        Ok(())
    }
    
    fn create_system_prompt(&self) -> String {
        let mut prompt = String::from(
            "당신은 전문 개발 어시스턴트입니다. \
            개발자들의 작업을 도와주고, 코드 리뷰, 문서 작성, 디버깅, 아키텍처 설계 등을 지원합니다.\n\n"
        );
        
        if let Some(ref info) = self.context.project_info {
            prompt.push_str(&format!(
                "현재 작업 중인 프로젝트:\n\
                - 이름: {}\n\
                - 언어: {}\n",
                info.name, info.language
            ));
            
            if let Some(ref framework) = info.framework {
                prompt.push_str(&format!("- 프레임워크: {}\n", framework));
            }
            
            prompt.push_str("\n");
        }
        
        // 대화 모드에 따른 가이드라인 설정
        let guidelines = match self.chat_mode {
            ChatMode::Normal => {
                "응답 시 다음 가이드라인을 따라주세요:\n\
                1. 짧고 간결하게 답변 (3-5문장 이내 선호)\n\
                2. 꼭 필요한 경우에만 코드 예제 제공\n\
                3. 장황한 설명 대신 핵심만 전달\n\
                4. 사용자가 추가 정보를 요청하면 상세히 설명\n\
                5. 한국어로 친절하게 설명"
            }
            ChatMode::Concise => {
                "응답 시 다음 가이드라인을 따라주세요:\n\
                1. 매우 간결하게 답변 (1-2문장)\n\
                2. 핵심만 전달\n\
                3. 코드는 최소한으로\n\
                4. 한국어로 답변"
            }
            ChatMode::Detailed => {
                "응답 시 다음 가이드라인을 따라주세요:\n\
                1. 상세하고 체계적으로 설명\n\
                2. 단계별로 구분하여 설명\n\
                3. 예제 코드와 함께 설명\n\
                4. 모범 사례와 주의사항 포함\n\
                5. 한국어로 친절하게 설명"
            }
            ChatMode::Code => {
                "응답 시 다음 가이드라인을 따라주세요:\n\
                1. 코드 중심으로 답변\n\
                2. 실행 가능한 완전한 코드 제공\n\
                3. 주석으로 간단히 설명\n\
                4. 코드 품질과 최적화 중시\n\
                5. 한국어 주석 사용"
            }
            ChatMode::Planning => {
                "응답 시 다음 가이드라인을 따라주세요:\n\
                1. 체계적인 계획 수립\n\
                2. 단계별 작업 분해\n\
                3. 우선순위와 의존관계 명시\n\
                4. 예상 소요시간 포함\n\
                5. 한국어로 명확하게 작성"
            }
        };
        
        prompt.push_str(guidelines);
        
        prompt
    }
    
    fn add_message(&mut self, role: &str, content: &str) {
        self.context.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
        });
    }
    
    fn should_render(&self, buffer: &str) -> bool {
        // 코드 블록이나 문단이 완성되었는지 확인
        buffer.ends_with('\n') || 
        buffer.ends_with("```") ||
        buffer.ends_with(". ") ||
        buffer.len() > 100
    }
}

impl CodeReview {
    pub fn format_markdown(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# 코드 리뷰 결과\n\n"));
        output.push_str(&format!("**전체 점수**: {:.1}/10\n\n", self.overall_score));
        
        if !self.positive_aspects.is_empty() {
            output.push_str("## 👍 긍정적인 부분\n\n");
            for aspect in &self.positive_aspects {
                output.push_str(&format!("- {}\n", aspect));
            }
            output.push_str("\n");
        }
        
        if !self.issues.is_empty() {
            output.push_str("## 🔍 발견된 이슈\n\n");
            for issue in &self.issues {
                let emoji = match issue.severity {
                    IssueSeverity::Critical => "🔴",
                    IssueSeverity::High => "🟠",
                    IssueSeverity::Medium => "🟡",
                    IssueSeverity::Low => "🟢",
                    IssueSeverity::Info => "ℹ️",
                };
                
                output.push_str(&format!(
                    "### {} [{:?}] {:?} - {}\n\n",
                    emoji, issue.severity, issue.category, issue.location
                ));
                output.push_str(&format!("{}\n", issue.description));
                
                if let Some(ref suggestion) = issue.suggestion {
                    output.push_str(&format!("\n**제안**: {}\n", suggestion));
                }
                output.push_str("\n");
            }
        }
        
        if !self.suggestions.is_empty() {
            output.push_str("## 💡 개선 제안\n\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("- {}\n", suggestion));
            }
        }
        
        output
    }
} 