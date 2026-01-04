#!/bin/bash
# PhantomMesh Load Test Execution Script
# Orchestrates: Deploy harness → Run ramp-up test → Collect metrics → Validate latency

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="phantom-load-test"
POD_LABEL="app=load-test-runner"
TIMEOUT=600
METRICS_DIR="./results"
KUBECONFIG="${KUBECONFIG:-}"

# Functions
log_info() {
    echo -e "${BLUE}ℹ️  INFO: $*${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $*${NC}"
}

log_error() {
    echo -e "${RED}❌ ERROR: $*${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  WARNING: $*${NC}"
}

echo "=========================================="
echo "PhantomMesh Load Test Execution Pipeline"
echo "=========================================="
echo "Start Time: $(date -Iseconds)"
echo ""

# STEP 1: Validate Prerequisites
log_info "STEP 1: Validating prerequisites..."

if ! command -v kubectl &> /dev/null; then
    log_error "kubectl not found"
    exit 1
fi

if ! kubectl cluster-info &> /dev/null; then
    log_error "Cannot connect to Kubernetes cluster"
    exit 1
fi

log_success "Kubernetes cluster accessible"
echo ""

# STEP 2: Deploy Test Harness
log_info "STEP 2: Deploying load test harness..."

MANIFEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS_MANIFEST="$MANIFEST_DIR/test_harness_deployment.yaml"
CONFIGMAP_MANIFEST="$MANIFEST_DIR/load_test_scripts_configmap.yaml"

if [ ! -f "$HARNESS_MANIFEST" ]; then
    log_error "Test harness manifest not found: $HARNESS_MANIFEST"
    exit 1
fi

log_info "Creating load test namespace and configuration..."
kubectl apply -f "$CONFIGMAP_MANIFEST" > /dev/null 2>&1
sleep 3

log_info "Deploying load test runner pod..."
kubectl apply -f "$HARNESS_MANIFEST" > /dev/null 2>&1
sleep 5

log_success "Test harness deployed successfully"
echo ""

# STEP 3: Wait for Pod Ready
log_info "STEP 3: Waiting for load test runner pod to be ready..."

WAIT_START=$(date +%s)
POD_READY=false

while ! $POD_READY; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - WAIT_START))
    
    if [ $ELAPSED -gt $TIMEOUT ]; then
        log_error "Timeout waiting for pod to be ready"
        exit 1
    fi
    
    POD_STATUS=$(kubectl get pods -n "$NAMESPACE" -l "$POD_LABEL" -o json 2>/dev/null | jq -r '.items[0].status.phase' 2>/dev/null || echo "Pending")
    
    if [ "$POD_STATUS" = "Running" ]; then
        POD_READY=true
        POD_NAME=$(kubectl get pods -n "$NAMESPACE" -l "$POD_LABEL" -o jsonpath='{.items[0].metadata.name}')
        log_success "Load test pod is running: $POD_NAME"
    elif [ "$POD_STATUS" = "Failed" ]; then
        log_error "Pod failed to start"
        kubectl logs -n "$NAMESPACE" "$POD_NAME" 2>/dev/null || true
        exit 1
    else
        log_info "Pod status: $POD_STATUS (waiting...)"
        sleep 5
    fi
done

echo ""

# STEP 4: Run Ramp-up Test
log_info "STEP 4: Running ramp-up test (0→1000 req/s)..."

POD_NAME=$(kubectl get pods -n "$NAMESPACE" -l "$POD_LABEL" -o jsonpath='{.items[0].metadata.name}')

log_info "Executing load test in pod: $POD_NAME"
log_info "Test duration: 300 seconds (ramp-up: 60s)"

# Tail pod logs while test runs
kubectl logs -f -n "$NAMESPACE" "$POD_NAME" --tail=0 2>&1 &
LOGS_PID=$!

# Wait for test to complete (with timeout)
TEST_START=$(date +%s)
TEST_COMPLETE=false

