# Windows 컨텍스트 메뉴에서 Ricci CLI 제거
# 관리자 권한으로 실행해야 합니다

# 관리자 권한 확인
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "이 스크립트는 관리자 권한이 필요합니다!" -ForegroundColor Red
    Write-Host "PowerShell을 관리자 권한으로 다시 실행하세요." -ForegroundColor Yellow
    exit 1
}

Write-Host "Ricci CLI 컨텍스트 메뉴 제거를 시작합니다..." -ForegroundColor Yellow

# 제거할 레지스트리 경로
$regPaths = @(
    "Registry::HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci",
    "Registry::HKEY_CLASSES_ROOT\Directory\shell\Ricci",
    "Registry::HKEY_CLASSES_ROOT\Directory\shell\RicciPlan"
)

# 레지스트리 항목 제거
$removed = 0
foreach ($path in $regPaths) {
    if (Test-Path $path) {
        try {
            Remove-Item -Path $path -Recurse -Force
            Write-Host "제거됨: $path" -ForegroundColor Green
            $removed++
        } catch {
            Write-Host "제거 실패: $path - $_" -ForegroundColor Red
        }
    } else {
        Write-Host "이미 제거됨: $path" -ForegroundColor Gray
    }
}

if ($removed -gt 0) {
    Write-Host "`n✓ 컨텍스트 메뉴 제거 완료!" -ForegroundColor Green
    Write-Host "총 $removed 개의 메뉴 항목이 제거되었습니다." -ForegroundColor Cyan
} else {
    Write-Host "`nRicci 컨텍스트 메뉴가 설치되어 있지 않습니다." -ForegroundColor Yellow
}

Write-Host "`n참고: 변경사항이 즉시 반영되지 않으면 Explorer를 재시작하세요." -ForegroundColor Yellow 