# Ricci CLI PowerShell 자동완성 설치 스크립트

Write-Host "Ricci CLI PowerShell 자동완성 설치를 시작합니다..." -ForegroundColor Green

# ricci 실행 파일 경로 확인
$ricciPath = Get-Command ricci -ErrorAction SilentlyContinue
if (-not $ricciPath) {
    # 로컬 빌드 경로 확인
    if (Test-Path ".\target\release\ricci.exe") {
        $ricciPath = ".\target\release\ricci.exe"
    } else {
        Write-Host "ricci 실행 파일을 찾을 수 없습니다." -ForegroundColor Red
        Write-Host "먼저 'cargo build --release'를 실행하거나 ricci를 PATH에 추가하세요."
        exit 1
    }
} else {
    $ricciPath = $ricciPath.Path
}

Write-Host "Ricci 경로: $ricciPath"

# 자동완성 디렉토리 생성
$completionDir = "$env:USERPROFILE\.config\ricci"
if (-not (Test-Path $completionDir)) {
    New-Item -ItemType Directory -Path $completionDir -Force | Out-Null
}

# PowerShell 자동완성 스크립트 생성
Write-Host "PowerShell 자동완성 스크립트 생성 중..."
& $ricciPath completion powershell | Out-File "$completionDir\ricci-completion.ps1" -Encoding UTF8

# PowerShell 프로필 확인
$profilePath = $PROFILE.CurrentUserAllHosts
if (-not (Test-Path $profilePath)) {
    # 프로필이 없으면 생성
    New-Item -ItemType File -Path $profilePath -Force | Out-Null
}

# 프로필에 자동완성 추가
$completionLine = ". `"$completionDir\ricci-completion.ps1`""
$profileContent = Get-Content $profilePath -ErrorAction SilentlyContinue

if ($profileContent -notcontains $completionLine) {
    Add-Content -Path $profilePath -Value ""
    Add-Content -Path $profilePath -Value "# Ricci CLI 자동완성"
    Add-Content -Path $profilePath -Value $completionLine
    Write-Host "PowerShell 프로필에 자동완성 설정을 추가했습니다." -ForegroundColor Green
} else {
    Write-Host "자동완성 설정이 이미 프로필에 있습니다." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "설치 완료!" -ForegroundColor Green
Write-Host "다음 명령어를 실행하여 즉시 적용하세요:" -ForegroundColor Cyan
Write-Host "  . `$PROFILE"
Write-Host ""
Write-Host "또는 PowerShell을 다시 시작하세요."
Write-Host ""
Write-Host "이제 다음과 같이 사용할 수 있습니다:" -ForegroundColor Yellow
Write-Host "  ricci <Tab>           - 사용 가능한 명령어 보기"
Write-Host "  ricci plan <Tab>      - plan 명령어 옵션 보기"
Write-Host "  ricci analyze <Tab>   - analyze 명령어 옵션 보기" 