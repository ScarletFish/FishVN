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
echo  3.  Package              (tauri build -^> nsis)
echo  4.  Deploy In-place      (覆盖热更, 免重装)
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
call npx tauri build
if %errorlevel% equ 0 (
    echo.
    echo ========================================
    echo  Build successful!
    echo ========================================
    if exist "src-tauri\target\release\bundle\nsis\" (
        start "" "src-tauri\target\release\bundle\nsis\"
    )
) else (
    echo.
    echo Build failed, error code: %errorlevel%
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
if not exist "src-tauri\target\release\fishvn.exe" (
    echo src-tauri\target\release\fishvn.exe not found.
    echo Run Package first via menu 3.
    echo.
    pause
    goto MENU
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
        timeout /t 1 /nobreak >nul
        echo Killed.
    ) else (
        echo Aborted. Close the app window first.
        echo.
        pause
        goto MENU
    )
    echo.
)
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
