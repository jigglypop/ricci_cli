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

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
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
            // 일반 텍스트는 그대로 출력
            print!("{text}");
        }
        std::io::stdout().flush()?;
        Ok(())
    }
    
    fn render_with_code_blocks(&self, text: &str) -> Result<()> {
        let mut in_code_block = false;
        let mut code_content = String::new();
        let mut language = String::new();
        let mut regular_content = String::new();
        
        for line in text.lines() {
            if line.trim().starts_with("```") {
                if in_code_block {
                    // 코드 블록 종료
                    self.highlight_code(&code_content, &language)?;
                    code_content.clear();
                    in_code_block = false;
                } else {
                    // 일반 텍스트 렌더링
                    if !regular_content.is_empty() {
                        self.render_markdown(&regular_content)?;
                        regular_content.clear();
                    }
                    
                    // 코드 블록 시작
                    language = line.trim()[3..].to_string();
                    in_code_block = true;
                }
            } else if in_code_block {
                code_content.push_str(line);
                code_content.push('\n');
            } else {
                regular_content.push_str(line);
                regular_content.push('\n');
            }
        }
        
        // 남은 내용 처리
        if !regular_content.is_empty() {
            self.render_markdown(&regular_content)?;
        }
        
        Ok(())
    }
    
    pub fn render(&self, text: &str) -> Result<()> {
        self.render_markdown(text)
    }
    
    fn render_markdown(&self, text: &str) -> Result<()> {
        self.mad_skin.print_text(text);
        Ok(())
    }
    
    fn highlight_code(&self, code: &str, language: &str) -> Result<()> {
        let syntax = self.syntax_set
            .find_syntax_by_token(language)
            .or_else(|| self.syntax_set.find_syntax_by_extension(language))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let theme = &self.theme_set.themes["base16-monokai.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        
        println!("\n{}", format!("```{}", language).dimmed());
        
        for line in code.lines() {
            let ranges = highlighter.highlight_line(line, &self.syntax_set)?;
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            println!("{}", escaped);
        }
        
        println!("{}", "```".dimmed());
        
        Ok(())
    }
} 