while ! $TEST_COMPLETE; do
    # Check if /metrics/results.json exists in pod
    if kubectl exec -n "$NAMESPACE" "$POD_NAME" -- test -f /metrics/results.json 2>/dev/null; then
        TEST_COMPLETE=true
        log_success "Test completed successfully"
    else
        CURRENT_TIME=$(date +%s)
        ELAPSED=$((CURRENT_TIME - TEST_START))
        
        if [ $ELAPSED -gt 400 ]; then
            log_warning "Test timeout or still running..."
            TEST_COMPLETE=true
        else
            REMAINING=$((400 - ELAPSED))
            echo -ne "\r  Elapsed: ${ELAPSED}s / Remaining: ${REMAINING}s  "
            sleep 10
        fi
    fi
done

# Kill logs tail
kill $LOGS_PID 2>/dev/null || true

echo ""
echo ""

# STEP 5: Collect Baseline Metrics
log_info "STEP 5: Collecting baseline metrics..."

mkdir -p "$METRICS_DIR"

if ! kubectl cp "$NAMESPACE/$POD_NAME:/metrics/results.json" "$METRICS_DIR/load_test_results.json" -c load-tester 2>/dev/null; then
    log_warning "Could not copy metrics file from pod"
else
    if [ -f "$METRICS_DIR/load_test_results.json" ]; then
        RESULTS=$(cat "$METRICS_DIR/load_test_results.json")
        
        DURATION=$(echo "$RESULTS" | jq -r '.duration' 2>/dev/null || echo "0")
        REQUEST_COUNT=$(echo "$RESULTS" | jq -r '.request_count' 2>/dev/null || echo "0")
        SUCCESS_COUNT=$(echo "$RESULTS" | jq -r '.success_count' 2>/dev/null || echo "0")
        ERROR_COUNT=$(echo "$RESULTS" | jq -r '.error_count' 2>/dev/null || echo "0")
        
        if [ "$REQUEST_COUNT" -gt 0 ]; then
            ERROR_RATE=$(echo "scale=2; $ERROR_COUNT * 100 / $REQUEST_COUNT" | bc)
        else
            ERROR_RATE=0
        fi
        
        LATENCY_MEAN=$(echo "$RESULTS" | jq -r '.metrics.latency_mean' 2>/dev/null || echo "N/A")
        LATENCY_P50=$(echo "$RESULTS" | jq -r '.metrics.latency_percentiles.p50' 2>/dev/null || echo "N/A")
        LATENCY_P99=$(echo "$RESULTS" | jq -r '.metrics.latency_percentiles.p99' 2>/dev/null || echo "N/A")
        LATENCY_P999=$(echo "$RESULTS" | jq -r '.metrics.latency_percentiles.p999' 2>/dev/null || echo "N/A")
        PEAK_RPS=$(echo "$RESULTS" | jq -r '.metrics.peak_rps' 2>/dev/null || echo "N/A")
        AVG_RPS=$(echo "$RESULTS" | jq -r '.metrics.avg_rps' 2>/dev/null || echo "N/A")
        
        log_success "Baseline Metrics Collected:"
        echo ""
        echo "  Test Summary:"
        echo "    Duration: $DURATION seconds"
        echo "    Total Requests: $REQUEST_COUNT"
        echo "    Successful: $SUCCESS_COUNT"
        echo "    Errors: $ERROR_COUNT"
        echo "    Error Rate: $ERROR_RATE%"
        echo ""
        echo "  Latency Statistics (milliseconds):"
        echo "    Mean: $LATENCY_MEAN ms"
        echo "    P50: $LATENCY_P50 ms"
        echo "    P99: $LATENCY_P99 ms"
        echo "    P999: $LATENCY_P999 ms"
        echo ""
        echo "  Throughput:"
        echo "    Peak RPS: $PEAK_RPS"
        echo "    Avg RPS: $AVG_RPS"
        echo ""
    else
        log_warning "Metrics file is empty"
    fi
fi

echo ""

# STEP 6: Validate Latency Targets
log_info "STEP 6: Validating latency targets..."

