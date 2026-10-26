@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
title FishVN
cd /d "%~dp0"

:CHECK_PORT
echo Checking for leftover dev process on port 4545...
set "STALE_PIDS="
for /f "tokens=5" %%p in ('netstat -ano ^| findstr "LISTENING" ^| findstr /C:":4545 "') do (
    echo !STALE_PIDS! | findstr /c:" %%p " >nul || set "STALE_PIDS=!STALE_PIDS! %%p"
)
if defined STALE_PIDS (
    echo.
    echo WARNING: Port 4545 is in use by PIDs:!STALE_PIDS!
    set /p KILL_STALE="Kill them now to free the port? (Y/N): "
    if /i "!KILL_STALE!"=="Y" (
        for %%p in (!STALE_PIDS!) do taskkill /PID %%p /F >nul 2>&1
        echo Cleaned. Port freed.
    ) else (
        echo Leaving them as-is. Port may conflict on startup.
    )
    echo.
)
set "STALE_PIDS="

:MENU
cls
echo ========================================
echo   FishVN - Launcher
echo ========================================
echo.
echo  1.  Dev Mode             (tauri dev)
echo  2.  Backend check        (clippy + test)
echo  3.  Package              (发版档: 开 LTO, 出 NSIS 安装包)
echo  4.  Deploy In-place      (只编 exe, 覆盖热更, 免重装)
echo.
echo  Q.  Quit
echo ========================================
echo.
set /p choice="Choice (1-4, Q): "
if "%choice%"=="1" goto DEV
if "%choice%"=="2" goto CHECK
if "%choice%"=="3" goto BUILD
if "%choice%"=="4" goto DEPLOY
if /i "%choice%"=="Q" goto EOF
echo.
echo Invalid choice
timeout /t 2 /nobreak >nul
goto MENU

:DEV
cls
echo ========================================
echo   Dev Mode
echo ========================================
echo.
echo  Vite:http://127.0.0.1:4545
echo  Window opens on its own; to debug in a browser
echo  open http://127.0.0.1:4545/?dev-http
echo.
echo  Press Ctrl+C to stop
echo ========================================
echo.
call npm run dev
echo.
echo Dev stopped.
pause
goto MENU

:CHECK
cls
echo ========================================
echo   Backend check
echo ========================================
echo.
call npm run check:rust
call npm test
echo.
pause
goto MENU

:BUILD
cls
echo ========================================
echo   Package
echo ========================================
echo.
echo  Output: src-tauri\target\release\bundle\nsis\
echo  Profile: LTO on, codegen-units 1, opt-level 3 (this run only)
echo  Requires Rust toolchain, ~5-10 min
echo ========================================
echo.
tasklist /fi "imagename eq fishvn.exe" 2>nul | findstr /i "fishvn.exe" >nul
if not errorlevel 1 (
    echo WARNING: fishvn.exe is running and may lock target\release.
    echo Close the app window first.
    echo.
    pause
)
echo Refreshing build resources...
if exist "src-tauri\target\release\build" (
    for /d %%d in ("src-tauri\target\release\build\fishvn-*") do rd /s /q "%%d" 2>nul
)
echo.
echo Press any key to start build, or close window to cancel...
pause >nul
set "CARGO_PROFILE_RELEASE_LTO=true"
set "CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1"
set "CARGO_PROFILE_RELEASE_OPT_LEVEL=3"
call npx tauri build
set "BUILD_RC=%errorlevel%"
set "CARGO_PROFILE_RELEASE_LTO="
set "CARGO_PROFILE_RELEASE_CODEGEN_UNITS="
set "CARGO_PROFILE_RELEASE_OPT_LEVEL="
if %BUILD_RC% equ 0 (
    echo.
    echo ========================================
    echo  Build successful!
    echo ========================================
    if exist "src-tauri\target\release\bundle\nsis\" (
        start "" "src-tauri\target\release\bundle\nsis\"
    )
) else (
    echo.
    echo Build failed, error code: %BUILD_RC%
)
echo.
pause
goto MENU

