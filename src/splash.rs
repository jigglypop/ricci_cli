use colored::*;
use figlet_rs::FIGfont;
use std::io::{self, Write};

pub fn display_splash() -> io::Result<()> {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");
    
    // Ricci CLI 타이틀을 FIGlet으로 표시
    if let Ok(standard_font) = FIGfont::standard() {
        if let Some(figure) = standard_font.convert("RICCI CLI") {
            println!("{}", figure.to_string().cyan().bold());
        }
    }
    
    println!("\n{}", "═══════════════════════════════════════════════════════════════════════".bright_blue());
    println!("{}", "    AI CLI by IT신기술융합팀".bright_magenta());
    println!("{}", "═══════════════════════════════════════════════════════════════════════".bright_blue());
    
    // 사용법
    println!("\n  {} 사용법:", "▸".bright_yellow());
    println!("    {}        - 셸 명령어(예: ls, cargo build)를 바로 실행합니다.", "명령어 입력".bright_cyan());
    println!("    {} 또는 {}      - AI와 대화하는 '대화 모드'로 전환합니다.", "c".bright_green(), "chat".bright_green());
    println!("    {} 또는 {}        - 도움말을 확인합니다.", "h".bright_magenta(), "/help".bright_magenta());
    println!("    {} 또는 {}     - 현재 대화로 작업 계획을 요약합니다.", "p".bright_cyan(), "/summary".bright_cyan());

    // 자동완성 안내
    println!("\n  {} 자동완성:", "▸".bright_yellow());
    println!("    {} (오른쪽 화살표)    - 입력 중 회색으로 표시되는 명령어를 완성합니다.", "→".bright_white());
    println!("    {}                - 가능한 명령어 목록을 확인합니다.", "Tab".bright_white());


    // 버전 정보
    println!("\n  {} Version {} | {}를 입력하여 대화를 종료합니다.", 
        "▸".bright_yellow(),
        env!("CARGO_PKG_VERSION").bright_white(),
        "exit".bright_cyan()
    );
    
    io::stdout().flush()?;
    Ok(())
}

pub fn display_mini_splash() {
    println!("{} - {}", 
        "RICCI CLI".cyan().bold(),
        "AI Development Assistant".bright_magenta()
    );
} 