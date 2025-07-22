mod types;
mod loader;
mod validators;

pub use types::*;
pub use loader::{load_config, save_config, get_config_path};
pub use validators::{validate_config, get_api_key};

use anyhow::Result;
use colored::*;

impl Config {
    pub fn load() -> Result<Self> {
        let config = load_config()?;
        validate_config(&config)?;
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        save_config(self)
    }
    
    pub fn update_api_key(&mut self, provider: &str, key: String) -> Result<()> {
        match provider {
            "openai" => self.openai_api_key = Some(key),
            "anthropic" => self.anthropic_api_key = Some(key),
            "gemini" => self.gemini_api_key = Some(key),
            _ => return Err(anyhow::anyhow!("알 수 없는 제공자: {}", provider)),
        }
        
        self.api_key_source = ApiKeySource::ConfigFile;
        self.save()?;
        Ok(())
    }
    
    pub fn print_status(&self) {
        println!("\n{}", "현재 설정:".bright_cyan().bold());
        println!("{}", "=".repeat(50).dimmed());
        
        let check = |key: &Option<String>| {
            if key.is_some() { "✓".green() } else { "✗".red() }
        };
        
        println!("API 키:");
        println!("  {} OpenAI: {}", 
            check(&self.openai_api_key),
            if self.openai_api_key.is_some() { "설정됨" } else { "미설정" }
        );
        println!("  {} Anthropic: {}", 
            check(&self.anthropic_api_key),
            if self.anthropic_api_key.is_some() { "설정됨" } else { "미설정" }
        );
        println!("  {} Gemini: {}", 
            check(&self.gemini_api_key),
            if self.gemini_api_key.is_some() { "설정됨" } else { "미설정" }
        );
        
        println!("\n모델 설정:");
        println!("  기본 제공자: {}", self.model_preferences.default_provider.yellow());
        println!("  기본 모델: {}", self.model_preferences.default_model.yellow());
        println!("  Temperature: {}", self.model_preferences.temperature.to_string().yellow());
        println!("  Max Tokens: {}", self.model_preferences.max_tokens.to_string().yellow());
        
        println!("\n출력 설정:");
        println!("  구문 강조: {}", 
            if self.output_preferences.syntax_highlighting { "켜짐".green() } else { "꺼짐".red() }
        );
        println!("  마크다운 렌더링: {}", 
            if self.output_preferences.markdown_rendering { "켜짐".green() } else { "꺼짐".red() }
        );
        println!("  세션 자동 저장: {}", 
            if self.output_preferences.auto_save_sessions { "켜짐".green() } else { "꺼짐".red() }
        );
    }
    
    pub fn get_active_api_key(&self) -> Result<&str> {
        match self.model_preferences.default_provider.as_str() {
            "openai" => self.openai_api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("OpenAI API 키가 설정되지 않았습니다")),
            "anthropic" => self.anthropic_api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("Anthropic API 키가 설정되지 않았습니다")),
            "gemini" => self.gemini_api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("Gemini API 키가 설정되지 않았습니다")),
            _ => Err(anyhow::anyhow!("알 수 없는 제공자: {}", self.model_preferences.default_provider))
        }
    }
    
    pub fn set_api_key(provider: &str, key: &str) -> Result<()> {
        let mut config = Self::load()?;
        config.update_api_key(provider, key.to_string())?;
        Ok(())
    }
    
    pub fn reset() -> Result<()> {
        let config_path = get_config_path()?;
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
        }
        Ok(())
    }
    
    pub fn display(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", "현재 설정:".bright_cyan().bold()));
        output.push_str(&format!("{}\n", "=".repeat(50).dimmed()));
        
        let mask_api_key = |key: &Option<String>| {
            key.as_ref()
                .map(|k| format!("{}...{}", &k[..6.min(k.len())], &k[k.len().saturating_sub(4)..]))
                .unwrap_or_else(|| "미설정".red().to_string())
        };
        
        output.push_str("API 키:\n");
        output.push_str(&format!("  OpenAI: {}\n", mask_api_key(&self.openai_api_key)));
        output.push_str(&format!("  Anthropic: {}\n", mask_api_key(&self.anthropic_api_key)));
        output.push_str(&format!("  Gemini: {}\n", mask_api_key(&self.gemini_api_key)));
        
        output.push_str(&format!("\n모델 설정:\n"));
        output.push_str(&format!("  기본 제공자: {}\n", self.model_preferences.default_provider.yellow()));
        output.push_str(&format!("  기본 모델: {}\n", self.model_preferences.default_model.yellow()));
        output.push_str(&format!("  Temperature: {}\n", self.model_preferences.temperature.to_string().yellow()));
        output.push_str(&format!("  Max Tokens: {}\n", self.model_preferences.max_tokens.to_string().yellow()));
        
        output
    }
} 