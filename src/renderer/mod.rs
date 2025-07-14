use anyhow::Result;
use colored::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use termimad::*;
use termimad::crossterm::style::Color as CrosstermColor;
use std::io::Write;

pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    mad_skin: MadSkin,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        let mut mad_skin = MadSkin::default();
        
        // 마크다운 스타일 커스터마이징
        mad_skin.set_headers_fg(CrosstermColor::Cyan);
        mad_skin.bold.set_fg(CrosstermColor::White);
        mad_skin.italic.set_fg(CrosstermColor::Yellow);
        mad_skin.bullet = StyledChar::from_fg_char(CrosstermColor::Green, '•');
        mad_skin.quote_mark = StyledChar::from_fg_char(CrosstermColor::Magenta, '▌');
        
        Self {
            syntax_set,
            theme_set,
            mad_skin,
        }
    }
    
    pub fn render_chunk(&self, text: &str) -> Result<()> {
        // 코드 블록 처리
        if text.contains("```") {
            self.render_with_code_blocks(text)?;
        } else {
            // 일반 텍스트는 그대로 출력 (색상 없이)
            print!("{}", text);
            std::io::stdout().flush()?;
        }
        Ok(())
    }
    
    pub fn render_markdown(&self, markdown: &str) -> Result<()> {
        // 전체 마크다운 문서 렌더링
        self.mad_skin.print_text(markdown);
        Ok(())
    }
    
    fn render_with_code_blocks(&self, text: &str) -> Result<()> {
        let mut in_code_block = false;
        let mut language = String::new();
        let mut code_buffer = String::new();
        let mut text_buffer = String::new();
        
        for line in text.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // 코드 블록 종료
                    self.highlight_code(&language, &code_buffer)?;
                    code_buffer.clear();
                    language.clear();
                    in_code_block = false;
                } else {
                    // 코드 블록 시작
                    if !text_buffer.is_empty() {
                        print!("{}", text_buffer);
                        text_buffer.clear();
                    }
                    language = line[3..].trim().to_string();
                    in_code_block = true;
                }
            } else if in_code_block {
                code_buffer.push_str(line);
                code_buffer.push('\n');
            } else {
                text_buffer.push_str(line);
                text_buffer.push('\n');
            }
        }
        
        // 남은 버퍼 처리
        if !text_buffer.is_empty() {
            print!("{}", text_buffer);
        }
        if !code_buffer.is_empty() {
            self.highlight_code(&language, &code_buffer)?;
        }
        
        Ok(())
    }
    
    fn highlight_code(&self, language: &str, code: &str) -> Result<()> {
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        
        // 언어별 구문 강조
        let syntax = self.syntax_set
            .find_syntax_by_name(language)
            .or_else(|| self.syntax_set.find_syntax_by_extension(language))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let mut highlighter = HighlightLines::new(syntax, theme);
        
        // 코드 블록 헤더
        println!("\n{}", format!("─── {} ───", language).dimmed());
        
        for line in code.lines() {
            let ranges = highlighter.highlight_line(line, &self.syntax_set)?;
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            println!("{}", escaped);
        }
        
        println!("{}", "─────────".dimmed());
        
        Ok(())
    }
    
    pub fn render_table(&self, headers: &[&str], rows: &[Vec<String>]) {
        use prettytable::{Table, cell};
        
        let mut table = Table::new();
        
        // 헤더 추가
        let header_cells: Vec<_> = headers.iter()
            .map(|h| cell!(h.bright_cyan().bold()))
            .collect();
        table.add_row(prettytable::Row::new(header_cells));
        
        // 데이터 행 추가
        for row in rows {
            let cells: Vec<_> = row.iter()
                .map(|c| cell!(c))
                .collect();
            table.add_row(prettytable::Row::new(cells));
        }
        
        table.printstd();
    }
    
    pub fn render_progress(&self, message: &str, current: usize, total: usize) {
        let percentage = (current as f32 / total as f32 * 100.0) as u32;
        let filled = (percentage as usize * 50) / 100;
        let empty = 50 - filled;
        
        print!("\r{}: [{}{}] {}%",
            message.cyan(),
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed(),
            percentage
        );
        
        if current == total {
            println!(" ✓");
        }
    }
} 