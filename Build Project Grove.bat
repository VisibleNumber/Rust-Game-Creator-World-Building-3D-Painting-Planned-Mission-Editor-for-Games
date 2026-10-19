@echo off
cd /d "%~dp0"
cargo build --release
if errorlevel 1 (
    pause
    exit /b 1
)
copy /y "target\release\project-grove.exe" "Project Grove.exe"
if errorlevel 1 (
    echo Close Project Grove before rebuilding the launcher executable.
    pause
    exit /b 1
)
