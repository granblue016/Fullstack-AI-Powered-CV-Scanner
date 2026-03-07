# Docker deployment script for Windows PowerShell

param(
    [string]$action = "up",
    [switch]$detach = $false,
    [switch]$build = $false
)

$validActions = @("up", "down", "logs", "build", "restart", "ps", "shell")

if ($action -notin $validActions) {
    Write-Host "Invalid action: $action"
    Write-Host "Valid actions: $($validActions -join ', ')"
    exit 1
}

Write-Host "🐳 Docker Manager - AI Resume Scanner" -ForegroundColor Cyan
Write-Host "Action: $action" -ForegroundColor Yellow

switch ($action) {
    "up" {
        Write-Host "Starting services..." -ForegroundColor Green
        if ($build) {
            docker-compose up --build $(if ($detach) { "-d" })
        } else {
            docker-compose up $(if ($detach) { "-d" })
        }
    }
    "down" {
        Write-Host "Stopping services..." -ForegroundColor Green
        docker-compose down
    }
    "logs" {
        Write-Host "Showing logs..." -ForegroundColor Green
        docker-compose logs -f
    }
    "build" {
        Write-Host "Building images..." -ForegroundColor Green
        docker-compose build
    }
    "restart" {
        Write-Host "Restarting services..." -ForegroundColor Green
        docker-compose restart
    }
    "ps" {
        Write-Host "Service status:" -ForegroundColor Green
        docker-compose ps
    }
    "shell" {
        $service = Read-Host "Enter service name (backend/frontend)"
        Write-Host "Opening shell for $service..." -ForegroundColor Green
        docker-compose exec $service sh
    }
}

Write-Host "Done!" -ForegroundColor Green
