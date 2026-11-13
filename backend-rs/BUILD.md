# Build Communicator Backend

This script builds the Communicator Rust backend using WSL (Ubuntu).
Run this after WSL is back online and packages are installed.

## Prerequisites
1. WSL Ubuntu installed and running
2. Rust installed in WSL (`rustc --version` should work)
3. Build dependencies installed: `build-essential`, `pkg-config`, `libssl-dev`, `libpq-dev`

## Setup WSL (one-time)
```powershell
# Install Rust in WSL
wsl --exec bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

# Install build dependencies
wsl -u root -- bash -c "apt-get update && apt-get install -y build-essential pkg-config libssl-dev libpq-dev"
```

## Build
```powershell
# Build inside WSL
wsl --exec bash -c "cd /mnt/c/Users/nommi/Downloads/scraper/communicator/backend-rs && source \$HOME/.cargo/env && cargo build --release"
```

## Alternative: Docker Build
If Docker Desktop is installed:
```powershell
cd C:\Users\nommi\Downloads\scraper\communicator\backend-rs
docker build -t communicator-backend .
docker run -p 3000:3000 --env-file ..\.env communicator-backend
```

## Run
```powershell
# After building, the binary is at:
# target\release\backend-rs.exe
cd C:\Users\nommi\Downloads\scraper\communicator\backend-rs
.\target\release\backend-rs.exe
```
