#!/usr/bin/env pwsh
# PhantomMesh VPN - 72-Hour Soak Test Runner
# Phase 4 Week 1 - Production Validation

param(
    [int]$DurationHours = 72,
    [int]$BaselineConcurrentUsers = 100,
    [int]$PeakConcurrentUsers = 1000,
    [int]$SampleIntervalMinutes = 5,
    [string]$TargetUrl = "http://localhost:24511",
    [string]$ResultsDir = "S:\PhantomMesh-VPN\staging\soak-results"
)

$ErrorActionPreference = "Stop"

# Create results directory
if (-not (Test-Path $ResultsDir)) {
    New-Item -ItemType Directory -Path $ResultsDir -Force | Out-Null
}

$testId = "soak_test_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
$startTime = Get-Date
$endTime = $startTime.AddHours($DurationHours)
$logFile = Join-Path $ResultsDir "$testId.log"
$metricsFile = Join-Path $ResultsDir "$testId_metrics.csv"
$summaryFile = Join-Path $ResultsDir "$testId_summary.json"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry
    Add-Content -Path $logFile -Value $logEntry
}

function Get-SystemMetrics {
    $cpuCounter = Get-Counter '\Processor(_Total)\% Processor Time' -ErrorAction SilentlyContinue
    $memCounter = Get-Counter '\Memory\Available MBytes' -ErrorAction SilentlyContinue
    
    return @{
        cpu_percent         = if ($cpuCounter) { [math]::Round($cpuCounter.CounterSamples[0].CookedValue, 2) } else { 0 }
        available_memory_mb = if ($memCounter) { [math]::Round($memCounter.CounterSamples[0].CookedValue, 2) } else { 0 }
        timestamp           = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    }
}

function Test-Endpoint {
    param([string]$Url, [int]$Concurrent)
    
    $results = @{
        requests  = 0
        successes = 0
        failures  = 0
        latencies = @()
    }
    
    $jobs = @()
    for ($i = 0; $i -lt $Concurrent; $i++) {
        $jobs += Start-Job -ScriptBlock {
            param($url)
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
            try {
                $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 30
                $stopwatch.Stop()
                return @{
                    success    = ($response.StatusCode -eq 200)
                    latency_ms = $stopwatch.ElapsedMilliseconds
                    error      = $null
                }
            }
            catch {
                $stopwatch.Stop()
                return @{
                    success    = $false
                    latency_ms = $stopwatch.ElapsedMilliseconds
                    error      = $_.Exception.Message
                }
            }
        } -ArgumentList "$Url/health"
    }
    
    $jobResults = $jobs | Wait-Job -Timeout 60 | Receive-Job
    $jobs | Remove-Job -Force
    
    foreach ($jr in $jobResults) {
        $results.requests++
        if ($jr.success) {
            $results.successes++
            $results.latencies += $jr.latency_ms
        }
        else {
            $results.failures++
        }
    }
    
    if ($results.latencies.Count -gt 0) {
        $sorted = $results.latencies | Sort-Object
        $results.p50 = $sorted[[math]::Floor($sorted.Count * 0.50)]
        $results.p99 = $sorted[[math]::Floor($sorted.Count * 0.99)]
        $results.avg = [math]::Round(($sorted | Measure-Object -Average).Average, 2)
    }
    else {
        $results.p50 = 0
        $results.p99 = 0
        $results.avg = 0
    }
    
    $results.success_rate = if ($results.requests -gt 0) { 
        [math]::Round(($results.successes / $results.requests) * 100, 2) 
    }
    else { 0 }
    
    return $results
}

function Get-LoadPattern {
    param([DateTime]$CurrentTime, [DateTime]$StartTime)
    
    $elapsed = ($CurrentTime - $StartTime).TotalHours
    $hourOfDay = $CurrentTime.Hour
    
    # Simulate realistic traffic patterns
    # Peak hours: 9AM-5PM (higher load)
    # Off-peak: 10PM-6AM (lower load)
    
    if ($hourOfDay -ge 9 -and $hourOfDay -lt 17) {
        return $PeakConcurrentUsers
    }
    elseif ($hourOfDay -ge 22 -or $hourOfDay -lt 6) {
        return [math]::Floor($BaselineConcurrentUsers * 0.3)
    }
    else {
        return $BaselineConcurrentUsers
    }
}

