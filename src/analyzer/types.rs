use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root_path: PathBuf,
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: HashMap<String, LanguageStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageStats {
    pub file_count: usize,
    pub line_count: usize,
    pub percentage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub direct_dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityInfo {
    pub file: String,
    pub complexity: u32,
    pub lines: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub average_complexity: f32,
    pub complex_files: Vec<ComplexityInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullAnalysisReport {
    pub structure: ProjectStructure,
    pub dependencies: DependencyAnalysis,
    pub complexity: ComplexityReport,
} 