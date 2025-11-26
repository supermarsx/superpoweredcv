@echo off
REM Navigate to the script's directory and then to the core directory
cd /d "%~dp0..\core"
REM Run the application
cargo run
