use clap::{Parser, Subcommand};
use clap_complete::Shell;
use crate::handlers::config::ConfigAction;

#[derive(Parser)]
#[clap(name = "ricci")]
#[clap(about = "AI 기반 개발 어시스턴트 CLI", version)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
    
    /// 직접 질문하기 (서브커맨드 없이)
    #[clap(value_name = "QUERY")]
    pub query: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 대화형 모드로 실행
    Chat {
        /// 프로젝트 컨텍스트 포함
        #[clap(short, long)]
        context: bool,
        
        /// 세션 저장 경로
        #[clap(short, long)]
        save: Option<String>,
    },
    /// 작업계획서 생성
    Plan {
        /// 프로젝트 설명 또는 요구사항
        description: String,
        /// 출력 형식 (markdown, json, yaml)
        #[clap(short, long, default_value = "markdown")]
        format: String,
        /// 상세 레벨 (1-5)
        #[clap(short, long, default_value = "3")]
        detail: u8,
        /// 일정 추정 포함
        #[clap(short, long)]
        estimate: bool,
    },
    
    /// 프로젝트 분석
    Analyze {
        /// 분석할 디렉토리 경로
        #[clap(default_value = ".")]
        path: String,
        
        /// 분석 유형 (structure, dependencies, complexity, all)
        #[clap(short, long, default_value = "all")]
        type_: String,
    },
    
    /// 코드 리뷰
    Review {
        /// 리뷰할 파일 또는 디렉토리
        path: String,
        
        /// 리뷰 기준 (security, performance, style, all)
        #[clap(short, long, default_value = "all")]
        criteria: String,
    },
    
    /// 문서 생성
    Doc {
        /// 문서화할 대상
        target: String,
        
        /// 문서 유형 (api, guide, readme, architecture)
        #[clap(short, long, default_value = "readme")]
        type_: String,
    },
    
    /// 설정 관리
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
    
    /// 쉘 완성 스크립트 생성
    Completion {
        /// 대상 쉘
        #[clap(value_enum)]
        shell: Shell,
    },
    
    /// 자동완성 설치
    Install {
        /// 대상 쉘 (자동 감지하려면 비워두세요)
        #[clap(value_enum)]
        shell: Option<Shell>,
    },
} 