:DEPLOY
set "INSTALL_DIR="
if exist "%LOCALAPPDATA%\FishVN\fishvn.exe" set "INSTALL_DIR=%LOCALAPPDATA%\FishVN"
if not defined INSTALL_DIR if exist "%ProgramFiles%\FishVN\fishvn.exe" set "INSTALL_DIR=%ProgramFiles%\FishVN"
if not defined INSTALL_DIR if exist "%ProgramFiles(x86)%\FishVN\fishvn.exe" set "INSTALL_DIR=%ProgramFiles(x86)%\FishVN"
if not defined INSTALL_DIR (
    echo No FishVN installation found.
    echo Expected: %LOCALAPPDATA%\FishVN\fishvn.exe
    echo.
    pause
    goto MENU
)
set "NEED_BUILD="
if not exist "src-tauri\target\release\fishvn.exe" set "NEED_BUILD=1"
if not defined NEED_BUILD (
    powershell -NoProfile -Command "$s=(Get-ChildItem -Recurse -File 'frontend\src','frontend\index.html','src-tauri\src','src-tauri\build.rs','src-tauri\tauri.conf.json' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1).LastWriteTime; if($s -gt (Get-Item 'src-tauri\target\release\fishvn.exe').LastWriteTime){exit 1}"
    if errorlevel 1 set "NEED_BUILD=1"
)
if defined NEED_BUILD (
    echo target\release\fishvn.exe is missing or older than the sources.
    set /p BUILD_NOW="Build exe only - skip NSIS - then deploy? (Y/N): "
    if /i not "!BUILD_NOW!"=="Y" goto MENU
    echo.
    call npx tauri build --no-bundle
    if !errorlevel! neq 0 (
        echo Build failed, error code: !errorlevel!
        echo.
        pause
        goto MENU
    )
    echo.
    if not exist "src-tauri\target\release\fishvn.exe" (
        echo Build finished but the exe is missing, check the output above.
        echo.
        pause
        goto MENU
    )
)
cls
echo ========================================
echo   Deploy In-place - 覆盖热更
echo ========================================
echo  目标: %INSTALL_DIR%
echo  备份: %INSTALL_DIR%\backup\ 保留最近 2 份
echo  数据: %APPDATA%\FishVN 不受影响
echo ========================================
echo.
tasklist /fi "imagename eq fishvn.exe" 2>nul | findstr /i "fishvn.exe" >nul
if not errorlevel 1 (
    set /p KILL_APP="fishvn.exe is running. Kill it now? (Y/N): "
    if /i "!KILL_APP!"=="Y" (
        taskkill /im fishvn.exe /f >nul 2>&1
        set "KILLED_PID=1"
        echo Killed.
    ) else (
        echo Aborted. Close the app window first.
        echo.
        pause
        goto MENU
    )
    echo.
)
if not defined KILLED_PID goto AFTER_KILL
set "KILL_WAIT=0"
:WAITKILL
timeout /t 1 /nobreak >nul
tasklist /fi "imagename eq fishvn.exe" 2>nul | findstr /i "fishvn.exe" >nul
if errorlevel 1 goto AFTER_KILL
set /a KILL_WAIT+=1
if %KILL_WAIT% lss 5 goto WAITKILL
echo WARNING: fishvn.exe still running, deploy may fail.
:AFTER_KILL
echo Press any key to deploy, or close window to cancel...
pause >nul
if not exist "%INSTALL_DIR%\backup" md "%INSTALL_DIR%\backup"
if exist "%INSTALL_DIR%\backup\fishvn.exe.bak-2" del /q "%INSTALL_DIR%\backup\fishvn.exe.bak-2"
if exist "%INSTALL_DIR%\backup\fishvn.exe.bak-1" ren "%INSTALL_DIR%\backup\fishvn.exe.bak-1" "fishvn.exe.bak-2"
copy /y "%INSTALL_DIR%\fishvn.exe" "%INSTALL_DIR%\backup\fishvn.exe.bak-1" >nul
copy /y "src-tauri\target\release\fishvn.exe" "%INSTALL_DIR%\fishvn.exe" >nul
if %errorlevel% equ 0 (
    rd /s /q "%INSTALL_DIR%\EBWebView\Default\Cache" 2>nul
    rd /s /q "%INSTALL_DIR%\EBWebView\Default\Code Cache" 2>nul
    rd /s /q "%INSTALL_DIR%\EBWebView\Default\GPUCache" 2>nul
    rd /s /q "%INSTALL_DIR%\EBWebView\GPUCache" 2>nul
    echo WebView2 cache cleared.
    echo.
    echo ========================================
    echo  Deploy successful!
    echo ========================================
    set /p LAUNCH_NOW="Launch FishVN now? (Y/N): "
    if /i "!LAUNCH_NOW!"=="Y" start "" "%INSTALL_DIR%\fishvn.exe"
) else (
    echo.
    echo Deploy failed, error code: %errorlevel%
    echo Previous exe kept at %INSTALL_DIR%\backup\fishvn.exe.bak-1
)
echo.
pause
goto MENU

:EOF
endlocal
