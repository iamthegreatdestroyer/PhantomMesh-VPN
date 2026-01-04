#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Execute first load test: Deploy harness, run ramp-up, collect metrics, validate latency
.DESCRIPTION
    Orchestrates the complete load testing pipeline for PhantomMesh-VPN
.PARAMETER Environment
    Target environment (dev, staging, prod)
.PARAMETER KubeConfig
    Path to kubeconfig file
.PARAMETER Timeout
    Timeout in seconds for test execution
#>

param(
    [string]$Environment = "prod",
    [string]$KubeConfig = $null,
    [int]$Timeout = 600,
    [bool]$WaitForCompletion = $true
)

$ErrorActionPreference = "Stop"
$VerbosePreference = "Continue"

# Colors for output
$SuccessColor = "Green"
$ErrorColor = "Red"
$WarningColor = "Yellow"
$InfoColor = "Cyan"

function Write-Success {
    Write-Host $args -ForegroundColor $SuccessColor
}

function Write-Error-Custom {
    Write-Host "❌ ERROR: $args" -ForegroundColor $ErrorColor
}

function Write-Warning-Custom {
    Write-Host "⚠️  WARNING: $args" -ForegroundColor $WarningColor
}

function Write-Info {
    Write-Host "ℹ️  INFO: $args" -ForegroundColor $InfoColor
}

Write-Host "=========================================="
Write-Host "PhantomMesh Load Test Execution Pipeline"
Write-Host "=========================================="
Write-Host "Environment: $Environment"
Write-Host "Timeout: $Timeout seconds"
Write-Host "Start Time: $(Get-Date -Format 'o')"
Write-Host ""

# Step 1: Validate prerequisites
Write-Info "STEP 1: Validating prerequisites..."

