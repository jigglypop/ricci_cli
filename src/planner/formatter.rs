use crate::planner::types::*;
use prettytable::{Table, row};
use colored::*;

impl ProjectPlan {
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# {}\n\n", self.title));
        output.push_str(&format!("{}\n\n", self.description));
        output.push_str(&format!("생성일: {}\n\n", self.created_at.format("%Y-%m-%d %H:%M")));
        
        // 목표
        if !self.objectives.is_empty() {
            output.push_str("## 목표\n\n");
            for obj in &self.objectives {
                output.push_str(&format!("- {}\n", obj));
            }
            output.push_str("\n");
        }
        
        // 단계별 계획
        if !self.phases.is_empty() {
            output.push_str("## 단계별 계획\n\n");
            for (i, phase) in self.phases.iter().enumerate() {
                output.push_str(&format!("### {} {}\n\n", i + 1, phase.name));
                output.push_str(&format!("{}\n\n", phase.description));
                
                if !phase.dependencies.is_empty() {
                    output.push_str(&format!("**의존성**: {}\n\n", phase.dependencies.join(", ")));
                }
                
                output.push_str(&format!("**예상 기간**: {}\n\n", phase.duration));
                
                if !phase.tasks.is_empty() {
                    output.push_str("**작업 목록**:\n\n");
                    for task in &phase.tasks {
                        output.push_str(&format!("- **{}** ({})\n", task.name, task.priority));
                        output.push_str(&format!("  - {}\n", task.description));
                        output.push_str(&format!("  - 예상 소요: {}\n", task.duration));
                        
                        if !task.acceptance_criteria.is_empty() {
                            output.push_str("  - 완료 기준:\n");
                            for criterion in &task.acceptance_criteria {
                                output.push_str(&format!("    - {}\n", criterion));
                            }
                        }
                    }
                    output.push_str("\n");
                }
            }
        }
        
        // 마일스톤
        if !self.milestones.is_empty() {
            output.push_str("## 마일스톤\n\n");
            for milestone in &self.milestones {
                output.push_str(&format!("### {}\n\n", milestone.name));
                output.push_str(&format!("{}\n\n", milestone.description));
                
                if !milestone.deliverables.is_empty() {
                    output.push_str("**산출물**:\n");
                    for deliverable in &milestone.deliverables {
                        output.push_str(&format!("- {}\n", deliverable));
                    }
                    output.push_str("\n");
                }
            }
        }
        
        // 위험 요소
        if !self.risks.is_empty() {
            output.push_str("## 위험 요소\n\n");
            for risk in &self.risks {
                let risk_score = format!("{}/{}", risk.probability, risk.impact);
                output.push_str(&format!("- **{}** ({})\n", risk.description, risk_score));
                output.push_str(&format!("  - 대응: {}\n", risk.mitigation));
            }
            output.push_str("\n");
        }
        
        // 총 기간
        output.push_str(&format!("## 총 예상 기간\n\n{}\n", self.total_duration));
        
        output
    }
    
    pub fn to_table(&self) -> String {
        let mut table = Table::new();
        table.add_row(row!["단계", "작업", "우선순위", "예상 시간"]);
        
        for phase in &self.phases {
            for task in &phase.tasks {
                table.add_row(row![
                    phase.name,
                    task.name,
                    format!("{:?}", task.priority),
                    task.duration.to_string()
                ]);
            }
        }
        
        table.to_string()
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Priority::Critical => "긴급".red().bold(),
            Priority::High => "높음".yellow(),
            Priority::Medium => "보통".normal(),
            Priority::Low => "낮음".dimmed(),
        };
        write!(f, "{}", text)
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            RiskLevel::High => "높음".red(),
            RiskLevel::Medium => "중간".yellow(),
            RiskLevel::Low => "낮음".green(),
        };
        write!(f, "{}", text)
    }
}

impl std::fmt::Display for EstimatedDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min_hours == self.max_hours {
            write!(f, "{:.1}시간", self.min_hours)
        } else {
            write!(f, "{:.1}-{:.1}시간 (평균 {:.1}시간)", 
                self.min_hours, self.max_hours, self.likely_hours)
        }
    }
} 