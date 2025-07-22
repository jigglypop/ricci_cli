use anyhow::{Result, Context};
use std::fs;
use std::path::PathBuf;
use colored::*;
use crate::config::types::{Config, ApiKeySource};

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    
    if config_path.exists() {
        println!("{} {}", "설정 파일 로드 중:".dimmed(), config_path.display());
        
        let content = fs::read_to_string(&config_path)
            .context("설정 파일 읽기 실패")?;
        
        toml::from_str(&content)
            .context("설정 파일 파싱 실패")
    } else {
        println!("{}", "기본 설정 사용 중".yellow());
        let mut config = Config::default();
        
        // 환경 변수에서 API 키 로드
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.openai_api_key = Some(key);
            config.api_key_source = ApiKeySource::Environment;
        }
        
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            config.anthropic_api_key = Some(key);
        }
        
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            config.gemini_api_key = Some(key);
        }
        
        Ok(config)
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .context("설정 디렉토리 생성 실패")?;
    }
    
    let content = toml::to_string_pretty(config)
        .context("설정 직렬화 실패")?;
    
    fs::write(&config_path, content)
        .context("설정 파일 저장 실패")?;
    
    println!("{} {}", "설정 저장 완료:".green(), config_path.display());
    Ok(())
}

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("설정 디렉토리를 찾을 수 없습니다")?;
    
    Ok(config_dir.join("ricci").join("config.toml"))
} 