use anyhow::{Result, Context};
use clap::{CommandFactory};
use clap_complete::{generate, Shell};
use colored::*;
use std::io::Write;
use crate::Cli;

pub fn install_completions(shell: Option<Shell>) -> Result<()> {
    // 쉘 자동 감지
    let detected_shell = if let Some(shell) = shell {
        shell
    } else {
        detect_shell()?
    };
    
    println!("{} {}", 
        "자동완성 설치 중:".bright_green(), 
        format!("{detected_shell:?}").cyan()
    );
    
    // 완성 스크립트 생성
    let mut cmd = Cli::command();
    let mut script = Vec::new();
    generate(detected_shell, &mut cmd, "ricci", &mut script);
    let script_content = String::from_utf8(script)?;
    
    // 설치 경로 결정
    match detected_shell {
        Shell::Bash => install_bash_completion(&script_content)?,
        Shell::Zsh => install_zsh_completion(&script_content)?,
        Shell::PowerShell => install_powershell_completion(&script_content)?,
        Shell::Fish => install_fish_completion(&script_content)?,
        _ => anyhow::bail!("지원하지 않는 쉘입니다: {detected_shell:?}"),
    }
    
    println!("{}", "✓ 자동완성 설치 완료!".green().bold());
    println!("\n다음 중 하나를 실행하여 적용하세요:");
    
    match detected_shell {
        Shell::Bash => println!("  source ~/.bashrc"),
        Shell::Zsh => println!("  source ~/.zshrc"),
        Shell::PowerShell => println!("  . $PROFILE"),
        Shell::Fish => println!("  source ~/.config/fish/config.fish"),
        _ => {}
    }
    
    println!("\n{}", "사용 예시:".yellow());
    println!("  ricci <Tab>        # 사용 가능한 명령어 보기");
    println!("  ricci plan <Tab>   # plan 옵션 보기");
    
    Ok(())
}

fn detect_shell() -> Result<Shell> {
    // Windows
    if cfg!(windows) {
        return Ok(Shell::PowerShell);
    }
    
    // Unix-like systems
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("bash") {
            return Ok(Shell::Bash);
        } else if shell.contains("zsh") {
            return Ok(Shell::Zsh);
        } else if shell.contains("fish") {
            return Ok(Shell::Fish);
        }
    }
    
    // 기본값
    Ok(Shell::Bash)
}

fn install_bash_completion(script: &str) -> Result<()> {
    let home = dirs::home_dir().context("홈 디렉토리를 찾을 수 없습니다")?;
    let completion_dir = home.join(".local").join("share").join("bash-completion").join("completions");
    std::fs::create_dir_all(&completion_dir)?;
    
    let completion_file = completion_dir.join("ricci");
    std::fs::write(&completion_file, script)?;
    
    // .bashrc에 추가
    let bashrc = home.join(".bashrc");
    if bashrc.exists() {
        let content = std::fs::read_to_string(&bashrc)?;
        if !content.contains("bash-completion/completions") {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&bashrc)?;
            writeln!(file, "\n# Ricci CLI 자동완성")?;
            writeln!(file, "[ -f ~/.local/share/bash-completion/completions/ricci ] && source ~/.local/share/bash-completion/completions/ricci")?;
        }
    }
    
    Ok(())
}

fn install_zsh_completion(script: &str) -> Result<()> {
    let home = dirs::home_dir().context("홈 디렉토리를 찾을 수 없습니다")?;
    let completion_dir = home.join(".local").join("share").join("zsh").join("completions");
    std::fs::create_dir_all(&completion_dir)?;
    
    let completion_file = completion_dir.join("_ricci");
    std::fs::write(&completion_file, script)?;
    
    // .zshrc에 fpath 추가
    let zshrc = home.join(".zshrc");
    if zshrc.exists() {
        let content = std::fs::read_to_string(&zshrc)?;
        if !content.contains(".local/share/zsh/completions") {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&zshrc)?;
            writeln!(file, "\n# Ricci CLI 자동완성")?;
            writeln!(file, "fpath=(~/.local/share/zsh/completions $fpath)")?;
            writeln!(file, "autoload -Uz compinit && compinit")?;
        }
    }
    
    Ok(())
}

fn install_powershell_completion(script: &str) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("설정 디렉토리를 찾을 수 없습니다")?
        .join("ricci");
    std::fs::create_dir_all(&config_dir)?;
    
    let completion_file = config_dir.join("ricci-completion.ps1");
    std::fs::write(&completion_file, script)?;
    
    // PowerShell 프로필에 추가
    if let Ok(profile) = std::env::var("PROFILE") {
        let profile_path = std::path::Path::new(&profile);
        if let Some(parent) = profile_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        if profile_path.exists() {
            let content = std::fs::read_to_string(profile_path)?;
            let import_line = format!(". \"{}\"", completion_file.display());
            
            if !content.contains(&import_line) {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(profile_path)?;
                writeln!(file, "\n# Ricci CLI 자동완성")?;
                writeln!(file, "{import_line}")?;
            }
        }
    }
    
    Ok(())
}

fn install_fish_completion(script: &str) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("설정 디렉토리를 찾을 수 없습니다")?
        .join("fish")
        .join("completions");
    std::fs::create_dir_all(&config_dir)?;
    
    let completion_file = config_dir.join("ricci.fish");
    std::fs::write(&completion_file, script)?;
    
    Ok(())
} 