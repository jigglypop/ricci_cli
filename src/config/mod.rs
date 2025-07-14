use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use colored::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub api_key_source: ApiKeySource,
    pub model_preferences: ModelPreferences,
    pub output_preferences: OutputPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
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

impl Default for ApiKeySource {
    fn default() -> Self {
        ApiKeySource::Environment
    }
}

impl Default for ModelPreferences {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            default_model: "gpt-4-turbo-preview".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}

impl Default for OutputPreferences {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            markdown_rendering: true,
            auto_save_sessions: false,
            session_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("ricci")
                .join("sessions"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .context("설정 파일 읽기 실패")?;
            toml::from_str(&content)
                .context("설정 파일 파싱 실패")?
        } else {
            Config::default()
        };
        
        // API 키 로드 (우선순위: 환경변수 > 설정파일 > REST API)
        config.load_api_keys().ok();
        
        Ok(config)
    }
    
    fn load_api_keys(&mut self) -> Result<()> {
        match &self.api_key_source {
            ApiKeySource::Environment => {
                // .env 파일이나 시스템 환경변수에서 로드
                if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                    self.openai_api_key = Some(key);
                }
                if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                    self.anthropic_api_key = Some(key);
                }
                if let Ok(key) = std::env::var("GEMINI_API_KEY") {
                    self.gemini_api_key = Some(key);
                }
            }
            ApiKeySource::ConfigFile => {
                // 이미 파일에서 로드됨
            }
            ApiKeySource::RestApi { url } => {
                // REST API에서 키 가져오기
                let url_clone = url.clone();
                self.load_api_keys_from_rest(&url_clone)?;
            }
        }
        
        Ok(())
    }
    
    fn load_api_keys_from_rest(&mut self, url: &str) -> Result<()> {
        // 나중에 구현할 REST API 호출 로직
        // 예시:
        // let response = reqwest::blocking::get(url)?;
        // let keys: ApiKeys = response.json()?;
        // self.openai_api_key = keys.openai;
        // self.anthropic_api_key = keys.anthropic;
        // self.gemini_api_key = keys.gemini;
        
        eprintln!("REST API 키 로드는 아직 구현되지 않았습니다: {}", url);
        Ok(())
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("설정 디렉토리 생성 실패")?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("설정 직렬화 실패")?;
        
        std::fs::write(&config_path, content)
            .context("설정 파일 저장 실패")?;
        
        Ok(())
    }
    
    pub fn set_api_key(provider: &str, key: &str) -> Result<()> {
        let mut config = Self::load()?;
        
        match provider.to_lowercase().as_str() {
            "openai" => config.openai_api_key = Some(key.to_string()),
            "anthropic" => config.anthropic_api_key = Some(key.to_string()),
            "gemini" => config.gemini_api_key = Some(key.to_string()),
            _ => anyhow::bail!("지원하지 않는 제공자: {}", provider),
        }
        
        // 설정 파일에 저장하고 소스를 ConfigFile로 변경
        config.api_key_source = ApiKeySource::ConfigFile;
        config.save()?;
        Ok(())
    }
    
    pub fn set_api_source(source: ApiKeySource) -> Result<()> {
        let mut config = Self::load()?;
        config.api_key_source = source;
        config.save()?;
        Ok(())
    }
    
    pub fn reset() -> Result<()> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            std::fs::remove_file(&config_path)
                .context("설정 파일 삭제 실패")?;
        }
        Ok(())
    }
    
    pub fn display(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", "현재 설정:".bright_blue().bold()));
        
        output.push_str(&format!("\n{}\n", "API 키:".cyan()));
        output.push_str(&format!("  OpenAI: {}\n", 
            self.openai_api_key.as_ref()
                .map(|k| format!("{}...{}", &k[..6.min(k.len())], &k[k.len().saturating_sub(4)..]))
                .unwrap_or_else(|| "설정되지 않음".red().to_string())
        ));
        output.push_str(&format!("  Anthropic: {}\n", 
            self.anthropic_api_key.as_ref()
                .map(|k| format!("{}...{}", &k[..6.min(k.len())], &k[k.len().saturating_sub(4)..]))
                .unwrap_or_else(|| "설정되지 않음".red().to_string())
        ));
        output.push_str(&format!("  Gemini: {}\n", 
            self.gemini_api_key.as_ref()
                .map(|k| format!("{}...{}", &k[..6.min(k.len())], &k[k.len().saturating_sub(4)..]))
                .unwrap_or_else(|| "설정되지 않음".red().to_string())
        ));
        
        output.push_str(&format!("\n  API 키 소스: {:?}\n", self.api_key_source));
        
        output.push_str(&format!("\n{}\n", "모델 설정:".cyan()));
        output.push_str(&format!("  기본 제공자: {}\n", self.model_preferences.default_provider));
        output.push_str(&format!("  기본 모델: {}\n", self.model_preferences.default_model));
        output.push_str(&format!("  Temperature: {}\n", self.model_preferences.temperature));
        output.push_str(&format!("  최대 토큰: {}\n", self.model_preferences.max_tokens));
        
        output.push_str(&format!("\n{}\n", "출력 설정:".cyan()));
        output.push_str(&format!("  구문 강조: {}\n", 
            if self.output_preferences.syntax_highlighting { "활성".green() } else { "비활성".red() }
        ));
        output.push_str(&format!("  마크다운 렌더링: {}\n", 
            if self.output_preferences.markdown_rendering { "활성".green() } else { "비활성".red() }
        ));
        output.push_str(&format!("  자동 세션 저장: {}\n", 
            if self.output_preferences.auto_save_sessions { "활성".green() } else { "비활성".red() }
        ));
        
        output
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("설정 디렉토리를 찾을 수 없습니다")?
            .join("ricci");
        
        Ok(config_dir.join("config.toml"))
    }
    
    pub fn get_active_api_key(&self) -> Result<&str> {
        match self.model_preferences.default_provider.as_str() {
            "openai" => self.openai_api_key.as_deref()
                .context("OpenAI API 키가 설정되지 않았습니다"),
            "anthropic" => self.anthropic_api_key.as_deref()
                .context("Anthropic API 키가 설정되지 않았습니다"),
            "gemini" => self.gemini_api_key.as_deref()
                .context("Gemini API 키가 설정되지 않았습니다"),
            _ => anyhow::bail!("알 수 없는 제공자: {}", self.model_preferences.default_provider),
        }
    }
} 