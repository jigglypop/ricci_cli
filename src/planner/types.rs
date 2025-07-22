use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

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
    pub name: String,
    pub type_: DependencyType,
    pub description: String,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedDuration {
    pub min_hours: f32,
    pub max_hours: f32,
    pub likely_hours: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,  // < 2 hours
    Small,    // 2-8 hours
    Medium,   // 1-3 days
    Large,    // 3-10 days
    Epic,     // > 10 days
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DependencyType {
    Technical,
    Resource,
    External,
    Knowledge,
} 