# Initialize CSV header
"timestamp,concurrent_users,requests,successes,failures,success_rate,p50_ms,p99_ms,avg_ms,cpu_percent,memory_mb,errors" | Out-File $metricsFile

Write-Host @"
═══════════════════════════════════════════════════════════════════════════════
  PHANTOMMESH VPN - 72-HOUR SOAK TEST
═══════════════════════════════════════════════════════════════════════════════

  Test ID:        $testId
  Start Time:     $($startTime.ToString("yyyy-MM-dd HH:mm:ss"))
  End Time:       $($endTime.ToString("yyyy-MM-dd HH:mm:ss"))
  Duration:       $DurationHours hours
  Target:         $TargetUrl
  Sample Rate:    Every $SampleIntervalMinutes minutes
  Baseline Load:  $BaselineConcurrentUsers concurrent users
  Peak Load:      $PeakConcurrentUsers concurrent users

  Logs:           $logFile
  Metrics:        $metricsFile

═══════════════════════════════════════════════════════════════════════════════
"@

Write-Log "Soak test starting: $testId"
Write-Log "Target: $TargetUrl, Duration: $DurationHours hours"

# Verify target is healthy
try {
    $healthCheck = Invoke-WebRequest -Uri "$TargetUrl/health" -UseBasicParsing -TimeoutSec 10
    if ($healthCheck.StatusCode -ne 200) {
        throw "Health check failed with status: $($healthCheck.StatusCode)"
    }
    Write-Log "Initial health check passed"
}
catch {
    Write-Log "FATAL: Target not healthy - $($_.Exception.Message)" -Level "ERROR"
    exit 1
}

$sampleCount = 0
$totalRequests = 0
$totalSuccesses = 0
$totalFailures = 0
$allLatencies = @()
$errors = @()

Write-Log "Beginning soak test loop..."

while ((Get-Date) -lt $endTime) {
    $sampleCount++
    $currentTime = Get-Date
    $elapsed = $currentTime - $startTime
    $remaining = $endTime - $currentTime
    
    # Get current load pattern
    $currentLoad = Get-LoadPattern -CurrentTime $currentTime -StartTime $startTime
    
    Write-Log "Sample #$sampleCount - Load: $currentLoad users - Elapsed: $([math]::Round($elapsed.TotalHours, 2))h - Remaining: $([math]::Round($remaining.TotalHours, 2))h"
    
    # Run test batch
    $testResults = Test-Endpoint -Url $TargetUrl -Concurrent $currentLoad
    $sysMetrics = Get-SystemMetrics
    
    # Accumulate stats
    $totalRequests += $testResults.requests
    $totalSuccesses += $testResults.successes
    $totalFailures += $testResults.failures
    $allLatencies += $testResults.latencies
    
    # Log metrics to CSV
    $csvLine = "$($sysMetrics.timestamp),$currentLoad,$($testResults.requests),$($testResults.successes),$($testResults.failures),$($testResults.success_rate),$($testResults.p50),$($testResults.p99),$($testResults.avg),$($sysMetrics.cpu_percent),$($sysMetrics.available_memory_mb),"
    Add-Content -Path $metricsFile -Value $csvLine
    
    # Check thresholds
    $thresholdViolations = @()
    if ($testResults.p99 -gt 200) { $thresholdViolations += "P99 > 200ms ($($testResults.p99)ms)" }
    if ($testResults.success_rate -lt 99) { $thresholdViolations += "Success < 99% ($($testResults.success_rate)%)" }
    
    if ($thresholdViolations.Count -gt 0) {
        $violationMsg = "THRESHOLD VIOLATION: " + ($thresholdViolations -join ", ")
        Write-Log $violationMsg -Level "WARN"
        $errors += @{
            timestamp  = $sysMetrics.timestamp
            sample     = $sampleCount
            violations = $thresholdViolations
        }
    }
    
    # Progress display
    $progressPct = [math]::Round(($elapsed.TotalHours / $DurationHours) * 100, 1)
    Write-Host "  └─ Success: $($testResults.success_rate)% | P50: $($testResults.p50)ms | P99: $($testResults.p99)ms | Progress: $progressPct%"
    
    # Sleep until next sample
    Start-Sleep -Seconds ($SampleIntervalMinutes * 60)
}

