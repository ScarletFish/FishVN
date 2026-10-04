@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
title FishVN
cd /d "%~dp0"

:CHECK_PORTS
echo Checking for leftover dev processes (ports 5173, 4545)...
set "STALE_PIDS="
for %%P in (5173 4545) do (
    for /f "tokens=5" %%p in ('netstat -ano ^| findstr "LISTENING" ^| findstr /C:":%%P "') do (
        echo !STALE_PIDS! | findstr /c:" %%p " >nul || set "STALE_PIDS=!STALE_PIDS! %%p"
    )
)
if defined STALE_PIDS (
    echo.
    echo WARNING: Ports 5173/4545 are in use by PIDs:!STALE_PIDS!
    set /p KILL_STALE="Kill them now to free the ports? (Y/N): "
    if /i "!KILL_STALE!"=="Y" (
        for %%p in (!STALE_PIDS!) do taskkill /PID %%p /F >nul 2>&1
        echo Cleaned. Ports freed.
    ) else (
        echo Leaving them as-is. Ports may conflict on startup.
    )
    echo.
)

:MENU
cls
echo ========================================
echo   FishVN - Launcher
echo ========================================
echo.
echo  1.  Dev Mode             (server + Vite HMR)
echo  2.  Backend check        (typecheck + test)
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
echo  Backend:  http://127.0.0.1:4545
echo  Frontend: http://127.0.0.1:5173
echo.
echo  Press Ctrl+C to stop all services
echo ========================================
echo.
npm run dev
echo.
echo Dev server stopped.
pause
goto MENU

:KILL
cls
echo Killing stale processes on 5173/4545...
for %%P in (5173 4545) do (
    for /f "tokens=5" %%p in ('netstat -ano ^| findstr "LISTENING" ^| findstr /C:":%%P "') do (
        taskkill /PID %%p /F >nul 2>&1
        echo Killed PID %%p
    )
)
echo Done.
timeout /t 2 /nobreak >nul
goto MENU

:CHECK
cls
echo Backend typecheck + test...
echo.
npm run typecheck
npm test
echo.
pause
goto MENU

:EOF