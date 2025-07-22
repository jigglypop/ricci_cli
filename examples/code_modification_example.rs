use ricci_cli::assistant::{DevAssistant, CodeSuggestion, FileChange, FileModifier};
use ricci_cli::config::Config;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 설정 로드
    let config = Config::load()?;
    let assistant = DevAssistant::new(config)?;
    
    // 예제 1: AI가 제안한 코드 변경사항 적용
    let suggestions = vec![
        CodeSuggestion {
            file_path: "src/main.rs".to_string(),
            original_code: r#"fn main() {
    println!("Hello, world!");
}"#.to_string(),
            suggested_code: r#"use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Hello, {}!", args.get(1).unwrap_or(&"world".to_string()));
}"#.to_string(),
            reason: "명령줄 인자를 받아 인사할 수 있도록 개선".to_string(),
        },
    ];
    
    // 사용자에게 변경사항을 보여주고 확인받은 후 적용
    assistant.apply_code_suggestions(suggestions).await?;
    
    // 예제 2: 직접 파일 수정 (단일 파일)
    let modifier = FileModifier::new(false);
    modifier.modify_file(
        "README.md",
        "# My Project\n\nThis is an updated README.",
        "README 파일 업데이트"
    ).await?;
    
    // 예제 3: 여러 파일 한번에 수정
    let changes = vec![
        FileChange {
            path: "src/lib.rs".to_string(),
            original_content: "// old content".to_string(),
            new_content: "// new improved content\npub mod utils;".to_string(),
            description: "utils 모듈 추가".to_string(),
        },
        FileChange {
            path: "src/utils.rs".to_string(),
            original_content: String::new(),
            new_content: "pub fn helper() -> String {\n    \"Helper function\".to_string()\n}".to_string(),
            description: "새로운 utils 모듈 생성".to_string(),
        },
    ];
    
    // 백업과 함께 안전하게 수정
    assistant.safe_modify_files(changes).await?;
    
    Ok(())
}

// 실행 예시:
// 
// 3개의 파일을 수정할 예정입니다:
// 
// 파일 1/3: src/main.rs
// 설명: 명령줄 인자를 받아 인사할 수 있도록 개선
// 
// 변경사항:
// - fn main() {
// -     println!("Hello, world!");
// - }
// + use std::env;
// + 
// + fn main() {
// +     let args: Vec<String> = env::args().collect();
// +     println!("Hello, {}!", args.get(1).unwrap_or(&"world".to_string()));
// + }
// 
// 'src/main.rs' 파일을 어떻게 처리하시겠습니까?
// > 적용 (Apply)
//   건너뛰기 (Skip)
//   수정 (Edit)
//   취소 (Cancel all) 