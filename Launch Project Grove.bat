@echo off
cd /d "%~dp0"
if not exist "Project Grove.exe" (
    call "Build Project Grove.bat"
    if errorlevel 1 exit /b 1
)
start "" "Project Grove.exe"
