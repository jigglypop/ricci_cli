use anyhow::{Result};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::config::Config;
use crate::api::OpenAIClient;

pub struct ProjectPlanner {
    client: OpenAIClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPlan {
    pub title: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub phases: Vec<Phase>,
    pub milestones: Vec<Milestone>,
    pub risks: Vec<Risk>,
    pub dependencies: Vec<Dependency>,
    pub total_duration: EstimatedDuration,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tasks: Vec<Task>,
    pub duration: EstimatedDuration,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assignee: Option<String>,
    pub priority: Priority,
    pub effort: EffortLevel,
    pub duration: EstimatedDuration,
    pub subtasks: Vec<SubTask>,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub name: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub date: Option<DateTime<Utc>>,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub probability: RiskLevel,
    pub impact: RiskLevel,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Blocking,
    Soft,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedDuration {
    pub min_days: f32,
    pub max_days: f32,
    pub likely_days: f32,
}

impl ProjectPlanner {
    pub fn new(config: Config) -> Result<Self> {
        let client = OpenAIClient::new(&config)?;
        Ok(Self { client })
    }
    
    pub async fn create_plan(
        &self, 
        description: &str, 
        detail_level: u8,
        include_estimates: bool
    ) -> Result<ProjectPlan> {
        let prompt = self.build_planning_prompt(description, detail_level, include_estimates);
        
        let response = self.client.query(&prompt).await?;
        
        // AI 응답을 파싱하여 구조화된 계획 생성
        let plan = self.parse_plan_response(&response, description)?;
        
        Ok(plan)
    }
    
    fn build_planning_prompt(&self, description: &str, detail_level: u8, include_estimates: bool) -> String {
        let detail_instruction = match detail_level {
            1 => "매우 간단한 개요만",
            2 => "주요 단계와 마일스톤",
            3 => "상세한 태스크와 하위 작업",
            4 => "매우 상세한 계획과 수락 기준",
            5 => "완전한 프로젝트 계획서",
            _ => "적절한 수준의",
        };
        
        let mut prompt = format!(
            "다음 프로젝트에 대한 {} 작업계획서를 작성해주세요:\n\n{}\n\n",
            detail_instruction, description
        );
        
        prompt.push_str("다음 형식으로 응답해주세요:\n\n");
        prompt.push_str("# 프로젝트 제목\n\n");
        prompt.push_str("## 목표\n- 목표 1\n- 목표 2\n\n");
        prompt.push_str("## 단계별 계획\n\n");
        prompt.push_str("### 1단계: [단계명]\n");
        prompt.push_str("설명: ...\n");
        prompt.push_str("주요 작업:\n");
        prompt.push_str("- [ ] 작업 1\n");
        prompt.push_str("  - [ ] 하위 작업 1.1\n");
        prompt.push_str("  - [ ] 하위 작업 1.2\n");
        
        if include_estimates {
            prompt.push_str("예상 소요 시간: X-Y일\n");
        }
        
        prompt.push_str("\n## 마일스톤\n");
        prompt.push_str("- **M1**: [마일스톤명] - [설명]\n");
        prompt.push_str("  - 산출물: ...\n\n");
        
        prompt.push_str("## 위험 요소\n");
        prompt.push_str("- **위험**: [설명]\n");
        prompt.push_str("  - 대응 방안: ...\n\n");
        
        prompt.push_str("## 의존성\n");
        prompt.push_str("- A는 B에 의존\n");
        
        prompt
    }
    
    fn parse_plan_response(&self, response: &str, original_description: &str) -> Result<ProjectPlan> {
        // 간단한 파싱 로직 (실제로는 더 정교한 파싱 필요)
        let lines: Vec<&str> = response.lines().collect();
        let mut plan = ProjectPlan {
            title: "프로젝트 계획".to_string(),
            description: original_description.to_string(),
            objectives: Vec::new(),
            phases: Vec::new(),
            milestones: Vec::new(),
            risks: Vec::new(),
            dependencies: Vec::new(),
            total_duration: EstimatedDuration {
                min_days: 0.0,
                max_days: 0.0,
                likely_days: 0.0,
            },
            created_at: Utc::now(),
        };
        
        let mut current_section = "";
        let mut current_phase: Option<Phase> = None;
        let mut phase_counter = 0;
        
        for line in lines {
            let trimmed = line.trim();
            
            // 섹션 감지
            if trimmed.starts_with("# ") {
                plan.title = trimmed[2..].to_string();
            } else if trimmed == "## 목표" {
                current_section = "objectives";
            } else if trimmed == "## 단계별 계획" {
                current_section = "phases";
            } else if trimmed.starts_with("### ") && current_section == "phases" {
                // 이전 단계 저장
                if let Some(phase) = current_phase.take() {
                    plan.phases.push(phase);
                }
                
                // 새 단계 시작
                phase_counter += 1;
                let phase_name = trimmed[4..].to_string();
                current_phase = Some(Phase {
                    id: format!("phase_{}", phase_counter),
                    name: phase_name,
                    description: String::new(),
                    tasks: Vec::new(),
                    duration: EstimatedDuration {
                        min_days: 5.0,
                        max_days: 10.0,
                        likely_days: 7.0,
                    },
                    dependencies: Vec::new(),
                });
            } else if trimmed == "## 마일스톤" {
                current_section = "milestones";
                // 마지막 단계 저장
                if let Some(phase) = current_phase.take() {
                    plan.phases.push(phase);
                }
            } else if trimmed == "## 위험 요소" {
                current_section = "risks";
            } else if trimmed == "## 의존성" {
                current_section = "dependencies";
            }
            
            // 내용 파싱
            match current_section {
                "objectives" => {
                    if trimmed.starts_with("- ") {
                        plan.objectives.push(trimmed[2..].to_string());
                    }
                }
                "phases" => {
                    if let Some(ref mut phase) = current_phase {
                        if trimmed.starts_with("설명: ") {
                            phase.description = trimmed[5..].to_string();
                        } else if trimmed.starts_with("- [ ] ") {
                            let task_name = trimmed[6..].to_string();
                            phase.tasks.push(Task {
                                id: format!("task_{}_{}", phase_counter, phase.tasks.len() + 1),
                                name: task_name,
                                description: String::new(),
                                assignee: None,
                                priority: Priority::Medium,
                                effort: EffortLevel::Medium,
                                duration: EstimatedDuration {
                                    min_days: 1.0,
                                    max_days: 3.0,
                                    likely_days: 2.0,
                                },
                                subtasks: Vec::new(),
                                acceptance_criteria: Vec::new(),
                            });
                        } else if trimmed.starts_with("  - [ ] ") && !phase.tasks.is_empty() {
                            let subtask_name = trimmed[8..].to_string();
                            if let Some(last_task) = phase.tasks.last_mut() {
                                last_task.subtasks.push(SubTask {
                                    name: subtask_name,
                                    completed: false,
                                });
                            }
                        }
                    }
                }
                "milestones" => {
                    if trimmed.starts_with("- **") {
                        if let Some(colon_pos) = trimmed.find(':') {
                            let name = trimmed[5..colon_pos-2].to_string();
                            let description = trimmed[colon_pos+2..].to_string();
                            plan.milestones.push(Milestone {
                                name,
                                description,
                                date: None,
                                deliverables: Vec::new(),
                            });
                        }
                    } else if trimmed.starts_with("  - 산출물: ") && !plan.milestones.is_empty() {
                        if let Some(last_milestone) = plan.milestones.last_mut() {
                            last_milestone.deliverables.push(trimmed[12..].to_string());
                        }
                    }
                }
                "risks" => {
                    if trimmed.starts_with("- **위험**: ") {
                        let description = trimmed[11..].to_string();
                        plan.risks.push(Risk {
                            description,
                            probability: RiskLevel::Medium,
                            impact: RiskLevel::Medium,
                            mitigation: String::new(),
                        });
                    } else if trimmed.starts_with("  - 대응 방안: ") && !plan.risks.is_empty() {
                        if let Some(last_risk) = plan.risks.last_mut() {
                            last_risk.mitigation = trimmed[14..].to_string();
                        }
                    }
                }
                _ => {}
            }
        }
        
        // 전체 기간 계산
        plan.total_duration = self.calculate_total_duration(&plan.phases);
        
        Ok(plan)
    }
    
    fn calculate_total_duration(&self, phases: &[Phase]) -> EstimatedDuration {
        let min: f32 = phases.iter().map(|p| p.duration.min_days).sum();
        let max: f32 = phases.iter().map(|p| p.duration.max_days).sum();
        let likely: f32 = phases.iter().map(|p| p.duration.likely_days).sum();
        
        EstimatedDuration {
            min_days: min,
            max_days: max,
            likely_days: likely,
        }
    }
}

impl ProjectPlan {
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# {}\n\n", self.title));
        output.push_str(&format!("**생성일**: {}\n\n", self.created_at.format("%Y-%m-%d %H:%M")));
        output.push_str(&format!("## 프로젝트 설명\n\n{}\n\n", self.description));
        
        output.push_str("## 목표\n\n");
        for objective in &self.objectives {
            output.push_str(&format!("- {}\n", objective));
        }
        output.push_str("\n");
        
        output.push_str("## 단계별 계획\n\n");
        for (i, phase) in self.phases.iter().enumerate() {
            output.push_str(&format!("### {}단계: {}\n\n", i + 1, phase.name));
            output.push_str(&format!("{}\n\n", phase.description));
            output.push_str(&format!("**예상 기간**: {:.0}-{:.0}일 (평균 {:.0}일)\n\n", 
                phase.duration.min_days, phase.duration.max_days, phase.duration.likely_days));
            
            output.push_str("#### 주요 작업\n\n");
            for task in &phase.tasks {
                output.push_str(&format!("- [ ] **{}** ({})\n", task.name, 
                    match task.priority {
                        Priority::Critical => "긴급",
                        Priority::High => "높음",
                        Priority::Medium => "보통",
                        Priority::Low => "낮음",
                    }
                ));
                
                for subtask in &task.subtasks {
                    output.push_str(&format!("  - [ ] {}\n", subtask.name));
                }
            }
            output.push_str("\n");
        }
        
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
        
        if !self.risks.is_empty() {
            output.push_str("## 위험 관리\n\n");
            for risk in &self.risks {
                let risk_level = match (&risk.probability, &risk.impact) {
                    (RiskLevel::High, RiskLevel::High) => "🔴 매우 높음",
                    (RiskLevel::High, _) | (_, RiskLevel::High) => "🟠 높음",
                    (RiskLevel::Medium, RiskLevel::Medium) => "🟡 중간",
                    _ => "🟢 낮음",
                };
                
                output.push_str(&format!("### {} {}\n\n", risk_level, risk.description));
                output.push_str(&format!("**대응 방안**: {}\n\n", risk.mitigation));
            }
        }
        
        output.push_str(&format!("## 전체 예상 기간\n\n"));
        output.push_str(&format!("- **최소**: {:.0}일\n", self.total_duration.min_days));
        output.push_str(&format!("- **최대**: {:.0}일\n", self.total_duration.max_days));
        output.push_str(&format!("- **예상**: {:.0}일\n", self.total_duration.likely_days));
        
        output
    }
} 