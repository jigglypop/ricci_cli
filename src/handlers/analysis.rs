use anyhow::Result;
use colored::*;
use crate::{
    assistant::DevAssistant,
    analyzer::CodeAnalyzer,
    planner::ProjectPlanner,
    config::Config,
};

pub async fn handle_plan(
    description: &str,
    format: &str,
    detail: u8,
    estimate: bool,
    config: &Config,
) -> Result<()> {
    println!("{}", "작업계획서 생성 중...".yellow());
    
    let planner = ProjectPlanner::new(config.clone())?;
    let plan = planner.create_plan(description, detail, estimate).await?;
    
    match format {
        "markdown" => {
            println!("\n{}", plan.to_markdown());
        }
        "json" => {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&plan)?);
        }
        _ => {
            anyhow::bail!("지원하지 않는 형식: {}", format);
        }
    }
    
    Ok(())
}

pub async fn handle_analyze(path: &str, type_: &str, config: &Config) -> Result<()> {
    println!("{} {}", "분석 중:".yellow(), path);
    
    let analyzer = CodeAnalyzer::new(config.clone())?;
    
    match type_ {
        "structure" => {
            let structure = analyzer.analyze_structure(path).await?;
            analyzer.print_structure_report(&structure);
        }
        "dependencies" => {
            let deps = analyzer.analyze_dependencies(path).await?;
            analyzer.print_dependency_report(&deps);
        }
        "complexity" => {
            let complexity = analyzer.analyze_complexity(path).await?;
            analyzer.print_complexity_report(&complexity);
        }
        "all" => {
            let report = analyzer.analyze_all(path).await?;
            analyzer.print_full_report(&report);
        }
        _ => {
            anyhow::bail!("지원하지 않는 분석 유형: {}", type_);
        }
    }
    
    Ok(())
}

pub async fn handle_review(path: &str, criteria: &str, config: &Config) -> Result<()> {
    println!("{} {}", "코드 리뷰 중:".yellow(), path);
    
    let assistant = DevAssistant::new(config.clone())?;
    let review = assistant.review_code(path, criteria).await?;
    
    println!("\n{}", review.format_markdown());
    
    Ok(())
}

pub async fn handle_doc(target: &str, type_: &str, config: &Config) -> Result<()> {
    println!("{} {} 문서 생성 중...", type_.cyan(), target);
    
    let assistant = DevAssistant::new(config.clone())?;
    let doc = assistant.generate_documentation(target, type_).await?;
    
    println!("\n{doc}");
    
    Ok(())
} 