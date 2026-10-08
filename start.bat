@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
title FishVN
cd /d "%~dp0"

:CHECK_PORT
echo Checking for leftover dev process on port 5173...
set "STALE_PIDS="
for /f "tokens=5" %%p in ('netstat -ano ^| findstr "LISTENING" ^| findstr /C:":5173 "') do (
    echo !STALE_PIDS! | findstr /c:" %%p " >nul || set "STALE_PIDS=!STALE_PIDS! %%p"
)
if defined STALE_PIDS (
    echo.
    echo WARNING: Port 5173 is in use by PIDs:!STALE_PIDS!
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
echo  Q.  Quit
echo ========================================
echo.
set /p choice="Choice (1-2, Q): "
if "%choice%"=="1" goto DEV
if "%choice%"=="2" goto CHECK
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
echo  Vite:http://127.0.0.1:5173
echo  Window opens on its own; the frontend cannot
echo  run in a plain browser (no Tauri IPC there).
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

:EOF
endlocal