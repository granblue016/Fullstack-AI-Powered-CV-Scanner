# AI Resume Scanner - Start Script
# Starts both backend and frontend services

Write-Host "Starting AI Resume Scanner..." -ForegroundColor Cyan
Write-Host ""

# Check if .env exists in backend
if (-not (Test-Path "backend\.env")) {
    Write-Host "Warning: backend\.env not found!" -ForegroundColor Yellow
    Write-Host "Creating template .env file..." -ForegroundColor Yellow
    @"
GEMINI_API_KEY=your-api-key-here
PORT=3001
RUST_LOG=info
"@ | Out-File -FilePath "backend\.env" -Encoding utf8
    Write-Host "Created backend\.env - Please add your Gemini API key" -ForegroundColor Green
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
    Write-Host "WARNING: Port 3001 (backend) is already in use!" -ForegroundColor Yellow
    Write-Host "Run 'stop.ps1' to stop existing services" -ForegroundColor Yellow
    exit 1
}

if (Test-Port 4321) {
    Write-Host "WARNING: Port 4321 (frontend) is already in use!" -ForegroundColor Yellow
    Write-Host "Run 'stop.ps1' to stop existing services" -ForegroundColor Yellow
    exit 1
}

# Start backend in background
Write-Host "Starting backend (Rust)..." -ForegroundColor Blue
Push-Location backend
$backendJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    cargo run --release 2>&1
}
Pop-Location

# Wait a bit for backend to start
Write-Host "Waiting for backend to initialize..." -ForegroundColor Gray
Start-Sleep -Seconds 3

# Check if backend started successfully
if ($backendJob.State -eq "Running") {
    Write-Host "[OK] Backend started on http://localhost:3001" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Backend failed to start!" -ForegroundColor Red
    Write-Host "Check backend logs with: Receive-Job $($backendJob.Id)" -ForegroundColor Yellow
    Stop-Job $backendJob
    Remove-Job $backendJob
    exit 1
}

Write-Host ""

# Start frontend in background
Write-Host "Starting frontend (Astro)..." -ForegroundColor Magenta
Push-Location frontend
$frontendJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    npm run dev 2>&1
}
Pop-Location

# Wait for frontend to start
Write-Host "Waiting for frontend to initialize..." -ForegroundColor Gray
Start-Sleep -Seconds 5

# Check if frontend started successfully
if ($frontendJob.State -eq "Running") {
    Write-Host "[OK] Frontend started on http://localhost:4321" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Frontend failed to start!" -ForegroundColor Red
    Write-Host "Check frontend logs with: Receive-Job $($frontendJob.Id)" -ForegroundColor Yellow
    Stop-Job $backendJob, $frontendJob
    Remove-Job $backendJob, $frontendJob
    exit 1
}

Write-Host ""
Write-Host "===============================================" -ForegroundColor Green
Write-Host "AI Resume Scanner is now running!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Frontend:  http://localhost:4321" -ForegroundColor Cyan
Write-Host "Backend:   http://localhost:3001" -ForegroundColor Blue
Write-Host ""
Write-Host "Press Ctrl+C to stop all services" -ForegroundColor Yellow
Write-Host ""

# Store job IDs for stop script
$backendJob.Id | Out-File -FilePath ".backend-job-id" -Encoding utf8
$frontendJob.Id | Out-File -FilePath ".frontend-job-id" -Encoding utf8

# Monitor jobs and show logs
try {
    while ($true) {
        # Check if jobs are still running
        if ($backendJob.State -ne "Running") {
            Write-Host "`n[ERROR] Backend stopped unexpectedly!" -ForegroundColor Red
            Receive-Job $backendJob
            break
        }
        if ($frontendJob.State -ne "Running") {
            Write-Host "`n[ERROR] Frontend stopped unexpectedly!" -ForegroundColor Red
            Receive-Job $frontendJob
            break
        }
        
        # Show recent output
        $backendOutput = Receive-Job $backendJob
        if ($backendOutput) {
            Write-Host "[Backend] $backendOutput" -ForegroundColor Blue
        }
        
        $frontendOutput = Receive-Job $frontendJob
        if ($frontendOutput) {
            Write-Host "[Frontend] $frontendOutput" -ForegroundColor Magenta
        }
        
        Start-Sleep -Seconds 2
    }
}
finally {
    # Cleanup on exit
    Write-Host "`nStopping services..." -ForegroundColor Yellow
    Stop-Job $backendJob, $frontendJob -ErrorAction SilentlyContinue
    Remove-Job $backendJob, $frontendJob -Force -ErrorAction SilentlyContinue
    
    # Kill any remaining processes on the ports
    $processes = Get-NetTCPConnection -LocalPort 3001, 4321 -ErrorAction SilentlyContinue
    foreach ($proc in $processes) {
        Stop-Process -Id $proc.OwningProcess -Force -ErrorAction SilentlyContinue
    }
    
    # Remove job ID files
    Remove-Item ".backend-job-id", ".frontend-job-id" -Force -ErrorAction SilentlyContinue
    
    Write-Host "[OK] All services stopped" -ForegroundColor Green
}
