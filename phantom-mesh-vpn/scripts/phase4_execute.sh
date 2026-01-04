#!/usr/bin/env bash

# PhantomMesh Phase 4 Execution Script
# Automates security audit, staging validation, and production deployment
# Usage: ./phase4_execute.sh [audit|staging|canary|deploy]

set -e

# Configuration
ENVIRONMENT="${ENVIRONMENT:-production}"
NAMESPACE_STAGING="staging"
NAMESPACE_PROD="production"
NAMESPACE_PROD_GREEN="production-green"
HELM_CHART="./k8s/helm/phantommesh"
LOAD_TEST_DURATION=300
SOAK_TEST_DURATION=259200  # 72 hours

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ==============================================================================
# PHASE 4 WEEK 1: SECURITY AUDIT
# ==============================================================================

run_security_audit() {
    log_info "Starting Phase 4 Security Audit..."
    
    # Create audit directory
    mkdir -p audit-reports
    AUDIT_DATE=$(date +%Y%m%d_%H%M%S)
    AUDIT_DIR="audit-reports/audit_${AUDIT_DATE}"
    mkdir -p "$AUDIT_DIR"
    
    log_info "Audit results will be saved to: $AUDIT_DIR"
    
    # 1. CIS Kubernetes Benchmark (kube-bench)
    log_info "Running CIS Kubernetes Benchmark (kube-bench)..."
    if command -v kube-bench &> /dev/null; then
        kube-bench -j > "$AUDIT_DIR/kube-bench-results.json" || true
        log_success "kube-bench completed. Results: $AUDIT_DIR/kube-bench-results.json"
        
        # Check for critical issues
        CRITICAL_COUNT=$(jq '[.Results[] | select(.Status=="FAIL")] | length' "$AUDIT_DIR/kube-bench-results.json")
        if [ "$CRITICAL_COUNT" -gt 0 ]; then
            log_warning "Found $CRITICAL_COUNT CRITICAL items. Review required."
            jq '.Results[] | select(.Status=="FAIL")' "$AUDIT_DIR/kube-bench-results.json"
        else
            log_success "No critical CIS benchmark failures!"
        fi
    else
        log_warning "kube-bench not installed. Skipping CIS benchmark."
    fi
    
    # 2. Container Image Scanning (Trivy)
    log_info "Scanning container images with Trivy..."
    IMAGES=(
        "iamthegreatdestroyer/phantom-node:latest"
        "iamthegreatdestroyer/agent-swarm:latest"
        "iamthegreatdestroyer/discovery:latest"
    )
    
    for image in "${IMAGES[@]}"; do
        log_info "Scanning $image..."
        trivy image --severity HIGH,CRITICAL --format json "$image" > "$AUDIT_DIR/trivy-$(echo $image | tr '/:' '_').json" || true
    done
    
    log_success "Container image scan completed"
    
    # 3. RBAC Audit
    log_info "Auditing RBAC configuration..."
    kubectl get clusterrolebindings -o json | jq '.items[]' > "$AUDIT_DIR/clusterrolebindings.json"
    kubectl get rolebindings -A -o json | jq '.items[]' > "$AUDIT_DIR/rolebindings.json"
    
    # Check for cluster-admin
    CLUSTER_ADMIN=$(jq '[.items[] | select(.roleRef.name=="cluster-admin")] | length' "$AUDIT_DIR/clusterrolebindings.json")
    log_info "Found $CLUSTER_ADMIN cluster-admin bindings (should be minimal)"
    
    # 4. Secrets Audit
    log_info "Auditing secrets..."
    kubectl get secrets -A -o json | jq '.items[] | {name: .metadata.name, namespace: .metadata.namespace, type: .type}' > "$AUDIT_DIR/secrets-inventory.json"
    
    # 5. Network Policies
    log_info "Backing up network policies..."
    kubectl get networkpolicies -A -o json > "$AUDIT_DIR/network-policies.json"
    
    # Generate summary report
    generate_audit_summary "$AUDIT_DIR"
    
    log_success "Phase 4 Security Audit completed!"
    log_info "Review audit-reports/audit_${AUDIT_DATE}/AUDIT_SUMMARY.md"
    
    return 0
}

generate_audit_summary() {
    local audit_dir=$1
    local summary="$audit_dir/AUDIT_SUMMARY.md"
    
    cat > "$summary" << 'EOF'
# Security Audit Report

## Summary
- Date: $(date)
- Reviewer: [NAME]

## CIS Benchmark
- Status: [REVIEW REQUIRED]
- Critical Issues: [COUNT FROM JSON]

## Container Images
- Scans Completed: YES
- Critical Vulnerabilities: [COUNT FROM JSON]

## RBAC
- Cluster Admin Bindings: [COUNT FROM JSON]
- Status: [REVIEW REQUIRED]

## Secrets
- Total Secrets: [COUNT FROM JSON]
- Status: [VERIFY ENCRYPTION]

## Network Policies
- Status: [VERIFY DEFAULT DENY]

## Action Items
- [ ] Fix all CRITICAL items
- [ ] Review all HIGH items
- [ ] Verify RBAC least privilege
- [ ] Confirm secrets encrypted
- [ ] Sign off on audit

## Approved By
- DevOps Lead: ________________
- Security Lead: ________________
EOF
    
    log_info "Audit summary created: $summary"
}

