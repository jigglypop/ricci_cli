use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub api_key_source: ApiKeySource,
    pub model_preferences: ModelPreferences,
    pub output_preferences: OutputPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
    #[default]
    Environment,
    ConfigFile,
    RestApi { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    pub default_provider: String,
    pub default_model: String,
    pub temperature: f32,
    pub max_tokens: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPreferences {
    pub syntax_highlighting: bool,
    pub markdown_rendering: bool,
    pub auto_save_sessions: bool,
    pub session_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            anthropic_api_key: None,
            gemini_api_key: None,
            api_key_source: ApiKeySource::Environment,
            model_preferences: ModelPreferences::default(),
            output_preferences: OutputPreferences::default(),
        }
    }
}

impl Default for ModelPreferences {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            default_model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

impl Default for OutputPreferences {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            markdown_rendering: true,
            auto_save_sessions: true,
            session_dir: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("ricci"),
        }
    }
} 