@echo off
chcp 65001 > nul
title Ricci CLI 설정

:: 관리자 권한 확인 및 요청
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo 관리자 권한으로 다시 실행합니다...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit
)

:MAIN_MENU
cls
echo.
echo ╔══════════════════════════════════════╗
echo ║        Ricci CLI 설정 도구           ║
echo ╚══════════════════════════════════════╝
echo.

:: Ricci 설치 확인
where ricci >nul 2>&1
if %errorlevel% equ 0 (
    echo [✓] Ricci가 설치되어 있습니다.
    for /f "tokens=*" %%i in ('where ricci') do echo     경로: %%i
) else (
    echo [✗] Ricci가 설치되어 있지 않습니다.
    echo     먼저 'cargo install --path .' 실행 필요
)

:: API 키 확인
echo.
set HAS_KEY=
if defined OPENAI_API_KEY (
    echo [✓] OpenAI API 키 설정됨
    set HAS_KEY=1
) else (
    echo [✗] OpenAI API 키 없음
)
if defined ANTHROPIC_API_KEY (
    echo [✓] Anthropic API 키 설정됨
    set HAS_KEY=1
) else (
    echo [✗] Anthropic API 키 없음
)
if defined GEMINI_API_KEY (
    echo [✓] Gemini API 키 설정됨
    set HAS_KEY=1
) else (
    echo [✗] Gemini API 키 없음
)

:: 우클릭 메뉴 확인
echo.
reg query "HKEY_CLASSES_ROOT\Directory\shell\Ricci" >nul 2>&1
if %errorlevel% equ 0 (
    echo [✓] 우클릭 메뉴가 설치되어 있습니다.
) else (
    echo [✗] 우클릭 메뉴가 설치되어 있지 않습니다.
)

echo.
echo ═══════════════════════════════════════
echo.
echo   1. 우클릭 메뉴 설치
echo   2. 우클릭 메뉴 제거
echo   3. API 키 설정
echo   4. 종료
echo.
choice /c 1234 /n /m "선택하세요: "

if errorlevel 4 exit
if errorlevel 3 goto SET_API_KEY
if errorlevel 2 goto UNINSTALL_MENU
if errorlevel 1 goto INSTALL_MENU

:INSTALL_MENU
cls
echo.
echo 우클릭 메뉴 설치 중...
echo.

:: Ricci 경로 찾기
set RICCI_PATH=
where ricci >nul 2>&1
if %errorlevel% equ 0 (
    for /f "tokens=*" %%i in ('where ricci') do set RICCI_PATH=%%i
) else if exist "%USERPROFILE%\.cargo\bin\ricci.exe" (
    set RICCI_PATH=%USERPROFILE%\.cargo\bin\ricci.exe
) else (
    echo [오류] Ricci를 찾을 수 없습니다!
    pause
    goto MAIN_MENU
)

:: 아이콘 경로
set ICON_PATH=%RICCI_PATH%
if exist "%~dp0assets\ricci.ico" set ICON_PATH=%~dp0assets\ricci.ico

:: 기존 제거
reg delete "HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" /f >nul 2>&1
reg delete "HKEY_CLASSES_ROOT\Directory\shell\Ricci" /f >nul 2>&1

:: 메뉴 추가
reg add "HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" /ve /d "여기서 Ricci 열기 🤖" /f >nul
reg add "HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" /v "Icon" /d "\"%ICON_PATH%\",0" /f >nul
reg add "HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci\command" /ve /d "cmd.exe /k cd /d \"%%V\" && \"%RICCI_PATH%\"" /f >nul

reg add "HKEY_CLASSES_ROOT\Directory\shell\Ricci" /ve /d "Ricci로 이 폴더 분석 📊" /f >nul
reg add "HKEY_CLASSES_ROOT\Directory\shell\Ricci" /v "Icon" /d "\"%ICON_PATH%\",0" /f >nul
reg add "HKEY_CLASSES_ROOT\Directory\shell\Ricci\command" /ve /d "cmd.exe /k cd /d \"%%1\" && \"%RICCI_PATH%\" analyze" /f >nul

echo.
echo [✓] 우클릭 메뉴가 설치되었습니다!
echo.
echo Explorer를 재시작합니다...
taskkill /f /im explorer.exe >nul 2>&1
start explorer.exe
timeout /t 2 >nul
goto MAIN_MENU

:UNINSTALL_MENU
cls
echo.
echo 우클릭 메뉴 제거 중...
reg delete "HKEY_CLASSES_ROOT\Directory\Background\shell\Ricci" /f >nul 2>&1
reg delete "HKEY_CLASSES_ROOT\Directory\shell\Ricci" /f >nul 2>&1
echo.
echo [✓] 우클릭 메뉴가 제거되었습니다!
timeout /t 2 >nul
goto MAIN_MENU

:SET_API_KEY
cls
echo.
echo ╔══════════════════════════════════════╗
echo ║         API 키 설정                  ║
echo ╚══════════════════════════════════════╝
echo.
echo 어떤 API 키를 설정하시겠습니까?
echo.
echo   1. OpenAI API 키
echo   2. Anthropic API 키  
echo   3. Gemini API 키
echo   4. 돌아가기
echo.
choice /c 1234 /n /m "선택하세요: "

if errorlevel 4 goto MAIN_MENU
if errorlevel 3 goto SET_GEMINI
if errorlevel 2 goto SET_ANTHROPIC
if errorlevel 1 goto SET_OPENAI

:SET_OPENAI
echo.
set /p KEY="OpenAI API 키 입력: "
if not "%KEY%"=="" (
    setx OPENAI_API_KEY "%KEY%" >nul
    echo [✓] OpenAI API 키가 설정되었습니다.
)
goto API_KEY_DONE

:SET_ANTHROPIC
echo.
set /p KEY="Anthropic API 키 입력: "
if not "%KEY%"=="" (
    setx ANTHROPIC_API_KEY "%KEY%" >nul
    echo [✓] Anthropic API 키가 설정되었습니다.
)
goto API_KEY_DONE

:SET_GEMINI
echo.
set /p KEY="Gemini API 키 입력: "
if not "%KEY%"=="" (
    setx GEMINI_API_KEY "%KEY%" >nul
    echo [✓] Gemini API 키가 설정되었습니다.
)

:API_KEY_DONE

echo.
echo 환경 변수가 적용되려면 새 터미널을 열어야 합니다.
pause
goto MAIN_MENU 