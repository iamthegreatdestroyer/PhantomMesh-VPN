#!/usr/bin/env pwsh
# Soak Test Monitor - Runs for 72 hours with continuous monitoring

param(
    [string]$ConfigPath = "S:\PhantomMesh-VPN\staging\soak-test-config.json"
)

$config = Get-Content $ConfigPath | ConvertFrom-Json
$startTime = Get-Date
$endTime = $startTime.AddHours($config.duration_hours)

Write-Host "Starting 72-hour soak test..."
Write-Host "End time: $endTime"

$metrics = @{
    samples = @()
    memory_readings = @()
    latency_readings = @()
    error_counts = @()
}

while ((Get-Date) -lt $endTime) {
    $sampleTime = Get-Date
    
    # Collect sample
    try {
        $response = Invoke-WebRequest -Uri "$($config.target_url)/health" -TimeoutSec 10
        $latency = $response.BaseResponse.ResponseTime.TotalMilliseconds
        
        $sample = @{
            timestamp = $sampleTime.ToString("o")
            latency_ms = $latency
            status = "healthy"
            error = $null
        }
    }
    catch {
        $sample = @{
            timestamp = $sampleTime.ToString("o")
            latency_ms = 0
            status = "error"
            error = $_.Exception.Message
        }
    }
    
    $metrics.samples += $sample
    
    # Log progress every hour
    $elapsed = (Get-Date) - $startTime
    if ($elapsed.TotalMinutes % 60 -lt 1) {
        Write-Host "[$($sampleTime.ToString('HH:mm:ss'))] Soak test progress: $([Math]::Round($elapsed.TotalHours, 1)) hours elapsed"
    }
    
    Start-Sleep $config.monitoring.sample_interval_seconds
}

Write-Host "Soak test completed!"
$metrics | ConvertTo-Json -Depth 10 | Set-Content "S:\PhantomMesh-VPN\staging\soak-test-results.json"
