#!/usr/bin/env pwsh
# AI Resume Scanner - Stop Script
# Stops all running backend and frontend services

Write-Host "🛑 Stopping AI Resume Scanner..." -ForegroundColor Yellow
Write-Host ""

$stopped = $false

# Try to stop jobs from job IDs
if (Test-Path ".backend-job-id") {
    $backendJobId = Get-Content ".backend-job-id" -ErrorAction SilentlyContinue
    if ($backendJobId) {
        Write-Host "   Stopping backend job (ID: $backendJobId)..." -ForegroundColor Gray
        Stop-Job -Id $backendJobId -ErrorAction SilentlyContinue
        Remove-Job -Id $backendJobId -Force -ErrorAction SilentlyContinue
        $stopped = $true
    }
    Remove-Item ".backend-job-id" -Force -ErrorAction SilentlyContinue
}

if (Test-Path ".frontend-job-id") {
    $frontendJobId = Get-Content ".frontend-job-id" -ErrorAction SilentlyContinue
    if ($frontendJobId) {
        Write-Host "   Stopping frontend job (ID: $frontendJobId)..." -ForegroundColor Gray
        Stop-Job -Id $frontendJobId -ErrorAction SilentlyContinue
        Remove-Job -Id $frontendJobId -Force -ErrorAction SilentlyContinue
        $stopped = $true
    }
    Remove-Item ".frontend-job-id" -Force -ErrorAction SilentlyContinue
}

# Kill processes on port 3001 (backend)
Write-Host "   Checking port 3001 (backend)..." -ForegroundColor Gray
$backendProcs = Get-NetTCPConnection -LocalPort 3001 -ErrorAction SilentlyContinue
if ($backendProcs) {
    foreach ($proc in $backendProcs) {
        $processName = (Get-Process -Id $proc.OwningProcess -ErrorAction SilentlyContinue).Name
        Write-Host "   Killing process: $processName (PID: $($proc.OwningProcess))" -ForegroundColor Yellow
        Stop-Process -Id $proc.OwningProcess -Force -ErrorAction SilentlyContinue
        $stopped = $true
    }
}

# Kill processes on port 4321 (frontend)
Write-Host "   Checking port 4321 (frontend)..." -ForegroundColor Gray
$frontendProcs = Get-NetTCPConnection -LocalPort 4321 -ErrorAction SilentlyContinue
if ($frontendProcs) {
    foreach ($proc in $frontendProcs) {
        $processName = (Get-Process -Id $proc.OwningProcess -ErrorAction SilentlyContinue).Name
        Write-Host "   Killing process: $processName (PID: $($proc.OwningProcess))" -ForegroundColor Yellow
        Stop-Process -Id $proc.OwningProcess -Force -ErrorAction SilentlyContinue
        $stopped = $true
    }
}

# Also kill any cargo/node processes that might be lingering
Write-Host "   Cleaning up any lingering processes..." -ForegroundColor Gray
Get-Process | Where-Object { $_.ProcessName -match "resume-scanner|astro|node" } | ForEach-Object {
    Write-Host "   Killing: $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Yellow
    Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    $stopped = $true
}

Write-Host ""
if ($stopped) {
    Write-Host "✓ All services stopped successfully" -ForegroundColor Green
} else {
    Write-Host "ℹ️  No running services found" -ForegroundColor Cyan
}
Write-Host ""
