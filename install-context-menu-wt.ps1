# Windows Terminal에서 Ricci 열기 옵션 추가
# Windows Terminal이 설치되어 있어야 합니다

param(
    [string]$RicciPath = ""
)

# 관리자 권한 확인
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "이 스크립트는 관리자 권한이 필요합니다!" -ForegroundColor Red
    Write-Host "PowerShell을 관리자 권한으로 다시 실행하세요." -ForegroundColor Yellow
    exit 1
}

# Windows Terminal 확인
$wtPath = Get-Command wt -ErrorAction SilentlyContinue
if (-not $wtPath) {
    Write-Host "Windows Terminal이 설치되어 있지 않습니다!" -ForegroundColor Red
    Write-Host "Microsoft Store에서 Windows Terminal을 설치하세요." -ForegroundColor Yellow
    exit 1
}

Write-Host "Windows Terminal 컨텍스트 메뉴 추가 중..." -ForegroundColor Green

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
        exit 1
    }
}

# Windows Terminal 아이콘 경로
$wtIconPath = "$env:LOCALAPPDATA\Microsoft\WindowsApps\wt.exe"

try {
    # Windows Terminal에서 Ricci 열기 (배경)
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\RicciWT" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\RicciWT" -Name "(Default)" -Value "Windows Terminal에서 Ricci 열기(&W)" -Force
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\RicciWT" -Name "Icon" -Value "`"$wtIconPath`",0" -Force
    
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\RicciWT\command" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\RicciWT\command" -Name "(Default)" -Value "wt -d `"%V`" powershell.exe -NoExit -Command `"& '$RicciPath' chat --context`"" -Force
    
    # Windows Terminal에서 폴더 분석 (디렉토리)
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciWT" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciWT" -Name "(Default)" -Value "Windows Terminal에서 Ricci로 분석(&W)" -Force
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciWT" -Name "Icon" -Value "`"$wtIconPath`",0" -Force
    
    New-Item -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciWT\command" -Force | Out-Null
    Set-ItemProperty -Path "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciWT\command" -Name "(Default)" -Value "wt -d `"%1`" powershell.exe -NoExit -Command `"& '$RicciPath' analyze`"" -Force
    
    Write-Host "`n✓ Windows Terminal 컨텍스트 메뉴 추가 완료!" -ForegroundColor Green
    Write-Host "`n이제 다음 옵션들도 사용할 수 있습니다:" -ForegroundColor Cyan
    Write-Host "  • 폴더 내 빈 공간 우클릭 → 'Windows Terminal에서 Ricci 열기'" -ForegroundColor White
    Write-Host "  • 폴더 우클릭 → 'Windows Terminal에서 Ricci로 분석'" -ForegroundColor White
    
} catch {
    Write-Host "오류 발생: $_" -ForegroundColor Red
    exit 1
} 