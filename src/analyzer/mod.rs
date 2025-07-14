use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use colored::*;
use crate::config::Config;
use crate::api::OpenAIClient;
use walkdir::WalkDir;

pub struct CodeAnalyzer {
    #[allow(dead_code)]
    client: OpenAIClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root_path: PathBuf,
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: HashMap<String, LanguageStats>,
    pub directories: Vec<DirectoryInfo>,
    pub entry_points: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageStats {
    pub file_count: usize,
    pub line_count: usize,
    pub percentage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub file_count: usize,
    pub purpose: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub direct_dependencies: Vec<Dependency>,
    pub dev_dependencies: Vec<Dependency>,
    pub outdated: Vec<OutdatedDependency>,
    pub security_issues: Vec<SecurityIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutdatedDependency {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub dependency: String,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityInfo {
    pub file: String,
    pub function: Option<String>,
    pub complexity: u32,
    pub lines: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSmell {
    pub smell_type: SmellType,
    pub location: String,
    pub description: String,
    pub severity: SmellSeverity,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SmellType {
    LongMethod,
    LargeClass,
    DuplicateCode,
    DeadCode,
    GodObject,
    FeatureEnvy,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SmellSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub average_complexity: f32,
    pub max_complexity: ComplexityInfo,
    pub complex_files: Vec<ComplexityInfo>,
    pub code_smells: Vec<CodeSmell>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullAnalysisReport {
    pub structure: ProjectStructure,
    pub dependencies: DependencyAnalysis,
    pub complexity: ComplexityReport,
    pub recommendations: Vec<String>,
}

impl CodeAnalyzer {
    pub fn new(config: Config) -> Result<Self> {
        let client = OpenAIClient::new(&config)?;
        Ok(Self { client })
    }
    
    pub async fn analyze_structure(&self, path: &str) -> Result<ProjectStructure> {
        let root_path = Path::new(path).canonicalize()?;
        let mut languages: HashMap<String, LanguageStats> = HashMap::new();
        let mut directories: Vec<DirectoryInfo> = Vec::new();
        let mut total_files = 0;
        let mut total_lines = 0;
        
        // 디렉토리별 파일 수 계산
        let mut dir_file_counts: HashMap<PathBuf, usize> = HashMap::new();
        
        for entry in WalkDir::new(&root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // 무시할 디렉토리
            if path.components().any(|c| {
                let name = c.as_os_str().to_string_lossy();
                name == "target" || name == "node_modules" || name == ".git" || name == "dist"
            }) {
                continue;
            }
            
            if path.is_file() {
                total_files += 1;
                
                // 파일 확장자로 언어 감지
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_string();
                    let lang = self.detect_language(&ext_str);
                    
                    // 파일 라인 수 계산
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let line_count = content.lines().count();
                        total_lines += line_count;
                        
                        let stats = languages.entry(lang).or_insert(LanguageStats {
                            file_count: 0,
                            line_count: 0,
                            percentage: 0.0,
                        });
                        stats.file_count += 1;
                        stats.line_count += line_count;
                    }
                }
                
                // 디렉토리별 파일 수 업데이트
                if let Some(parent) = path.parent() {
                    *dir_file_counts.entry(parent.to_path_buf()).or_insert(0) += 1;
                }
            }
        }
        
        // 언어별 비율 계산
        for stats in languages.values_mut() {
            stats.percentage = (stats.line_count as f32 / total_lines as f32) * 100.0;
        }
        
        // 주요 디렉토리 정보
        for (dir, count) in dir_file_counts.iter() {
            if let Ok(relative) = dir.strip_prefix(&root_path) {
                let dir_name = relative.to_string_lossy().to_string();
                if !dir_name.is_empty() && *count > 0 {
                    let purpose = self.infer_directory_purpose(&dir_name);
                    directories.push(DirectoryInfo {
                        path: dir_name,
                        file_count: *count,
                        purpose,
                    });
                }
            }
        }
        
        // 엔트리 포인트 찾기
        let entry_points = self.find_entry_points(&root_path)?;
        
        Ok(ProjectStructure {
            root_path,
            total_files,
            total_lines,
            languages,
            directories,
            entry_points,
        })
    }
    
    pub async fn analyze_dependencies(&self, path: &str) -> Result<DependencyAnalysis> {
        let root_path = Path::new(path);
        let mut direct_dependencies = Vec::new();
        let mut dev_dependencies = Vec::new();
        
        // Rust 프로젝트
        if root_path.join("Cargo.toml").exists() {
            let content = std::fs::read_to_string(root_path.join("Cargo.toml"))?;
            self.parse_cargo_dependencies(&content, &mut direct_dependencies, &mut dev_dependencies)?;
        }
        
        // Node.js 프로젝트
        if root_path.join("package.json").exists() {
            let content = std::fs::read_to_string(root_path.join("package.json"))?;
            self.parse_npm_dependencies(&content, &mut direct_dependencies, &mut dev_dependencies)?;
        }
        
        // Python 프로젝트
        if root_path.join("requirements.txt").exists() {
            let content = std::fs::read_to_string(root_path.join("requirements.txt"))?;
            self.parse_pip_dependencies(&content, &mut direct_dependencies)?;
        }
        
        // 간단한 보안 이슈 체크 (실제로는 외부 서비스 활용 필요)
        let security_issues = self.check_security_issues(&direct_dependencies);
        
        Ok(DependencyAnalysis {
            direct_dependencies,
            dev_dependencies,
            outdated: Vec::new(), // 실제로는 레지스트리 확인 필요
            security_issues,
        })
    }
    
    pub async fn analyze_complexity(&self, path: &str) -> Result<ComplexityReport> {
        let root_path = Path::new(path);
        let mut complex_files = Vec::new();
        let mut total_complexity = 0u32;
        let mut file_count = 0;
        let mut code_smells = Vec::new();
        
        for entry in WalkDir::new(root_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() && self.is_source_file(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let complexity = self.calculate_cyclomatic_complexity(&content);
                    let lines = content.lines().count();
                    
                    if complexity > 10 {
                        complex_files.push(ComplexityInfo {
                            file: path.strip_prefix(root_path)
                                .unwrap_or(path)
                                .to_string_lossy()
                                .to_string(),
                            function: None,
                            complexity,
                            lines,
                        });
                    }
                    
                    // 코드 스멜 검사
                    if lines > 500 {
                        code_smells.push(CodeSmell {
                            smell_type: SmellType::LargeClass,
                            location: path.display().to_string(),
                            description: format!("파일이 너무 큽니다 ({}줄)", lines),
                            severity: SmellSeverity::High,
                        });
                    }
                    
                    total_complexity += complexity;
                    file_count += 1;
                }
            }
        }
        
        let average_complexity = if file_count > 0 {
            total_complexity as f32 / file_count as f32
        } else {
            0.0
        };
        
        let max_complexity = complex_files.iter()
            .max_by_key(|f| f.complexity)
            .cloned()
            .unwrap_or(ComplexityInfo {
                file: String::new(),
                function: None,
                complexity: 0,
                lines: 0,
            });
        
        Ok(ComplexityReport {
            average_complexity,
            max_complexity,
            complex_files,
            code_smells,
        })
    }
    
    pub async fn analyze_all(&self, path: &str) -> Result<FullAnalysisReport> {
        let structure = self.analyze_structure(path).await?;
        let dependencies = self.analyze_dependencies(path).await?;
        let complexity = self.analyze_complexity(path).await?;
        
        // AI를 사용한 추천사항 생성
        let recommendations = self.generate_recommendations(&structure, &dependencies, &complexity).await?;
        
        Ok(FullAnalysisReport {
            structure,
            dependencies,
            complexity,
            recommendations,
        })
    }
    
    pub fn print_structure_report(&self, structure: &ProjectStructure) {
        println!("{}", "프로젝트 구조 분석".bright_cyan().bold());
        println!("{}", "=".repeat(50).dimmed());
        
        println!("총 파일 수: {}", structure.total_files.to_string().yellow());
        println!("총 코드 라인: {}", structure.total_lines.to_string().yellow());
        
        println!("\n{}", "언어별 통계:".cyan());
        for (lang, stats) in &structure.languages {
            println!("  {} - {} 파일, {} 라인 ({:.1}%)",
                lang.green(),
                stats.file_count,
                stats.line_count,
                stats.percentage
            );
        }
        
        println!("\n{}", "주요 디렉토리:".cyan());
        for dir in &structure.directories {
            println!("  📁 {} ({} 파일) - {}",
                dir.path.blue(),
                dir.file_count,
                dir.purpose.dimmed()
            );
        }
        
        if !structure.entry_points.is_empty() {
            println!("\n{}", "엔트리 포인트:".cyan());
            for entry in &structure.entry_points {
                println!("  🚀 {}", entry.yellow());
            }
        }
    }
    
    pub fn print_dependency_report(&self, deps: &DependencyAnalysis) {
        println!("{}", "의존성 분석".bright_cyan().bold());
        println!("{}", "=".repeat(50).dimmed());
        
        println!("직접 의존성: {} 개", deps.direct_dependencies.len().to_string().yellow());
        println!("개발 의존성: {} 개", deps.dev_dependencies.len().to_string().yellow());
        
        if !deps.security_issues.is_empty() {
            println!("\n{}", "⚠️  보안 이슈:".red().bold());
            for issue in &deps.security_issues {
                println!("  {} - {} ({})",
                    issue.dependency.red(),
                    issue.description,
                    issue.severity.yellow()
                );
            }
        }
        
        if !deps.outdated.is_empty() {
            println!("\n{}", "오래된 의존성:".yellow());
            for outdated in &deps.outdated {
                println!("  {} {} → {}",
                    outdated.name,
                    outdated.current_version.dimmed(),
                    outdated.latest_version.green()
                );
            }
        }
    }
    
    pub fn print_complexity_report(&self, complexity: &ComplexityReport) {
        println!("{}", "복잡도 분석".bright_cyan().bold());
        println!("{}", "=".repeat(50).dimmed());
        
        println!("평균 복잡도: {:.1}", complexity.average_complexity);
        
        if complexity.max_complexity.complexity > 0 {
            println!("\n{}", "가장 복잡한 파일:".red());
            println!("  {} (복잡도: {})",
                complexity.max_complexity.file.yellow(),
                complexity.max_complexity.complexity.to_string().red()
            );
        }
        
        if !complexity.complex_files.is_empty() {
            println!("\n{}", "복잡한 파일들 (복잡도 > 10):".yellow());
            for file in &complexity.complex_files {
                println!("  {} - 복잡도: {}, {} 라인",
                    file.file,
                    file.complexity.to_string().yellow(),
                    file.lines
                );
            }
        }
        
        if !complexity.code_smells.is_empty() {
            println!("\n{}", "코드 스멜:".yellow());
            for smell in &complexity.code_smells {
                let icon = match smell.severity {
                    SmellSeverity::High => "🔴",
                    SmellSeverity::Medium => "🟠",
                    SmellSeverity::Low => "🟡",
                };
                println!("  {} {:?} - {}",
                    icon,
                    smell.smell_type,
                    smell.description
                );
            }
        }
    }
    
    pub fn print_full_report(&self, report: &FullAnalysisReport) {
        self.print_structure_report(&report.structure);
        println!();
        self.print_dependency_report(&report.dependencies);
        println!();
        self.print_complexity_report(&report.complexity);
        
        if !report.recommendations.is_empty() {
            println!("\n{}", "💡 개선 권장사항:".bright_cyan().bold());
            for (i, rec) in report.recommendations.iter().enumerate() {
                println!("{}. {}", i + 1, rec);
            }
        }
    }
    
    fn detect_language(&self, extension: &str) -> String {
        match extension {
            "rs" => "Rust",
            "js" | "jsx" => "JavaScript",
            "ts" | "tsx" => "TypeScript",
            "py" => "Python",
            "java" => "Java",
            "go" => "Go",
            "cpp" | "cc" | "cxx" => "C++",
            "c" => "C",
            "cs" => "C#",
            "rb" => "Ruby",
            "php" => "PHP",
            "swift" => "Swift",
            "kt" => "Kotlin",
            _ => "Other",
        }.to_string()
    }
    
    fn infer_directory_purpose(&self, dir_name: &str) -> String {
        match dir_name {
            name if name.contains("src") => "소스 코드",
            name if name.contains("test") => "테스트 코드",
            name if name.contains("doc") => "문서",
            name if name.contains("example") => "예제 코드",
            name if name.contains("script") => "스크립트",
            name if name.contains("config") => "설정 파일",
            name if name.contains("asset") || name.contains("static") => "정적 리소스",
            name if name.contains("api") => "API 코드",
            name if name.contains("component") => "컴포넌트",
            name if name.contains("util") || name.contains("helper") => "유틸리티",
            name if name.contains("model") => "데이터 모델",
            name if name.contains("view") => "뷰 레이어",
            name if name.contains("controller") => "컨트롤러",
            _ => "기타",
        }.to_string()
    }
    
    fn find_entry_points(&self, root_path: &Path) -> Result<Vec<String>> {
        let mut entry_points = Vec::new();
        
        // 일반적인 엔트리 포인트 파일들
        let common_entries = [
            "main.rs", "lib.rs", "index.js", "index.ts", "app.js", "app.ts",
            "main.py", "__main__.py", "server.js", "server.ts", "index.html"
        ];
        
        for entry in &common_entries {
            for path in WalkDir::new(root_path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if path.file_name() == std::ffi::OsStr::new(entry) {
                    if let Ok(relative) = path.path().strip_prefix(root_path) {
                        entry_points.push(relative.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(entry_points)
    }
    
    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy();
            matches!(ext.as_ref(), 
                "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "java" | 
                "go" | "c" | "cpp" | "cc" | "cs" | "rb" | "php" | "swift" | "kt"
            )
        } else {
            false
        }
    }
    
    fn calculate_cyclomatic_complexity(&self, content: &str) -> u32 {
        // 간단한 복잡도 계산 (실제로는 AST 분석 필요)
        let mut complexity = 1;
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("if ") || trimmed.contains("else if") {
                complexity += 1;
            }
            if trimmed.contains("for ") || trimmed.contains("while ") {
                complexity += 1;
            }
            if trimmed.contains("case ") || trimmed.contains("catch ") {
                complexity += 1;
            }
            if trimmed.contains("&&") || trimmed.contains("||") {
                complexity += 1;
            }
        }
        
        complexity
    }
    
    fn parse_cargo_dependencies(
        &self, 
        content: &str, 
        direct: &mut Vec<Dependency>,
        dev: &mut Vec<Dependency>
    ) -> Result<()> {
        // 간단한 Cargo.toml 파싱
        let mut in_dependencies = false;
        let mut in_dev_dependencies = false;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed == "[dependencies]" {
                in_dependencies = true;
                in_dev_dependencies = false;
            } else if trimmed == "[dev-dependencies]" {
                in_dependencies = false;
                in_dev_dependencies = true;
            } else if trimmed.starts_with('[') {
                in_dependencies = false;
                in_dev_dependencies = false;
            } else if (in_dependencies || in_dev_dependencies) && trimmed.contains('=') {
                if let Some((name, version)) = trimmed.split_once('=') {
                    let dep = Dependency {
                        name: name.trim().to_string(),
                        version: version.trim().trim_matches('"').to_string(),
                        license: None,
                    };
                    
                    if in_dependencies {
                        direct.push(dep);
                    } else {
                        dev.push(dep);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_npm_dependencies(
        &self, 
        content: &str, 
        direct: &mut Vec<Dependency>,
        dev: &mut Vec<Dependency>
    ) -> Result<()> {
        let json: serde_json::Value = serde_json::from_str(content)?;
        
        if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps {
                direct.push(Dependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("unknown").to_string(),
                    license: None,
                });
            }
        }
        
        if let Some(deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps {
                dev.push(Dependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("unknown").to_string(),
                    license: None,
                });
            }
        }
        
        Ok(())
    }
    
    fn parse_pip_dependencies(&self, content: &str, direct: &mut Vec<Dependency>) -> Result<()> {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let parts: Vec<&str> = trimmed.split("==").collect();
                let name = parts[0].to_string();
                let version = if parts.len() > 1 { 
                    parts[1].to_string() 
                } else { 
                    "latest".to_string() 
                };
                
                direct.push(Dependency {
                    name,
                    version,
                    license: None,
                });
            }
        }
        
        Ok(())
    }
    
    fn check_security_issues(&self, dependencies: &[Dependency]) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        
        // 간단한 보안 체크 (실제로는 CVE 데이터베이스 확인 필요)
        for dep in dependencies {
            if dep.name.contains("log4j") && dep.version.starts_with("2.") {
                issues.push(SecurityIssue {
                    dependency: dep.name.clone(),
                    severity: "High".to_string(),
                    description: "Log4j 취약점 가능성".to_string(),
                });
            }
        }
        
        issues
    }
    
    async fn generate_recommendations(
        &self,
        structure: &ProjectStructure,
        dependencies: &DependencyAnalysis,
        complexity: &ComplexityReport
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        // 구조 기반 권장사항
        if structure.total_files > 1000 {
            recommendations.push("프로젝트가 매우 큽니다. 모듈화를 고려해보세요.".to_string());
        }
        
        // 의존성 기반 권장사항
        if dependencies.direct_dependencies.len() > 50 {
            recommendations.push("의존성이 많습니다. 불필요한 의존성을 제거하는 것을 고려해보세요.".to_string());
        }
        
        if !dependencies.security_issues.is_empty() {
            recommendations.push("보안 취약점이 발견되었습니다. 의존성을 업데이트하세요.".to_string());
        }
        
        // 복잡도 기반 권장사항
        if complexity.average_complexity > 10.0 {
            recommendations.push("평균 복잡도가 높습니다. 함수를 더 작게 분할하세요.".to_string());
        }
        
        if !complexity.code_smells.is_empty() {
            recommendations.push("코드 스멜이 발견되었습니다. 리팩토링을 고려해보세요.".to_string());
        }
        
        Ok(recommendations)
    }
} 