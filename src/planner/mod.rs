mod types;
mod formatter;

pub use types::*;

use anyhow::Result;
use chrono::Utc;
use crate::config::Config;
use crate::api::OpenAIClient;

pub struct ProjectPlanner {
    client: OpenAIClient,
}

impl ProjectPlanner {
    pub fn new(config: Config) -> Result<Self> {
        let client = OpenAIClient::new(&config)?;
        Ok(Self { client })
    }
    
    pub async fn create_plan(&self, description: &str, detail_level: u8, include_estimates: bool) -> Result<ProjectPlan> {
        let prompt = self.build_prompt(description, detail_level, include_estimates);
        let _response = self.client.query(&prompt).await?;
        
        // 간단한 파싱 로직 (실제로는 더 정교한 파싱 필요)
        let plan = ProjectPlan {
            title: "프로젝트 계획".to_string(),
            description: description.to_string(),
            objectives: Vec::new(),
            phases: Vec::new(),
            milestones: Vec::new(),
            risks: Vec::new(),
            dependencies: Vec::new(),
            total_duration: EstimatedDuration {
                min_hours: 0.0,
                max_hours: 0.0,
                likely_hours: 0.0,
            },
            created_at: Utc::now(),
        };
        
        // AI 응답을 파싱하여 계획 구조 채우기
        // TODO: 실제 파싱 로직 구현
        
        Ok(plan)
    }
    
    fn build_prompt(&self, description: &str, detail_level: u8, include_estimates: bool) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("당신은 프로젝트 계획 전문가입니다. 다음 프로젝트에 대한 상세한 계획을 작성해주세요.\n\n");
        prompt.push_str(&format!("프로젝트 설명: {}\n\n", description));
        
        prompt.push_str("다음 형식으로 작성해주세요:\n\n");
        prompt.push_str("# 프로젝트명\n\n");
        prompt.push_str("## 목표\n");
        prompt.push_str("- 목표 1\n");
        prompt.push_str("- 목표 2\n\n");
        
        prompt.push_str("## 단계별 계획\n");
        prompt.push_str("### 1단계: [단계명]\n");
        prompt.push_str("설명: ...\n");
        prompt.push_str("작업:\n");
        prompt.push_str("- [ ] 작업 1 (우선순위: 높음)\n");
        prompt.push_str("  - 설명: ...\n");
        prompt.push_str("  - 완료 기준: ...\n");
        
        if detail_level >= 2 {
            prompt.push_str("  - 하위 작업:\n");
            prompt.push_str("    - [ ] 세부 작업 1\n");
            prompt.push_str("    - [ ] 세부 작업 2\n");
        }
        
        if include_estimates {
            prompt.push_str("  - 예상 소요 시간: X-Y시간\n");
        }
        
        prompt.push_str("\n## 마일스톤\n");
        prompt.push_str("- **M1**: [마일스톤명] - [설명]\n");
        prompt.push_str("  - 산출물: ...\n\n");
        
        prompt.push_str("## 위험 요소\n");
        prompt.push_str("- **위험**: [설명]\n");
        prompt.push_str("  - 확률/영향: 높음/중간\n");
        prompt.push_str("  - 대응 방안: ...\n\n");
        
        prompt.push_str("## 의존성\n");
        prompt.push_str("- [의존성 설명]\n");
        
        prompt
    }
} 