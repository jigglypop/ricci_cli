mod types;
mod structure;
mod dependencies;
mod complexity;

pub use types::*;
pub use structure::analyze_structure;
pub use dependencies::analyze_dependencies;
pub use complexity::analyze_complexity;

use anyhow::Result;
use crate::config::Config;

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new(_config: Config) -> Result<Self> {
        Ok(Self)
    }
    
    pub async fn analyze_structure(&self, path: &str) -> Result<ProjectStructure> {
        analyze_structure(path).await
    }
    
    pub async fn analyze_dependencies(&self, path: &str) -> Result<DependencyAnalysis> {
        analyze_dependencies(path).await
    }
    
    pub async fn analyze_complexity(&self, path: &str) -> Result<ComplexityReport> {
        analyze_complexity(path).await
    }
    
    pub async fn analyze_all(&self, path: &str) -> Result<FullAnalysisReport> {
        let structure = self.analyze_structure(path).await?;
        let dependencies = self.analyze_dependencies(path).await?;
        let complexity = self.analyze_complexity(path).await?;
        
        Ok(FullAnalysisReport {
            structure,
            dependencies,
            complexity,
        })
    }
    
    pub fn print_structure_report(&self, structure: &ProjectStructure) {
        structure::print_structure_report(structure);
    }
    
    pub fn print_dependency_report(&self, deps: &DependencyAnalysis) {
        dependencies::print_dependency_report(deps);
    }
    
    pub fn print_complexity_report(&self, complexity: &ComplexityReport) {
        complexity::print_complexity_report(complexity);
    }
    
    pub fn print_full_report(&self, report: &FullAnalysisReport) {
        self.print_structure_report(&report.structure);
        self.print_dependency_report(&report.dependencies);
        self.print_complexity_report(&report.complexity);
    }
} 