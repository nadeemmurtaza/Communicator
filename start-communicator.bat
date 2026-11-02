@echo off
cd /d "%~dp0"
if exist "backend-rs\target\release\backend-rs.exe" (
    echo Starting native Rust backend...
    set PORT=8000
    backend-rs\target\release\backend-rs.exe
) else (
    call .\.venv\Scripts\activate.bat
    python -m uvicorn communicator.api.app:app --host 0.0.0.0 --port 8000
)
