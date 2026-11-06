@echo off
setlocal
cd /d "%~dp0"

REM ============================================================
REM Communicator + Cloudflare Tunnel Deployment Script
REM ============================================================
REM This script starts the Rust backend and a Cloudflare tunnel
REM to expose it at communicator.newax.co
REM ============================================================

if not exist "backend-rs\target\release\backend-rs.exe" (
    echo [ERROR] Rust backend is not built. Building now...
    cargo build --release --manifest-path backend-rs\Cargo.toml
    if errorlevel 1 (
        echo [ERROR] Build failed. Fix the errors and try again.
        exit /b 1
    )
)

REM Load environment variables from .env if it exists
if exist ".env" (
    echo [INFO] Loading .env file...
    for /f "tokens=* delims=" %%a in (.env) do (
        set "%%a"
    )
)

REM Set default port
if not defined PORT set PORT=8000

echo [INFO] Starting Communicator Rust Backend on port %PORT%...
echo [INFO] Tunnel URL: https://communicator.newax.co
echo [INFO] Press Ctrl+C to stop both backend and tunnel
echo.

REM Start the Rust backend in the background
start "Communicator Backend" "backend-rs\target\release\backend-rs.exe"

REM Wait for the backend to be ready
echo [INFO] Waiting for backend to start...
:wait_for_backend
timeout /t 2 /nobreak >nul
curl -s http://127.0.0.1:%PORT%/api/v1/health >nul 2>&1
if errorlevel 1 (
    echo [INFO] Backend not ready yet...
    goto wait_for_backend
)

echo [INFO] Backend is ready!
echo [INFO] Starting Cloudflare tunnel...
echo.

REM Start cloudflared tunnel
cloudflared tunnel --config "%USERPROFILE%\.cloudflared\config.yml" run

exit /b %ERRORLEVEL%
