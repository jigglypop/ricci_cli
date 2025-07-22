use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatMode {
    Normal,     // 일반 대화
    Concise,    // 간결한 응답
    Detailed,   // 상세한 응답
    Code,       // 코드 중심
    Planning,   // 계획 수립 모드
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantContext {
    pub messages: Vec<Message>,
    pub project_info: Option<ProjectInfo>,
    pub current_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub language: String,
    pub framework: Option<String>,
    pub dependencies: Vec<String>,
    pub structure: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub file_path: String,
    pub original_code: String,
    pub suggested_code: String,
    pub reason: String,
}

impl Default for AssistantContext {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            project_info: None,
            current_files: Vec::new(),
        }
    }
} 