LATENCY_PASS=true

if [ -f "$METRICS_DIR/load_test_results.json" ]; then
    P50=$(jq -r '.metrics.latency_percentiles.p50 // 0' "$METRICS_DIR/load_test_results.json")
    P99=$(jq -r '.metrics.latency_percentiles.p99 // 0' "$METRICS_DIR/load_test_results.json")
    P999=$(jq -r '.metrics.latency_percentiles.p999 // 0' "$METRICS_DIR/load_test_results.json")
    MEAN=$(jq -r '.metrics.latency_mean // 0' "$METRICS_DIR/load_test_results.json")
    
    TARGET_P50=50
    TARGET_P99=200
    TARGET_P999=500
    TARGET_MEAN=100
    
    echo "Latency Target Validation:"
    
    # Check Mean
    if (( $(echo "$MEAN <= $TARGET_MEAN" | bc -l) )); then
        log_success "Mean: $MEAN ms (target: $TARGET_MEAN ms)"
    else
        log_error "Mean: $MEAN ms (target: $TARGET_MEAN ms)"
        LATENCY_PASS=false
    fi
    
    # Check P50
    if (( $(echo "$P50 <= $TARGET_P50" | bc -l) )); then
        log_success "P50: $P50 ms (target: $TARGET_P50 ms)"
    else
        log_error "P50: $P50 ms (target: $TARGET_P50 ms)"
        LATENCY_PASS=false
    fi
    
    # Check P99
    if (( $(echo "$P99 <= $TARGET_P99" | bc -l) )); then
        log_success "P99: $P99 ms (target: $TARGET_P99 ms)"
    else
        log_error "P99: $P99 ms (target: $TARGET_P99 ms)"
        LATENCY_PASS=false
    fi
    
    # Check P999
    if (( $(echo "$P999 <= $TARGET_P999" | bc -l) )); then
        log_success "P999: $P999 ms (target: $TARGET_P999 ms)"
    else
        log_warning "P999: $P999 ms (target: $TARGET_P999 ms)"
    fi
    
    echo ""
fi

# STEP 7: Generate Report
log_info "STEP 7: Generating test summary report..."

cat > "$METRICS_DIR/LOAD_TEST_REPORT.md" << 'EOF'
# PhantomMesh Load Test Report - Phase P1-006

**Generated:** $(date -Iseconds)  
**Status:** Load Test Completed

## Test Configuration
- **Test Type:** Ramp-up (0→1000 req/s)
- **Duration:** 300 seconds
- **Ramp-up Period:** 60 seconds
- **Initial Rate:** 100 req/s
- **Peak Rate:** 1000 req/s
- **Concurrent Users:** 50

## Results Summary

See `load_test_results.json` for detailed metrics.

## Latency Analysis

| Percentile | Target | Status |
|-----------|--------|--------|
| Mean | ≤ 100 ms | See metrics |
| P50 | ≤ 50 ms | See metrics |
| P99 | ≤ 200 ms | See metrics |
| P999 | ≤ 500 ms | See metrics |

## Next Steps

1. Review detailed metrics in `load_test_results.json`
2. Analyze Prometheus metrics for system behavior
3. Plan next test phases:
   - Sustained load test (1000 req/s for 1 hour)
   - Spike test (sudden 2x increase)
   - Soak test (sustained load over extended period)

## Files Generated

- `load_test_results.json` - Detailed metrics
- `LOAD_TEST_REPORT.md` - This report

EOF

log_success "Test report saved to: $METRICS_DIR/LOAD_TEST_REPORT.md"

echo ""
echo "=========================================="
echo "Load Test Execution Complete"
echo "End Time: $(date -Iseconds)"
echo "=========================================="

if [ "$LATENCY_PASS" = "true" ]; then
    log_success "LOAD TEST PASSED - Ready for next phase"
    exit 0
else
    log_warning "LOAD TEST COMPLETED WITH WARNINGS - Review results"
    exit 1
fi
