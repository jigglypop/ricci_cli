# Windows 컨텍스트 메뉴에 Ricci CLI 추가
# 관리자 권한으로 실행해야 합니다

param(
    [string]$RicciPath = ""
)

# 관리자 권한 확인
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "이 스크립트는 관리자 권한이 필요합니다!" -ForegroundColor Red
    Write-Host "PowerShell을 관리자 권한으로 다시 실행하세요." -ForegroundColor Yellow
    exit 1
}

Write-Host "Ricci CLI 컨텍스트 메뉴 설치를 시작합니다..." -ForegroundColor Green

# Ricci 실행 파일 경로 찾기
if ([string]::IsNullOrEmpty($RicciPath)) {
    # PATH에서 찾기
    $ricciCmd = Get-Command ricci -ErrorAction SilentlyContinue
    if ($ricciCmd) {
        $RicciPath = $ricciCmd.Path
    }
    # 현재 디렉토리에서 찾기
    elseif (Test-Path ".\target\release\ricci.exe") {
        $RicciPath = (Get-Item ".\target\release\ricci.exe").FullName
    }
    # 설치된 경로에서 찾기
    elseif (Test-Path "$env:USERPROFILE\.cargo\bin\ricci.exe") {
        $RicciPath = "$env:USERPROFILE\.cargo\bin\ricci.exe"
    }
    else {
        Write-Host "Ricci 실행 파일을 찾을 수 없습니다!" -ForegroundColor Red
        Write-Host "경로를 직접 지정하세요: .\install-context-menu.ps1 -RicciPath 'C:\path\to\ricci.exe'" -ForegroundColor Yellow
        exit 1
    }
}

if (-not (Test-Path $RicciPath)) {
    Write-Host "지정된 경로에 Ricci가 없습니다: $RicciPath" -ForegroundColor Red
    exit 1
}

Write-Host "Ricci 경로: $RicciPath" -ForegroundColor Cyan

# 아이콘 경로 (ricci.exe 자체를 아이콘으로 사용)
$IconPath = $RicciPath

# 레지스트리 경로
$regPaths = @(
    # 디렉토리 배경 (빈 공간 우클릭)
    "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci",
    "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci\command",
    
    # 디렉토리 (폴더 우클릭)
    "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci",
    "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci\command"
)

# 레지스트리 항목 생성
try {
    # 디렉토리 배경 메뉴
    Write-Host "디렉토리 배경 메뉴 추가 중..." -ForegroundColor Yellow
    
    # 메뉴 항목 생성
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" -Name "(Default)" -Value "여기서 Ricci 열기(&R)" -Force
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" -Name "Icon" -Value "`"$IconPath`",0" -Force
    
    # 명령 생성
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci\command" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci\command" -Name "(Default)" -Value "powershell.exe -NoExit -Command `"cd '%V'; & '$RicciPath' chat --context`"" -Force
    
    # 디렉토리 메뉴
    Write-Host "디렉토리 메뉴 추가 중..." -ForegroundColor Yellow
    
    # 메뉴 항목 생성
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci" -Name "(Default)" -Value "Ricci로 이 폴더 분석(&R)" -Force
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci" -Name "Icon" -Value "`"$IconPath`",0" -Force
    
    # 명령 생성
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci\command" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci\command" -Name "(Default)" -Value "powershell.exe -NoExit -Command `"cd '%1'; & '$RicciPath' analyze`"" -Force
    
    # 추가 메뉴 항목들
    Write-Host "추가 메뉴 항목 생성 중..." -ForegroundColor Yellow
    
    # "작업계획서 생성" 메뉴
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciPlan" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciPlan" -Name "(Default)" -Value "Ricci로 작업계획서 생성(&P)" -Force
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciPlan" -Name "Icon" -Value "`"$IconPath`",0" -Force
    
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciPlan\command" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciPlan\command" -Name "(Default)" -Value "powershell.exe -NoExit -Command `"cd '%1'; Write-Host '프로젝트 설명을 입력하세요:' -ForegroundColor Cyan; `$desc = Read-Host; & '$RicciPath' plan `"`$desc`" --detail 3 --estimate`"" -Force
    
    Write-Host "`n✓ 컨텍스트 메뉴 설치 완료!" -ForegroundColor Green
    Write-Host "`n이제 Windows Explorer에서:" -ForegroundColor Cyan
    Write-Host "  • 폴더 내 빈 공간 우클릭 → '여기서 Ricci 열기'" -ForegroundColor White
    Write-Host "  • 폴더 우클릭 → 'Ricci로 이 폴더 분석'" -ForegroundColor White
    Write-Host "  • 폴더 우클릭 → 'Ricci로 작업계획서 생성'" -ForegroundColor White
    Write-Host "`n참고: 변경사항이 즉시 표시되지 않으면 Explorer를 재시작하세요." -ForegroundColor Yellow
    
} catch {
    Write-Host "오류 발생: $_" -ForegroundColor Red
    exit 1
} 