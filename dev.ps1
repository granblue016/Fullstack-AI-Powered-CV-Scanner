#!/usr/bin/env pwsh
# AI Resume Scanner - Development Mode Script
# Starts backend in separate window, frontend in current window

Write-Host "🔧 Starting AI Resume Scanner (Development Mode)" -ForegroundColor Cyan
Write-Host ""

# Check if .env exists in backend
if (-not (Test-Path "backend\.env")) {
    Write-Host "⚠️  Warning: backend\.env not found!" -ForegroundColor Yellow
    Write-Host "   Creating template .env file..." -ForegroundColor Yellow
    @"
GEMINI_API_KEY=your-api-key-here
PORT=3001
RUST_LOG=info
"@ | Out-File -FilePath "backend\.env" -Encoding utf8
    Write-Host "   ✓ Created backend\.env - Please add your Gemini API key" -ForegroundColor Green
    Write-Host ""
}

# Function to check if port is in use
function Test-Port {
    param($Port)
    $connection = Test-NetConnection -ComputerName localhost -Port $Port -InformationLevel Quiet -WarningAction SilentlyContinue
    return $connection
}

# Check if ports are already in use
if (Test-Port 3001) {
    Write-Host "⚠️  Port 3001 (backend) is already in use!" -ForegroundColor Yellow
    $response = Read-Host "Stop existing service? (y/n)"
    if ($response -eq 'y') {
        & ".\stop.ps1"
        Start-Sleep -Seconds 2
    } else {
        exit 1
    }
}

if (Test-Port 4321) {
    Write-Host "⚠️  Port 4321 (frontend) is already in use!" -ForegroundColor Yellow
    $response = Read-Host "Stop existing service? (y/n)"
    if ($response -eq 'y') {
        & ".\stop.ps1"
        Start-Sleep -Seconds 2
    } else {
        exit 1
    }
}

# Start backend in new window
Write-Host "📦 Starting backend in new window..." -ForegroundColor Blue
$backendPath = Join-Path $PSScriptRoot "backend"
Start-Process pwsh -ArgumentList "-NoExit", "-Command", "cd '$backendPath'; Write-Host '🦀 Backend (Rust + Axum)' -ForegroundColor Blue; Write-Host 'Press Ctrl+C to stop' -ForegroundColor Yellow; Write-Host ''; cargo run"

Write-Host "   Waiting for backend to start..." -ForegroundColor Gray
Start-Sleep -Seconds 5

# Check if backend is running
$maxAttempts = 10
$attempt = 0
while ($attempt -lt $maxAttempts) {
    if (Test-Port 3001) {
        Write-Host "   ✓ Backend is running on http://localhost:3001" -ForegroundColor Green
        break
    }
    $attempt++
    Start-Sleep -Seconds 1
}

if ($attempt -eq $maxAttempts) {
    Write-Host "   ⚠️  Backend might not have started. Check the backend window." -ForegroundColor Yellow
}

Write-Host ""

# Start frontend in current window
Write-Host "🎨 Starting frontend (Astro + React)..." -ForegroundColor Magenta
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "Development Mode Active" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "🌐 Frontend:  http://localhost:4321 (this window)" -ForegroundColor Cyan
Write-Host "📡 Backend:   http://localhost:3001 (separate window)" -ForegroundColor Blue
Write-Host ""
Write-Host "Press Ctrl+C in this window to stop frontend" -ForegroundColor Yellow
Write-Host "Press Ctrl+C in backend window to stop backend" -ForegroundColor Yellow
Write-Host ""

Push-Location frontend
try {
    npm run dev
}
finally {
    Pop-Location
    Write-Host ""
    Write-Host "Frontend stopped. Backend is still running in separate window." -ForegroundColor Yellow
    Write-Host "Run 'stop.ps1' to stop all services" -ForegroundColor Yellow
}
