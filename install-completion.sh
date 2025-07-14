#!/bin/bash

# Ricci CLI 자동완성 설치 스크립트

echo "Ricci CLI 자동완성 설치를 시작합니다..."

# 현재 쉘 감지
if [ -n "$BASH_VERSION" ]; then
    SHELL_TYPE="bash"
elif [ -n "$ZSH_VERSION" ]; then
    SHELL_TYPE="zsh"
else
    echo "지원하지 않는 쉘입니다. Bash 또는 Zsh만 지원합니다."
    exit 1
fi

echo "감지된 쉘: $SHELL_TYPE"

# ricci 실행 파일 경로 확인
RICCI_PATH=$(which ricci 2>/dev/null)
if [ -z "$RICCI_PATH" ]; then
    # 로컬 빌드 경로 확인
    if [ -f "./target/release/ricci" ]; then
        RICCI_PATH="./target/release/ricci"
    else
        echo "ricci 실행 파일을 찾을 수 없습니다."
        echo "먼저 'cargo build --release'를 실행하거나 ricci를 PATH에 추가하세요."
        exit 1
    fi
fi

echo "Ricci 경로: $RICCI_PATH"

# 자동완성 디렉토리 생성
COMPLETION_DIR="$HOME/.local/share/ricci"
mkdir -p "$COMPLETION_DIR"

# 자동완성 스크립트 생성
if [ "$SHELL_TYPE" = "bash" ]; then
    echo "Bash 자동완성 스크립트 생성 중..."
    "$RICCI_PATH" completion bash > "$COMPLETION_DIR/ricci.bash"
    
    # .bashrc에 추가
    if ! grep -q "ricci.bash" "$HOME/.bashrc" 2>/dev/null; then
        echo "" >> "$HOME/.bashrc"
        echo "# Ricci CLI 자동완성" >> "$HOME/.bashrc"
        echo "source $COMPLETION_DIR/ricci.bash" >> "$HOME/.bashrc"
        echo ".bashrc에 자동완성 설정을 추가했습니다."
    else
        echo "자동완성 설정이 이미 .bashrc에 있습니다."
    fi
    
    echo ""
    echo "설치 완료! 다음 명령어를 실행하여 즉시 적용하세요:"
    echo "  source ~/.bashrc"
    
elif [ "$SHELL_TYPE" = "zsh" ]; then
    echo "Zsh 자동완성 스크립트 생성 중..."
    "$RICCI_PATH" completion zsh > "$COMPLETION_DIR/_ricci"
    
    # .zshrc에 추가
    if ! grep -q "ricci completion" "$HOME/.zshrc" 2>/dev/null; then
        echo "" >> "$HOME/.zshrc"
        echo "# Ricci CLI 자동완성" >> "$HOME/.zshrc"
        echo "fpath=($COMPLETION_DIR \$fpath)" >> "$HOME/.zshrc"
        echo "autoload -U compinit && compinit" >> "$HOME/.zshrc"
        echo ".zshrc에 자동완성 설정을 추가했습니다."
    else
        echo "자동완성 설정이 이미 .zshrc에 있습니다."
    fi
    
    echo ""
    echo "설치 완료! 다음 명령어를 실행하여 즉시 적용하세요:"
    echo "  source ~/.zshrc"
fi

echo ""
echo "이제 다음과 같이 사용할 수 있습니다:"
echo "  ricci <Tab>           - 사용 가능한 명령어 보기"
echo "  ricci plan <Tab>      - plan 명령어 옵션 보기"
echo "  ricci analyze <Tab>   - analyze 명령어 옵션 보기" 