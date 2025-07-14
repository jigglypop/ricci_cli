use std::env;
use std::path::Path;

fn main() {
    // 빌드 시 자동완성 설치 메시지 표시
    if env::var("CARGO_FEATURE_AUTO_COMPLETE").is_ok() {
        println!("cargo:warning=자동완성을 설치하려면 설치 후 'ricci install'을 실행하세요.");
    }
    
    // 빌드 정보 저장
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now().to_rfc3339());
    
    // README 파일이 있는지 확인
    let readme_path = Path::new("README.md");
    if readme_path.exists() {
        println!("cargo:rerun-if-changed=README.md");
    }
} 