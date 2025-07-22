use anyhow::Result;
use std::path::Path;
use std::fs;
use crate::assistant::types::{CodeReview, IssueSeverity};
use crate::api::OpenAIClient;
use colored::*;

pub async fn review_code(
    client: &OpenAIClient,
    path: &str,
    criteria: &str
) -> Result<CodeReview> {
    let code_content = if Path::new(path).is_file() {
        fs::read_to_string(path)?
    } else {
        return Err(anyhow::anyhow!("경로가 파일이 아닙니다: {}", path));
    };
    
    let prompt = format!(
        "다음 코드를 검토하고 {} 기준으로 평가해주세요:\n\n```\n{}\n```\n\n\
        JSON 형식으로 응답해주세요:\n\
        {{\n\
          \"overall_score\": 0-100,\n\
          \"issues\": [\n\
            {{\n\
              \"severity\": \"Critical|High|Medium|Low|Info\",\n\
              \"category\": \"Security|Performance|Style|BestPractice|Bug|Documentation\",\n\
              \"location\": \"파일:라인\",\n\
              \"description\": \"문제 설명\",\n\
              \"suggestion\": \"개선 방안\"\n\
            }}\n\
          ],\n\
          \"suggestions\": [\"전반적인 개선 제안\"],\n\
          \"positive_aspects\": [\"잘된 점\"]\n\
        }}",
        criteria, code_content
    );
    
    let response = client.query(&prompt).await?;
    let review: CodeReview = serde_json::from_str(&response)?;
    
    Ok(review)
}

impl CodeReview {
    pub fn format_markdown(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# 코드 리뷰 결과\n\n"));
        output.push_str(&format!("**전체 점수**: {}/100\n\n", self.overall_score));
        
        if !self.positive_aspects.is_empty() {
            output.push_str("## 👍 잘된 점\n\n");
            for aspect in &self.positive_aspects {
                output.push_str(&format!("- {}\n", aspect));
            }
            output.push_str("\n");
        }
        
        if !self.issues.is_empty() {
            output.push_str("## 🔍 발견된 문제\n\n");
            for issue in &self.issues {
                let severity_icon = match issue.severity {
                    IssueSeverity::Critical => "🔴",
                    IssueSeverity::High => "🟠",
                    IssueSeverity::Medium => "🟡",
                    IssueSeverity::Low => "🟢",
                    IssueSeverity::Info => "ℹ️",
                };
                
                output.push_str(&format!("### {} {:?} - {:?}\n\n", 
                    severity_icon, issue.severity, issue.category));
                output.push_str(&format!("**위치**: {}\n\n", issue.location));
                output.push_str(&format!("{}\n\n", issue.description));
                
                if let Some(suggestion) = &issue.suggestion {
                    output.push_str(&format!("**제안**: {}\n\n", suggestion));
                }
            }
        }
        
        if !self.suggestions.is_empty() {
            output.push_str("## 💡 개선 제안\n\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("- {}\n", suggestion));
            }
        }
        
        output
    }
    
    pub fn print_summary(&self) {
        println!("\n{}", "코드 리뷰 요약".bright_cyan().bold());
        println!("{}", "=".repeat(50).dimmed());
        
        let score_color = if self.overall_score >= 80.0 {
            self.overall_score.to_string().green()
        } else if self.overall_score >= 60.0 {
            self.overall_score.to_string().yellow()
        } else {
            self.overall_score.to_string().red()
        };
        
        println!("전체 점수: {}/100", score_color);
        
        let critical_count = self.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical))
            .count();
        let high_count = self.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::High))
            .count();
        
        if critical_count > 0 {
            println!("치명적 문제: {}", critical_count.to_string().red().bold());
        }
        if high_count > 0 {
            println!("높은 우선순위 문제: {}", high_count.to_string().yellow());
        }
        
        println!("전체 문제: {}", self.issues.len());
        println!("개선 제안: {}", self.suggestions.len());
    }
} 