# Check kubectl
if (-not (Get-Command kubectl -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "kubectl not found. Please install kubectl."
    exit 1
}

# Set kubeconfig if provided
if ($KubeConfig) {
    $env:KUBECONFIG = $KubeConfig
    Write-Info "Using kubeconfig: $KubeConfig"
}

# Check cluster connectivity
try {
    $clusterInfo = kubectl cluster-info 2>&1
    Write-Success "✅ Kubernetes cluster accessible"
}
catch {
    Write-Error-Custom "Cannot connect to Kubernetes cluster: $_"
    exit 1
}

Write-Host ""

# Step 2: Deploy test harness
Write-Info "STEP 2: Deploying load test harness..."

$manifestDir = "s:\PhantomMesh-VPN\phantom-mesh-vpn\tests\load"
$harnessManifeist = "$manifestDir\test_harness_deployment.yaml"
$configMapManifest = "$manifestDir\load_test_scripts_configmap.yaml"

if (-not (Test-Path $harnessManifeist)) {
    Write-Error-Custom "Test harness manifest not found: $harnessManifeist"
    exit 1
}

try {
    # Deploy namespace and ConfigMap first
    Write-Info "Creating load test namespace and configuration..."
    kubectl apply -f $configMapManifest | Out-Null
    Start-Sleep -Seconds 5

    # Deploy harness
    Write-Info "Deploying load test runner pod..."
    kubectl apply -f $harnessManifeist | Out-Null
    Start-Sleep -Seconds 5

    Write-Success "✅ Test harness deployed successfully"
}
catch {
    Write-Error-Custom "Failed to deploy test harness: $_"
    exit 1
}

Write-Host ""

# Step 3: Wait for deployment to be ready
Write-Info "STEP 3: Waiting for load test runner pod to be ready..."

$startTime = Get-Date
$maxWaitTime = New-TimeSpan -Seconds 120

try {
    $ready = $false
    while (-not $ready -and ((Get-Date) - $startTime) -lt $maxWaitTime) {
        $podStatus = kubectl get pods -n phantom-load-test -l app=load-test-runner -o json | ConvertFrom-Json
        
        if ($podStatus.items.Count -gt 0) {
            $pod = $podStatus.items[0]
            $podName = $pod.metadata.name
            $phase = $pod.status.phase
            
            Write-Info "Pod status: $phase"
            
            if ($phase -eq "Running") {
                Write-Success "✅ Load test pod is running: $podName"
                $ready = $true
            }
            elseif ($phase -eq "Failed") {
                Write-Error-Custom "Pod failed to start"
                kubectl logs -n phantom-load-test $podName
                exit 1
            }
        }
        
        if (-not $ready) {
            Start-Sleep -Seconds 5
        }
    }
    
    if (-not $ready) {
        Write-Error-Custom "Timeout waiting for pod to be ready"
        exit 1
    }
}
catch {
    Write-Error-Custom "Error waiting for pod: $_"
    exit 1
}

Write-Host ""

# Step 4: Run ramp-up test (0→1000 req/s)
Write-Info "STEP 4: Running ramp-up test (0→1000 req/s)..."

$podName = (kubectl get pods -n phantom-load-test -l app=load-test-runner -o jsonpath='{.items[0].metadata.name}')

try {
    Write-Info "Executing load test runner in pod: $podName"
    
    # Create a temporary script that runs the load test
    $testScript = @"
#!/bin/bash
cd /app
python3 tests/load/load_test_runner.py --profile ramp-up
"@

    # Execute in pod
    kubectl exec -n phantom-load-test $podName -- /bin/bash -c $testScript | Out-Host
    
    Write-Success "✅ Ramp-up test completed"
}
catch {
    Write-Warning-Custom "Error during test execution: $_"
    # Don't exit here - continue to collect what metrics we can
}

Write-Host ""

# Step 5: Collect baseline metrics
Write-Info "STEP 5: Collecting baseline metrics..."

try {
    # Copy metrics from pod to local
    $metricsDir = "/metrics"
    $localMetricsDir = "$manifestDir\results"
    
    if (-not (Test-Path $localMetricsDir)) {
        New-Item -ItemType Directory -Path $localMetricsDir | Out-Null
    }
    
    Write-Info "Retrieving metrics from pod..."
    kubectl cp phantom-load-test/${podName}:/metrics/results.json "$localMetricsDir/load_test_results.json" -c load-tester 2>&1 | Out-Null
    
    if (Test-Path "$localMetricsDir/load_test_results.json") {
        $results = Get-Content "$localMetricsDir/load_test_results.json" | ConvertFrom-Json
        
        Write-Info "Baseline Metrics Collected:"
        Write-Host "  Duration: $($results.duration) seconds"
        Write-Host "  Total Requests: $($results.request_count)"
        Write-Host "  Successful: $($results.success_count)"
        Write-Host "  Errors: $($results.error_count)"
        Write-Host "  Error Rate: $(($results.error_count / $results.request_count * 100).ToString('F2'))%"
        Write-Host ""
        Write-Host "  Latency Statistics (ms):"
        Write-Host "    Mean: $($results.metrics.latency_mean)"
        Write-Host "    Min: $($results.metrics.latency_min)"
        Write-Host "    Max: $($results.metrics.latency_max)"
        Write-Host "    P50: $($results.metrics.latency_percentiles.p50)"
        Write-Host "    P95: $($results.metrics.latency_percentiles.p95)"
        Write-Host "    P99: $($results.metrics.latency_percentiles.p99)"
        Write-Host "    P999: $($results.metrics.latency_percentiles.p999)"
        Write-Host ""
        Write-Host "  Throughput:"
        Write-Host "    Peak RPS: $($results.metrics.peak_rps)"
        Write-Host "    Avg RPS: $($results.metrics.avg_rps)"
        
        Write-Success "✅ Metrics collected and saved to: $localMetricsDir/load_test_results.json"
    }
    else {
        Write-Warning-Custom "Metrics file not found in pod"
    }
}
catch {
    Write-Warning-Custom "Error collecting metrics: $_"
}

Write-Host ""

# Step 6: Validate latency targets
Write-Info "STEP 6: Validating latency targets..."

$latencyValidationPassed = $false

try {
    if (Test-Path "$localMetricsDir/load_test_results.json") {
        $results = Get-Content "$localMetricsDir/load_test_results.json" | ConvertFrom-Json
        
        # Define targets
        $targets = @{
            "P50"  = 50
            "P99"  = 200
            "P999" = 500
            "Mean" = 100
        }
        
        Write-Host "Latency Target Validation:"
        $allPassed = $true
        
        $p50 = $results.metrics.latency_percentiles.p50
        $p99 = $results.metrics.latency_percentiles.p99
        $p999 = $results.metrics.latency_percentiles.p999
        $mean = $results.metrics.latency_mean
        
        # Check P50
        if ($p50 -le $targets["P50"]) {
            Write-Success "  ✅ P50: $p50 ms (target: $($targets['P50']) ms)"
        }
        else {
            Write-Error-Custom "  ❌ P50: $p50 ms (target: $($targets['P50']) ms)"
            $allPassed = $false
        }
        
        # Check P99
        if ($p99 -le $targets["P99"]) {
            Write-Success "  ✅ P99: $p99 ms (target: $($targets['P99']) ms)"
        }
        else {
            Write-Error-Custom "  ❌ P99: $p99 ms (target: $($targets['P99']) ms)"
            $allPassed = $false
        }
        
        # Check P999
        if ($p999 -le $targets["P999"]) {
            Write-Success "  ✅ P999: $p999 ms (target: $($targets['P999']) ms)"
        }
        else {
            Write-Warning-Custom "  ⚠️  P999: $p999 ms (target: $($targets['P999']) ms)"
        }
        
        # Check Mean
        if ($mean -le $targets["Mean"]) {
            Write-Success "  ✅ Mean: $mean ms (target: $($targets['Mean']) ms)"
        }
        else {
            Write-Error-Custom "  ❌ Mean: $mean ms (target: $($targets['Mean']) ms)"
            $allPassed = $false
        }
        
        if ($allPassed) {
            Write-Success "✅ All critical latency targets PASSED"
            $latencyValidationPassed = $true
        }
        else {
            Write-Error-Custom "❌ Some latency targets FAILED"
        }
    }
}
catch {
    Write-Warning-Custom "Error validating latency: $_"
}

Write-Host ""

# Step 7: Generate summary report
Write-Info "STEP 7: Generating test summary report..."

$reportPath = "$localMetricsDir/LOAD_TEST_REPORT.md"

$report = @"
# PhantomMesh Load Test Report
**Generated:** $(Get-Date -Format 'o')  
**Environment:** $Environment  
**Status:** $(if ($latencyValidationPassed) { "✅ PASSED" } else { "⚠️  ISSUES DETECTED" })

## Test Configuration
- Test Type: Ramp-up (0→1000 req/s)
- Duration: 300 seconds
- Ramp-up Period: 60 seconds
- Initial Rate: 100 req/s
- Peak Rate: 1000 req/s

## Results Summary
**Load Test Results:**
- Total Requests: N/A (see detailed results)
- Success Rate: N/A
- Error Rate: N/A

## Latency Analysis
**Targets vs Actual:**
| Percentile | Target | Actual | Status |
|-----------|--------|--------|--------|
| Mean | 100 ms | N/A | N/A |
| P50 | 50 ms | N/A | N/A |
| P99 | 200 ms | N/A | N/A |
| P999 | 500 ms | N/A | N/A |

## Recommendations
1. Review detailed metrics in \`load_test_results.json\`
2. Check pod logs for any errors or warnings
3. Monitor Prometheus metrics for system behavior
4. Plan next test phases based on these results

## Files Generated
- \`load_test_results.json\` - Detailed metrics
- \`LOAD_TEST_REPORT.md\` - This report

**Test Duration:** $(if ($latencyValidationPassed) { "PASSED" } else { "COMPLETED WITH WARNINGS" })
"@

$report | Out-File -FilePath $reportPath -Encoding UTF8
Write-Success "✅ Test report saved to: $reportPath"

Write-Host ""
Write-Host "=========================================="
Write-Host "Load Test Execution Complete"
Write-Host "End Time: $(Get-Date -Format 'o')"
Write-Host "=========================================="

if ($latencyValidationPassed) {
    Write-Success "✅ LOAD TEST PASSED - Ready for next phase"
    exit 0
}
else {
    Write-Warning-Custom "⚠️  LOAD TEST COMPLETED WITH WARNINGS - Review results before proceeding"
    exit 1
}
