# Ricci CLI

> AI 기반 개발 어시스턴트 - Gemini CLI 스타일의 대화형 개발 도구

## 개요

Ricci CLI는 개발자를 위한 AI 어시스턴트입니다. 대화형 인터페이스를 통해 코드 리뷰, 작업계획서 생성, 프로젝트 분석, 문서 작성 등을 지원합니다.

## 주요 기능

### 🤖 대화형 AI 어시스턴트
- **스트리밍 응답**: 실시간으로 AI 응답을 확인
- **컨텍스트 인식**: 프로젝트 구조와 파일을 이해하고 맞춤형 조언 제공
- **마크다운 렌더링**: 코드 블록 구문 강조와 깔끔한 포맷팅

### 📋 작업계획서 생성
- **자동 태스크 분해**: 프로젝트 설명을 기반으로 상세한 작업 계획 생성
- **일정 추정**: 각 단계별 예상 소요 시간 계산
- **위험 관리**: 잠재적 위험 요소와 대응 방안 제시
- **다양한 출력 형식**: Markdown, JSON, YAML 지원

### 🔍 프로젝트 분석
- **구조 분석**: 파일 구성, 언어별 통계, 디렉토리 구조 파악
- **의존성 분석**: 사용 중인 라이브러리와 버전 확인
- **복잡도 분석**: 코드 복잡도 측정 및 개선점 제안
- **코드 스멜 탐지**: 리팩토링이 필요한 부분 식별

### 📝 코드 리뷰 & 문서화
- **자동 코드 리뷰**: 보안, 성능, 스타일 기준으로 코드 평가
- **문서 자동 생성**: API 문서, README, 아키텍처 문서 생성
- **모범 사례 제안**: 코드 품질 향상을 위한 구체적인 조언

## 설치

### 사전 요구사항

- Rust 1.70 이상
- OpenAI API 키 (또는 Anthropic/Gemini API 키)

### 빌드

```bash
git clone https://github.com/yourusername/ricci-cli
cd ricci-cli
cargo build --release
```

### 환경 설정

#### 방법 1: .env 파일 사용 (권장)

```bash
# env.example을 복사해서 .env 파일 생성
cp env.example .env

# .env 파일 편집
# OPENAI_API_KEY=sk-your-actual-api-key-here
```

#### 방법 2: 환경 변수 설정

```bash
# 환경 변수 설정
export OPENAI_API_KEY="your-api-key"
```

#### 방법 3: 설정 명령어 사용

```bash
# 설정 명령어로 API 키 저장
ricci config set-key openai "your-api-key"
```

#### 방법 4: REST API에서 가져오기 (준비 중)

```bash
# config.toml에서 설정
[api_key_source]
rest_api = { url = "https://your-api-server.com/api/keys" }
```

## 사용법

### 기본 대화형 모드

```bash
# 대화형 모드 시작
ricci

# 프로젝트 컨텍스트와 함께 시작
ricci chat --context

# 세션 저장
ricci chat --save session.json
```

### 직접 질문

```bash
# 단일 질문
ricci "이 프로젝트의 구조를 설명해줘"
```

### 작업계획서 생성

```bash
# 기본 계획서
ricci plan "React로 Todo 앱 만들기"

# 상세 계획서 with 일정 추정
ricci plan "전자상거래 플랫폼 구축" --detail 4 --estimate

# JSON 형식으로 출력
ricci plan "API 서버 개발" --format json
```

### 프로젝트 분석

```bash
# 전체 분석
ricci analyze

# 특정 분석만 수행
ricci analyze --type structure
ricci analyze --type dependencies
ricci analyze --type complexity

# 특정 디렉토리 분석
ricci analyze ./src
```

### 코드 리뷰

```bash
# 파일 리뷰
ricci review main.rs

# 특정 기준으로 리뷰
ricci review src/ --criteria security
ricci review src/ --criteria performance
```

### 문서 생성

```bash
# README 생성
ricci doc myproject --type readme

# API 문서 생성
ricci doc api/ --type api

# 아키텍처 문서
ricci doc . --type architecture
```

## 대화형 모드 명령어

대화형 모드에서 사용할 수 있는 특수 명령어:

- `/clear` - 대화 컨텍스트 초기화
- `/context` - 현재 컨텍스트 확인
- `/save` - 현재 세션 저장
- `/help` - 도움말 표시

## 설정

### API 키 관리

```bash
# API 키 설정
ricci config set-key openai "sk-..."
ricci config set-key anthropic "sk-ant-..."
ricci config set-key gemini "AIza..."

# 설정 확인
ricci config show

# 설정 초기화
ricci config reset
```

### 설정 파일

설정은 `~/.config/ricci/config.toml`에 저장됩니다:

```toml
openai_api_key = "sk-..."
anthropic_api_key = "sk-ant-..."
gemini_api_key = "AIza..."

[model_preferences]
default_provider = "openai"
default_model = "gpt-4-turbo-preview"
temperature = 0.7
max_tokens = 2000

[output_preferences]
syntax_highlighting = true
markdown_rendering = true
auto_save_sessions = false
session_dir = "/home/user/.local/share/ricci/sessions"
```

## 예제

### 작업계획서 생성 예제

```bash
$ ricci plan "마이크로서비스 아키텍처로 블로그 플랫폼 구축" --detail 3 --estimate

# 블로그 플랫폼 구축 계획

## 목표
- 확장 가능한 마이크로서비스 아키텍처 구현
- 사용자 인증 및 권한 관리
- 포스트 작성/편집/삭제 기능
- 댓글 시스템
- 검색 기능

## 단계별 계획

### 1단계: 아키텍처 설계 및 환경 구성
예상 기간: 3-5일 (평균 4일)

#### 주요 작업
- [ ] 마이크로서비스 아키텍처 설계 (보통)
  - [ ] 서비스 경계 정의
  - [ ] API 게이트웨이 설계
  - [ ] 서비스 간 통신 방식 결정
...
```

### 프로젝트 분석 예제

```bash
$ ricci analyze

프로젝트 구조 분석
==================================================
총 파일 수: 42
총 코드 라인: 3,847

언어별 통계:
  Rust - 35 파일, 3,215 라인 (83.6%)
  TOML - 3 파일, 256 라인 (6.7%)
  Markdown - 4 파일, 376 라인 (9.8%)

주요 디렉토리:
  📁 src/ (35 파일) - 소스 코드
  📁 src/api/ (5 파일) - API 코드
  📁 src/assistant/ (8 파일) - 어시스턴트 로직
...
```

## 기여하기

프로젝트에 기여하고 싶으시다면:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 라이선스

MIT License - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 문의

- Issue Tracker: [GitHub Issues](https://github.com/yourusername/ricci-cli/issues)
- Email: your.email@example.com
