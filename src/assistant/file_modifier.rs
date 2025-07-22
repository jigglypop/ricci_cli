use anyhow::{Result, Context};
use colored::*;
use dialoguer::Select;
use std::fs;
use std::path::Path;
use similar::{ChangeTag, TextDiff};

pub struct FileModifier {
    auto_confirm: bool,
    show_diff: bool,
}

#[derive(Debug)]
pub struct FileChange {
    pub path: String,
    pub original_content: String,
    pub new_content: String,
    pub description: String,
}

impl FileModifier {
    pub fn new(auto_confirm: bool) -> Self {
        Self {
            auto_confirm,
            show_diff: true,
        }
    }
    
    /// 파일 변경사항을 미리보기하고 사용자 확인을 받습니다
    pub async fn apply_changes(&self, changes: Vec<FileChange>) -> Result<()> {
        if changes.is_empty() {
            println!("{}", "변경할 파일이 없습니다.".yellow());
            return Ok(());
        }
        
        println!("\n{}", format!("{}개의 파일을 수정할 예정입니다:", changes.len()).bright_cyan().bold());
        
        for (idx, change) in changes.iter().enumerate() {
            println!("\n{}", format!("파일 {}/{}: {}", idx + 1, changes.len(), change.path).bright_blue().bold());
            println!("{}", format!("설명: {}", change.description).dimmed());
            
            if self.show_diff {
                self.show_diff(&change.original_content, &change.new_content);
            }
            
            if !self.auto_confirm {
                let choice = self.ask_user_choice(&change.path)?;
                match choice {
                    UserChoice::Apply => self.apply_single_change(change)?,
                    UserChoice::Skip => {
                        println!("{}", "건너뛰었습니다.".yellow());
                        continue;
                    }
                    UserChoice::Edit => {
                        let edited_content = self.edit_change(change)?;
                        self.write_file(&change.path, &edited_content)?;
                    }
                    UserChoice::Cancel => {
                        println!("{}", "작업을 취소했습니다.".red());
                        return Ok(());
                    }
                }
            } else {
                self.apply_single_change(change)?;
            }
        }
        
        println!("\n{}", "모든 변경사항이 적용되었습니다.".green().bold());
        Ok(())
    }
    
    /// 단일 파일 변경을 확인하고 적용합니다
    pub async fn modify_file(&self, path: &str, new_content: &str, description: &str) -> Result<()> {
        let original_content = if Path::new(path).exists() {
            fs::read_to_string(path).context("파일 읽기 실패")?
        } else {
            String::new()
        };
        
        let change = FileChange {
            path: path.to_string(),
            original_content,
            new_content: new_content.to_string(),
            description: description.to_string(),
        };
        
        self.apply_changes(vec![change]).await
    }
    
    fn show_diff(&self, original: &str, new: &str) {
        let diff = TextDiff::from_lines(original, new);
        
        println!("\n{}", "변경사항:".yellow().bold());
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            
            let line = change.to_string_lossy();
            let formatted = match change.tag() {
                ChangeTag::Delete => format!("{} {}", sign, line).red(),
                ChangeTag::Insert => format!("{} {}", sign, line).green(),
                ChangeTag::Equal => {
                    // 변경사항 주변의 컨텍스트만 표시
                    if change.new_index().is_some() {
                        format!("{} {}", sign, line).dimmed()
                    } else {
                        continue;
                    }
                }
            };
            
            print!("{}", formatted);
        }
        println!();
    }
    
    fn ask_user_choice(&self, filename: &str) -> Result<UserChoice> {
        let options = vec![
            "적용 (Apply)",
            "건너뛰기 (Skip)",
            "수정 (Edit)",
            "취소 (Cancel all)",
        ];
        
        let selection = Select::new()
            .with_prompt(format!("'{}' 파일을 어떻게 처리하시겠습니까?", filename))
            .items(&options)
            .default(0)
            .interact()?;
        
        Ok(match selection {
            0 => UserChoice::Apply,
            1 => UserChoice::Skip,
            2 => UserChoice::Edit,
            _ => UserChoice::Cancel,
        })
    }
    
    fn edit_change(&self, change: &FileChange) -> Result<String> {
        println!("{}", "수정할 내용을 입력하세요 (Ctrl+D로 종료):".yellow());
        
        // 임시 파일에 현재 내용을 저장
        let temp_path = format!("{}.tmp", change.path);
        fs::write(&temp_path, &change.new_content)?;
        
        // 사용자의 기본 에디터로 파일 열기
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
        std::process::Command::new(editor)
            .arg(&temp_path)
            .status()
            .context("에디터 실행 실패")?;
        
        // 수정된 내용 읽기
        let edited_content = fs::read_to_string(&temp_path)?;
        fs::remove_file(temp_path)?;
        
        Ok(edited_content)
    }
    
    fn apply_single_change(&self, change: &FileChange) -> Result<()> {
        self.write_file(&change.path, &change.new_content)?;
        println!("{}", format!("✓ {} 수정 완료", change.path).green());
        Ok(())
    }
    
    fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let path = Path::new(path);
        
        // 디렉토리가 없으면 생성
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content).context("파일 쓰기 실패")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum UserChoice {
    Apply,
    Skip,
    Edit,
    Cancel,
}

/// 백업과 함께 안전한 파일 수정
pub struct SafeFileModifier {
    modifier: FileModifier,
    backup_dir: String,
}

impl SafeFileModifier {
    pub fn new(auto_confirm: bool) -> Self {
        Self {
            modifier: FileModifier::new(auto_confirm),
            backup_dir: ".ricci_backups".to_string(),
        }
    }
    
    pub async fn modify_with_backup(&self, changes: Vec<FileChange>) -> Result<()> {
        // 백업 디렉토리 생성
        fs::create_dir_all(&self.backup_dir)?;
        
        // 각 파일 백업
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        for change in &changes {
            if Path::new(&change.path).exists() {
                let backup_path = format!("{}/{}_{}.bak", 
                    self.backup_dir, 
                    change.path.replace('/', "_").replace('\\', "_"),
                    timestamp
                );
                fs::copy(&change.path, backup_path)?;
            }
        }
        
        // 변경사항 적용
        self.modifier.apply_changes(changes).await?;
        
        println!("\n{}", format!("백업 파일은 {} 디렉토리에 저장되었습니다.", self.backup_dir).dimmed());
        Ok(())
    }
} 