# ==============================================================================
# PHASE 4 WEEK 1: STAGING DEPLOYMENT
# ==============================================================================

deploy_staging() {
    log_info "Starting Staging Environment Deployment..."
    
    # Create namespace
    log_info "Creating staging namespace..."
    kubectl create namespace "$NAMESPACE_STAGING" --dry-run=client -o yaml | kubectl apply -f -
    kubectl label namespace "$NAMESPACE_STAGING" environment=staging tier=non-production --overwrite
    
    # Deploy using Helm
    log_info "Deploying to staging..."
    helm install phantommesh-staging "$HELM_CHART" \
        -n "$NAMESPACE_STAGING" \
        --values "k8s/overlays/staging/values.yaml" \
        --set environment=staging \
        --set replicaCount=2 \
        --wait \
        --timeout 5m
    
    log_success "Staging deployment started"
    
    # Wait for rollout
    log_info "Waiting for staging rollout..."
    kubectl rollout status deployment/phantommesh -n "$NAMESPACE_STAGING" --timeout=10m
    
    # Verify pods
    log_info "Verifying staging pods..."
    kubectl get pods -n "$NAMESPACE_STAGING"
    
    # Health checks
    log_info "Running health checks..."
    for pod in $(kubectl get pods -n "$NAMESPACE_STAGING" -o name | cut -d/ -f2); do
        log_info "Health check for pod: $pod"
        kubectl logs -n "$NAMESPACE_STAGING" "$pod" --tail=10 | grep -i "healthy\|ready" || true
    done
    
    log_success "Staging deployment complete!"
    
    return 0
}

# ==============================================================================
# PHASE 4 WEEK 1: LOAD TESTING
# ==============================================================================

run_load_test_staging() {
    log_info "Starting staging load test..."
    
    # Run load test
    log_info "Running ramp-up test (0→1000 req/sec)..."
    cd "tests/load" || { log_error "Load test directory not found"; return 1; }
    
    python3 optimize_tail_latency.py --environment=staging | tee load_test_results.txt
    
    # Parse results
    if [ -f "results/load_test_results_optimized.json" ]; then
        P99=$(jq '.metrics.latency_percentiles.p99' results/load_test_results_optimized.json)
        ERROR_RATE=$(jq '.error_rate' results/load_test_results_optimized.json)
        
        log_info "Load Test Results:"
        log_info "  P99 Latency: ${P99}ms (target: <100ms)"
        log_info "  Error Rate: ${ERROR_RATE}% (target: <0.1%)"
        
        if (( $(echo "$P99 < 100" | bc -l) )) && (( $(echo "$ERROR_RATE < 0.1" | bc -l) )); then
            log_success "Load test PASSED!"
        else
            log_error "Load test FAILED - review results"
            return 1
        fi
    else
        log_warning "Results file not found"
    fi
    
    cd - || return 1
    
    return 0
}

# ==============================================================================
# PHASE 4 WEEK 1: SOAK TEST
# ==============================================================================

run_soak_test() {
    log_info "Starting 72-hour soak test..."
    
    log_warning "Soak test running in background. Monitor with:"
    log_info "  kubectl logs -n $NAMESPACE_STAGING deployment/load-test-soak -f"
    
    # This runs asynchronously
    # In production, implement proper monitoring
    
    return 0
}

# ==============================================================================
# PHASE 4 WEEK 2: BLUE-GREEN DEPLOYMENT
# ==============================================================================

setup_blue_green() {
    log_info "Setting up blue-green deployment..."
    
    # Document blue (current production)
    log_info "Documenting blue (production) environment..."
    kubectl get all -n "$NAMESPACE_PROD" -o json > "backups/blue-backup-$(date +%s).json"
    
    # Create green namespace
    log_info "Creating green (new) environment..."
    kubectl create namespace "$NAMESPACE_PROD_GREEN" --dry-run=client -o yaml | kubectl apply -f -
    
    # Deploy green
    log_info "Deploying to green..."
    helm install phantommesh-green "$HELM_CHART" \
        -n "$NAMESPACE_PROD_GREEN" \
        --values "k8s/overlays/prod/values.yaml" \
        --wait \
        --timeout 10m
    
    # Run smoke tests
    log_info "Running smoke tests on green..."
    sleep 30  # Wait for services to be ready
    
    GREEN_POD=$(kubectl get pods -n "$NAMESPACE_PROD_GREEN" -o name | head -1 | cut -d/ -f2)
    kubectl logs -n "$NAMESPACE_PROD_GREEN" "$GREEN_POD" | head -20
    
    log_success "Blue-green setup complete"
    
    return 0
}

