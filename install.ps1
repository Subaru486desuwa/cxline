# install.ps1 - cxline Windows installer
# Usage: powershell -ExecutionPolicy Bypass -File install.ps1

$ErrorActionPreference = "Stop"

Write-Host "=== cxline installer (Windows) ===" -ForegroundColor Cyan
Write-Host ""

# Check cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cargo not found. Install Rust from https://rustup.rs first." -ForegroundColor Red
    exit 1
}

Write-Host "[1/3] Building and installing cxline..." -ForegroundColor Yellow
cargo install --path .
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: cargo install failed." -ForegroundColor Red
    exit 1
}

# Verify installation
if (-not (Get-Command cxline -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cxline not found in PATH after install. Ensure ~/.cargo/bin is in PATH." -ForegroundColor Red
    exit 1
}

Write-Host "[2/3] Running cxline setup..." -ForegroundColor Yellow
cxline setup
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: cxline setup failed." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[3/3] Done!" -ForegroundColor Green
Write-Host "Restart PowerShell, then type 'codex' to use with status in the title bar."