# Calculate final statistics
$finalSuccessRate = if ($totalRequests -gt 0) { [math]::Round(($totalSuccesses / $totalRequests) * 100, 2) } else { 0 }
$sortedLatencies = $allLatencies | Sort-Object
$finalP50 = if ($sortedLatencies.Count -gt 0) { $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.50)] } else { 0 }
$finalP99 = if ($sortedLatencies.Count -gt 0) { $sortedLatencies[[math]::Floor($sortedLatencies.Count * 0.99)] } else { 0 }
$finalAvg = if ($sortedLatencies.Count -gt 0) { [math]::Round(($sortedLatencies | Measure-Object -Average).Average, 2) } else { 0 }

$actualDuration = (Get-Date) - $startTime

# Final validation
$validationPassed = ($finalP99 -le 200) -and ($finalSuccessRate -ge 99)

$summary = @{
    test_id                = $testId
    start_time             = $startTime.ToString("o")
    end_time               = (Get-Date).ToString("o")
    planned_duration_hours = $DurationHours
    actual_duration_hours  = [math]::Round($actualDuration.TotalHours, 2)
    total_samples          = $sampleCount
    total_requests         = $totalRequests
    total_successes        = $totalSuccesses
    total_failures         = $totalFailures
    final_success_rate     = $finalSuccessRate
    final_p50_ms           = $finalP50
    final_p99_ms           = $finalP99
    final_avg_ms           = $finalAvg
    threshold_violations   = $errors.Count
    validation_passed      = $validationPassed
    errors                 = $errors
}

$summary | ConvertTo-Json -Depth 5 | Out-File $summaryFile

Write-Host @"

═══════════════════════════════════════════════════════════════════════════════
  SOAK TEST COMPLETE
═══════════════════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════════════════╗
║                        72-HOUR SOAK TEST RESULTS                              ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  DURATION                                                                     ║
║    Planned:              $DurationHours hours                                        ║
║    Actual:               $([math]::Round($actualDuration.TotalHours, 2)) hours                                        ║
║    Samples:              $sampleCount                                               ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  THROUGHPUT                                                                   ║
║    Total Requests:       $totalRequests                                          ║
║    Successful:           $totalSuccesses                                          ║
║    Failed:               $totalFailures                                               ║
║    Success Rate:         $finalSuccessRate%                                          ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  LATENCY                                                                      ║
║    P50:                  $finalP50 ms                                             ║
║    P99:                  $finalP99 ms                                             ║
║    Average:              $finalAvg ms                                             ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║  STABILITY                                                                    ║
║    Threshold Violations: $($errors.Count)                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝

"@

if ($validationPassed) {
    Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  ✅ SOAK TEST PASSED - SYSTEM STABLE FOR PRODUCTION                          ║" -ForegroundColor Green
    Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Green
}
else {
    Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Red
    Write-Host "║  ❌ SOAK TEST FAILED - REVIEW THRESHOLD VIOLATIONS                           ║" -ForegroundColor Red
    Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Red
}

Write-Host ""
Write-Host "Results saved to:"
Write-Host "  Logs:    $logFile"
Write-Host "  Metrics: $metricsFile"
Write-Host "  Summary: $summaryFile"

Write-Log "Soak test completed. Validation: $(if ($validationPassed) { 'PASSED' } else { 'FAILED' })"

return $summary
