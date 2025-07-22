use anyhow::{Result, anyhow};
use crate::config::types::Config;

pub fn validate_config(config: &Config) -> Result<()> {
    // API 키 확인
    let has_any_key = config.openai_api_key.is_some() 
        || config.anthropic_api_key.is_some()
        || config.gemini_api_key.is_some();
    
    if !has_any_key {
        return Err(anyhow!(
            "최소 하나의 API 키가 필요합니다.\n\
            환경 변수 설정: OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY"
        ));
    }
    
    // 모델 설정 확인
    if config.model_preferences.temperature < 0.0 || config.model_preferences.temperature > 2.0 {
        return Err(anyhow!("temperature는 0.0에서 2.0 사이여야 합니다"));
    }
    
    if config.model_preferences.max_tokens == 0 {
        return Err(anyhow!("max_tokens는 0보다 커야 합니다"));
    }
    
    Ok(())
}

pub fn get_api_key(config: &Config, provider: &str) -> Option<String> {
    match provider {
        "openai" => config.openai_api_key.clone(),
        "anthropic" => config.anthropic_api_key.clone(),
        "gemini" => config.gemini_api_key.clone(),
        _ => None,
    }
} 