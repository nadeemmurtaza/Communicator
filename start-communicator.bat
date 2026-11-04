@echo off
setlocal
cd /d "%~dp0"
if not exist "backend-rs\target
elease\backend-rs.exe" (
    echo Rust backend is not built. Run: cargo build --release --manifest-path backend-rs\Cargo.toml
    exit /b 1
)

echo Starting native Rust backend...
set "PORT=8000"
"backend-rs\target
elease\backend-rs.exe"
exit /b %ERRORLEVEL%