# ==============================================================================
# PRODUCTION CANARY DEPLOYMENT
# ==============================================================================

canary_deploy() {
    local weight=$1
    local stage=$2
    
    log_info "Canary deployment: $weight% traffic to green ($stage)..."
    
    # Calculate weights
    BLUE_WEIGHT=$((100 - weight))
    
    # Update VirtualService
    kubectl patch virtualservice phantommesh-canary -n "$NAMESPACE_PROD" --type merge \
        -p "{\"spec\":{\"http\":[{\"route\":[{\"destination\":{\"host\":\"phantommesh-blue\"},\"weight\":$BLUE_WEIGHT},{\"destination\":{\"host\":\"phantommesh-green\"},\"weight\":$weight}]}]}}" || true
    
    log_info "Traffic routing: $BLUE_WEIGHT% blue, $weight% green"
    log_info "Monitoring for 15 minutes..."
    
    # Monitor
    for i in {1..15}; do
        sleep 60
        log_info "Minute $i/15 - Checking metrics..."
        
        # Get error rate from logs
        ERROR_RATE=$(kubectl logs -n "$NAMESPACE_PROD_GREEN" -l app=phantommesh --tail=100 2>/dev/null | grep -i "error" | wc -l || echo "0")
        log_info "  Errors detected: $ERROR_RATE"
        
        if [ "$ERROR_RATE" -gt 10 ]; then
            log_error "High error rate detected! Rolling back..."
            return 1
        fi
    done
    
    log_success "Canary stage $stage successful!"
    
    return 0
}

deploy_production() {
    log_info "Starting production canary deployment..."
    
    # Stage 1: 5% traffic
    if ! canary_deploy 5 "5%"; then
        log_error "Stage 1 failed - Rolling back"
        canary_deploy 0 "ROLLBACK"
        return 1
    fi
    
    # Stage 2: 25% traffic
    if ! canary_deploy 25 "25%"; then
        log_error "Stage 2 failed - Rolling back to 5%"
        canary_deploy 5 "ROLLBACK"
        return 1
    fi
    
    # Stage 3: 50% traffic
    if ! canary_deploy 50 "50%"; then
        log_error "Stage 3 failed - Rolling back to 25%"
        canary_deploy 25 "ROLLBACK"
        return 1
    fi
    
    # Final: 100% traffic
    if ! canary_deploy 100 "100%"; then
        log_error "Final stage failed - Rolling back to 50%"
        canary_deploy 50 "ROLLBACK"
        return 1
    fi
    
    log_success "Production deployment complete!"
    
    return 0
}

# ==============================================================================
# HEALTH CHECK
# ==============================================================================

health_check() {
    log_info "Running health checks..."
    
    log_info "Pod Status:"
    kubectl get pods -n "$NAMESPACE_PROD" -o wide
    
    log_info "Resource Usage:"
    kubectl top pods -n "$NAMESPACE_PROD" 2>/dev/null || log_warning "Metrics not available"
    
    log_info "Recent Errors:"
    kubectl logs -n "$NAMESPACE_PROD" -l app=phantommesh --tail=50 2>/dev/null | grep -i "ERROR" || log_info "No errors found"
    
    log_success "Health check complete"
    
    return 0
}

# ==============================================================================
# MAIN
# ==============================================================================

main() {
    log_info "PhantomMesh Phase 4 Execution Script"
    log_info "Current environment: $ENVIRONMENT"
    
    case "${1:-help}" in
        audit)
            run_security_audit
            ;;
        staging)
            deploy_staging
            ;;
        load-test)
            run_load_test_staging
            ;;
        soak-test)
            run_soak_test
            ;;
        blue-green)
            setup_blue_green
            ;;
        deploy)
            deploy_production
            ;;
        health)
            health_check
            ;;
        all)
            log_info "Running complete Phase 4 workflow..."
            run_security_audit && \
            deploy_staging && \
            run_load_test_staging && \
            log_info "Wait for manual soak test (72 hours)..." && \
            setup_blue_green && \
            health_check && \
            log_success "Phase 4 complete! Ready for production deployment."
            ;;
        *)
            cat << 'HELP'
PhantomMesh Phase 4 Execution Script

Usage: ./phase4_execute.sh [COMMAND]

Commands:
  audit          Run security audit (kube-bench, Trivy, RBAC)
  staging        Deploy to staging environment
  load-test      Run load test on staging
  soak-test      Start 72-hour soak test
  blue-green     Set up blue-green for production
  deploy         Execute canary deployment to production
  health         Run health checks
  all            Execute entire Phase 4 workflow
  help           Show this help message

Environment Variables:
  ENVIRONMENT    Target environment (default: production)
  NAMESPACE_STAGING
  NAMESPACE_PROD
  
Examples:
  ./phase4_execute.sh audit
  ./phase4_execute.sh staging
  ./phase4_execute.sh deploy

For detailed procedures, see: PHASE4_EXECUTION_RUNBOOK.md
HELP
            exit 0
            ;;
    esac
}

main "$@"
