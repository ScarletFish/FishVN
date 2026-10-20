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
    set /p KILL_STALE="Kill it now to free the port? (Y/N): "
    if /i "!KILL_STALE!"=="Y" (
        for %%p in (!STALE_PIDS!) do taskkill /PID %%p /F >nul 2>&1
        echo Cleaned. Port freed.
    ) else (
        echo Leaving it as-is. Port may conflict on startup.
    )
    echo.
)

:MENU
cls
echo ========================================
echo   FishVN - Launcher
echo ========================================
echo.
echo  1.  Dev Mode             (tauri dev)
echo  2.  Backend check        (clippy + test)
echo  3.  Package              (tauri build -^> nsis)
echo  Q.  Quit
echo ========================================
echo.
set /p choice="Choice (1-3, Q): "
if "%choice%"=="1" goto DEV
if "%choice%"=="2" goto CHECK
if "%choice%"=="3" goto BUILD
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
npm run dev
echo.
echo Dev stopped.
pause
goto MENU

:CHECK
cls
echo Backend check (clippy + cargo test)...
echo.
npm run check:rust
npm test
echo.
pause
goto MENU

:BUILD
cls
echo ========================================
echo   Package
echo ========================================
echo.
tasklist /fi "imagename eq fishvn.exe" 2>nul | findstr /i "fishvn.exe" >nul
if not errorlevel 1 (
    echo WARNING: fishvn.exe is running and may lock target\release.
    echo Close the app window first.
    echo.
    pause
)
echo Refreshing build resources (icons / assets)...
if exist "src-tauri\target\release\build" (
    for /d %%d in ("src-tauri\target\release\build\fishvn-*") do rd /s /q "%%d" 2>nul
)
echo.
npx tauri build
echo.
echo ========================================
echo   exe    src-tauri\target\release\fishvn.exe
echo   setup  src-tauri\target\release\bundle\nsis\
echo ========================================
echo.
pause
goto MENU

:EOF
endlocal