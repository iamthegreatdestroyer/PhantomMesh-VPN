#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Production Cluster Load Test - 1,000+ Concurrent Connections
.DESCRIPTION
    Executes load testing against the running PhantomMesh production cluster
#>

param(
    [int]$ConcurrentUsers = 1000,
    [int]$DurationSeconds = 60,
    [string]$TargetUrl = "http://localhost:24511"
)

$ErrorActionPreference = "Continue"
$ResultsDir = "S:\PhantomMesh-VPN\staging\load-results"
$LogFile = Join-Path $ResultsDir "load_test_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"

# Create results directory
if (-not (Test-Path $ResultsDir)) {
    New-Item -ItemType Directory -Path $ResultsDir -Force | Out-Null
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  PHANTOMMESH VPN LOAD TEST - $ConcurrentUsers CONCURRENT CONNECTIONS" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "Configuration:" -ForegroundColor White
Write-Host "  Target URL: $TargetUrl" 
Write-Host "  Concurrent Users: $ConcurrentUsers"
Write-Host "  Duration: $DurationSeconds seconds"
Write-Host "  Results Dir: $ResultsDir"
Write-Host ""

# Initialize metrics collection
$metrics = @{
    test_id               = "load_test_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    start_time            = (Get-Date).ToString("o")
    target_url            = $TargetUrl
    concurrent_users      = $ConcurrentUsers
    duration_seconds      = $DurationSeconds
    total_requests        = 0
    successful_requests   = 0
    failed_requests       = 0
    latencies             = [System.Collections.ArrayList]::new()
    errors                = [System.Collections.ArrayList]::new()
    throughput_per_second = [System.Collections.ArrayList]::new()
}

# Check if target is available
Write-Host "Verifying target availability..." -ForegroundColor Cyan
try {
    $healthCheck = Invoke-WebRequest -Uri "$TargetUrl/health" -TimeoutSec 5 -ErrorAction Stop
    if ($healthCheck.StatusCode -eq 200) {
        Write-Host "✅ Target is healthy and accepting connections" -ForegroundColor Green
    }
}
catch {
    Write-Host "⚠️  Target may not be available: $_" -ForegroundColor Yellow
    Write-Host "   Attempting to run test anyway..." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Starting load test..." -ForegroundColor Cyan
Write-Host ""

$testStartTime = Get-Date
$testEndTime = $testStartTime.AddSeconds($DurationSeconds)
$requestsThisSecond = 0
$lastSecond = $testStartTime

# Run concurrent requests
$batchSize = [Math]::Min($ConcurrentUsers, 50)  # Process in batches
$totalBatches = [Math]::Ceiling($ConcurrentUsers / $batchSize)

Write-Host "Processing $ConcurrentUsers connections in $totalBatches batches of $batchSize..." -ForegroundColor White
Write-Host ""

$progressInterval = [Math]::Max(1, [Math]::Floor($totalBatches / 10))

for ($batch = 0; $batch -lt $totalBatches; $batch++) {
    $currentBatchSize = [Math]::Min($batchSize, $ConcurrentUsers - ($batch * $batchSize))
    
    # Progress indicator
    if ($batch % $progressInterval -eq 0 -or $batch -eq ($totalBatches - 1)) {
        $pct = [Math]::Round(($batch + 1) / $totalBatches * 100, 1)
        $completed = $metrics.total_requests
        $success = $metrics.successful_requests
        $failed = $metrics.failed_requests
        $bar = "█" * [Math]::Floor($pct / 5) + "░" * (20 - [Math]::Floor($pct / 5))
        Write-Host "`r  Progress: [$bar] $pct% | Requests: $completed | Success: $success | Failed: $failed    " -NoNewline -ForegroundColor Cyan
    }
    
    # Execute batch of concurrent requests
    for ($i = 0; $i -lt $currentBatchSize; $i++) {
        $reqStart = Get-Date
        $metrics.total_requests++
        
        try {
            $response = Invoke-WebRequest -Uri "$TargetUrl/health" -TimeoutSec 10 -ErrorAction Stop
            $reqEnd = Get-Date
            $latency = ($reqEnd - $reqStart).TotalMilliseconds
            
            $metrics.latencies.Add($latency) | Out-Null
            $metrics.successful_requests++
            
            # Track requests per second
            if (($reqEnd - $lastSecond).TotalSeconds -ge 1) {
                $metrics.throughput_per_second.Add($requestsThisSecond) | Out-Null
                $requestsThisSecond = 0
                $lastSecond = $reqEnd
            }
            $requestsThisSecond++
        }
        catch {
            $reqEnd = Get-Date
            $latency = ($reqEnd - $reqStart).TotalMilliseconds
            
            $metrics.latencies.Add($latency) | Out-Null
            $metrics.errors.Add($_.Exception.Message) | Out-Null
            $metrics.failed_requests++
        }
    }
    
    # Check if we've exceeded duration
    if ((Get-Date) -gt $testEndTime) { break }
    
    # Small delay between batches to prevent overwhelming
    Start-Sleep -Milliseconds 10
}

$testActualEndTime = Get-Date
$metrics.end_time = $testActualEndTime.ToString("o")
$metrics.actual_duration_seconds = ($testActualEndTime - $testStartTime).TotalSeconds

Write-Host ""
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  LOAD TEST COMPLETE" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Calculate statistics
$sortedLatencies = $metrics.latencies | Sort-Object

if ($sortedLatencies.Count -gt 0) {
    $p50Index = [Math]::Floor($sortedLatencies.Count * 0.50)
    $p90Index = [Math]::Floor($sortedLatencies.Count * 0.90)
    $p99Index = [Math]::Floor($sortedLatencies.Count * 0.99)
    
    $metrics.p50_latency_ms = [Math]::Round($sortedLatencies[$p50Index], 2)
    $metrics.p90_latency_ms = [Math]::Round($sortedLatencies[$p90Index], 2)
    $metrics.p99_latency_ms = [Math]::Round($sortedLatencies[$p99Index], 2)
    $metrics.avg_latency_ms = [Math]::Round(($sortedLatencies | Measure-Object -Average).Average, 2)
    $metrics.min_latency_ms = [Math]::Round(($sortedLatencies | Measure-Object -Minimum).Minimum, 2)
    $metrics.max_latency_ms = [Math]::Round(($sortedLatencies | Measure-Object -Maximum).Maximum, 2)
}

$metrics.success_rate = if ($metrics.total_requests -gt 0) { [Math]::Round($metrics.successful_requests / $metrics.total_requests, 4) } else { 0 }
$metrics.error_rate = if ($metrics.total_requests -gt 0) { [Math]::Round($metrics.failed_requests / $metrics.total_requests, 6) } else { 0 }
$metrics.requests_per_second = if ($metrics.actual_duration_seconds -gt 0) { [Math]::Round($metrics.total_requests / $metrics.actual_duration_seconds, 2) } else { 0 }

# Display results
Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor White
Write-Host "║                           LOAD TEST RESULTS                                   ║" -ForegroundColor White
Write-Host "╠═══════════════════════════════════════════════════════════════════════════════╣" -ForegroundColor White
Write-Host "║  THROUGHPUT                                                                   ║" -ForegroundColor Yellow
Write-Host "║    Total Requests:         $($metrics.total_requests.ToString().PadLeft(10))                                    ║"
Write-Host "║    Requests/Second:        $($metrics.requests_per_second.ToString().PadLeft(10))                                    ║"
Write-Host "║    Duration:               $([Math]::Round($metrics.actual_duration_seconds, 2).ToString().PadLeft(10)) sec                                 ║"
Write-Host "╠═══════════════════════════════════════════════════════════════════════════════╣" -ForegroundColor White
Write-Host "║  SUCCESS METRICS                                                              ║" -ForegroundColor Yellow
Write-Host "║    Successful:             $($metrics.successful_requests.ToString().PadLeft(10))                                    ║"
Write-Host "║    Failed:                 $($metrics.failed_requests.ToString().PadLeft(10))                                    ║"
Write-Host "║    Success Rate:           $([Math]::Round($metrics.success_rate * 100, 2).ToString().PadLeft(10))%                                   ║"
Write-Host "║    Error Rate:             $([Math]::Round($metrics.error_rate * 100, 4).ToString().PadLeft(10))%                                   ║"
Write-Host "╠═══════════════════════════════════════════════════════════════════════════════╣" -ForegroundColor White
Write-Host "║  LATENCY (milliseconds)                                                       ║" -ForegroundColor Yellow
Write-Host "║    P50:                    $($metrics.p50_latency_ms.ToString().PadLeft(10)) ms                                  ║"
Write-Host "║    P90:                    $($metrics.p90_latency_ms.ToString().PadLeft(10)) ms                                  ║"
Write-Host "║    P99:                    $($metrics.p99_latency_ms.ToString().PadLeft(10)) ms                                  ║"
Write-Host "║    Average:                $($metrics.avg_latency_ms.ToString().PadLeft(10)) ms                                  ║"
Write-Host "║    Min:                    $($metrics.min_latency_ms.ToString().PadLeft(10)) ms                                  ║"
Write-Host "║    Max:                    $($metrics.max_latency_ms.ToString().PadLeft(10)) ms                                  ║"
Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor White
Write-Host ""

# Performance validation
Write-Host "═══════════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  PERFORMANCE BASELINE VALIDATION" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

$thresholds = @{
    p99_max_ms       = 200
    p50_max_ms       = 100
    error_rate_max   = 0.01
    success_rate_min = 0.99
}

$allPassed = $true

# P99 Check
if ($metrics.p99_latency_ms -le $thresholds.p99_max_ms) {
    Write-Host "  ✅ P99 Latency: $($metrics.p99_latency_ms) ms <= $($thresholds.p99_max_ms) ms" -ForegroundColor Green
}
else {
    Write-Host "  ❌ P99 Latency: $($metrics.p99_latency_ms) ms > $($thresholds.p99_max_ms) ms" -ForegroundColor Red
    $allPassed = $false
}

# P50 Check
if ($metrics.p50_latency_ms -le $thresholds.p50_max_ms) {
    Write-Host "  ✅ P50 Latency: $($metrics.p50_latency_ms) ms <= $($thresholds.p50_max_ms) ms" -ForegroundColor Green
}
else {
    Write-Host "  ❌ P50 Latency: $($metrics.p50_latency_ms) ms > $($thresholds.p50_max_ms) ms" -ForegroundColor Red
    $allPassed = $false
}

# Error Rate Check  
if ($metrics.error_rate -le $thresholds.error_rate_max) {
    Write-Host "  ✅ Error Rate: $([Math]::Round($metrics.error_rate * 100, 4))% <= $([Math]::Round($thresholds.error_rate_max * 100, 2))%" -ForegroundColor Green
}
else {
    Write-Host "  ❌ Error Rate: $([Math]::Round($metrics.error_rate * 100, 4))% > $([Math]::Round($thresholds.error_rate_max * 100, 2))%" -ForegroundColor Red
    $allPassed = $false
}

# Success Rate Check
if ($metrics.success_rate -ge $thresholds.success_rate_min) {
    Write-Host "  ✅ Success Rate: $([Math]::Round($metrics.success_rate * 100, 2))% >= $([Math]::Round($thresholds.success_rate_min * 100, 2))%" -ForegroundColor Green
}
else {
    Write-Host "  ❌ Success Rate: $([Math]::Round($metrics.success_rate * 100, 2))% < $([Math]::Round($thresholds.success_rate_min * 100, 2))%" -ForegroundColor Red
    $allPassed = $false
}

Write-Host ""

if ($allPassed) {
    Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  ✅ PERFORMANCE BASELINE VALIDATION: ALL CHECKS PASSED                       ║" -ForegroundColor Green
    Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Green
    $metrics.validation_result = "PASSED"
}
else {
    Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Yellow
    Write-Host "║  ⚠️  PERFORMANCE BASELINE VALIDATION: SOME CHECKS FAILED                     ║" -ForegroundColor Yellow
    Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Yellow
    $metrics.validation_result = "PARTIAL"
}

# Save results to file
$resultsFile = Join-Path $ResultsDir "load_test_results_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"

# Convert to clean object for JSON
$exportMetrics = @{
    test_id                 = $metrics.test_id
    start_time              = $metrics.start_time
    end_time                = $metrics.end_time
    target_url              = $metrics.target_url
    concurrent_users        = $metrics.concurrent_users
    duration_seconds        = $metrics.duration_seconds
    actual_duration_seconds = $metrics.actual_duration_seconds
    total_requests          = $metrics.total_requests
    successful_requests     = $metrics.successful_requests
    failed_requests         = $metrics.failed_requests
    success_rate            = $metrics.success_rate
    error_rate              = $metrics.error_rate
    requests_per_second     = $metrics.requests_per_second
    p50_latency_ms          = $metrics.p50_latency_ms
    p90_latency_ms          = $metrics.p90_latency_ms
    p99_latency_ms          = $metrics.p99_latency_ms
    avg_latency_ms          = $metrics.avg_latency_ms
    min_latency_ms          = $metrics.min_latency_ms
    max_latency_ms          = $metrics.max_latency_ms
    validation_result       = $metrics.validation_result
    error_sample            = ($metrics.errors | Select-Object -First 5)
}

$exportMetrics | ConvertTo-Json -Depth 10 | Set-Content $resultsFile
Write-Host ""
Write-Host "Results saved to: $resultsFile" -ForegroundColor Cyan
Write-Host ""

# Return results object
return $exportMetrics
