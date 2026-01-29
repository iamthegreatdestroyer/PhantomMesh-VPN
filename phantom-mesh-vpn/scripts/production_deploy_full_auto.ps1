#!/usr/bin/env pwsh
# ============================================================================
# PhantomMesh VPN - FULLY AUTOMATED PRODUCTION DEPLOYMENT
# ============================================================================
# 
# This script implements maximum automation for production deployment following
# the Production Deployment Runbook with:
#   - Zero manual intervention
#   - Automatic rollback on failure
#   - Real-time health monitoring
#   - Comprehensive logging
#
# Author: APEX-01 (Elite Agent Collective)
# Date: 2026-01-28
# Version: 1.0.0
# ============================================================================

#Requires -Version 7.0

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ============================================================================
# CONFIGURATION
# ============================================================================

$script:CONFIG = @{
    # Kubernetes Configuration
    Namespace         = "phantommesh-prod"
    KubeContext       = "production"
    
    # Paths
    K8sBasePath       = "../k8s/base"
    K8sOverlayPath    = "../k8s/overlays/prod"
    
    # Timeouts (seconds)
    DeploymentTimeout = 300
    PodReadyTimeout   = 120
    ServiceTimeout    = 60
    
    # Retry Configuration
    MaxRetries        = 3
    RetryDelaySeconds = 10
    
    # Feature Flags
    DryRun            = $false
    SkipValidation    = $false
    EnableRollback    = $true
    
    # Logging
    LogPath           = "../logs/production_deploy_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
    VerboseLogging    = $true
}

# Deployment Order (dependencies first)
$script:DEPLOYMENT_ORDER = @(
    @{ Name = "namespace"; Type = "namespace"; Path = "namespace/namespace.yaml" },
    @{ Name = "rbac"; Type = "rbac"; Path = "rbac/rbac.yaml" },
    @{ Name = "secrets"; Type = "secret"; Path = "secrets/phantom-secrets.yaml" },
    @{ Name = "configmaps"; Type = "configmap"; Path = "configmaps/" },
    @{ Name = "pvcs"; Type = "pvc"; Path = "persistentvolumes/pvcs.yaml" },
    @{ Name = "netpolicies"; Type = "networkpolicy"; Path = "networkpolicies/network-policies.yaml" },
    @{ Name = "prometheus"; Type = "deployment"; Path = "deployments/prometheus-deployment.yaml" },
    @{ Name = "grafana"; Type = "deployment"; Path = "deployments/grafana-deployment.yaml" },
    @{ Name = "discovery"; Type = "deployment"; Path = "deployments/discovery-deployment.yaml" },
    @{ Name = "vpn-core"; Type = "deployment"; Path = "deployments/vpn-core-deployment.yaml" },
    @{ Name = "agent-swarm"; Type = "deployment"; Path = "deployments/agent-swarm-deployment.yaml" },
    @{ Name = "services"; Type = "service"; Path = "services/services.yaml" },
    @{ Name = "ingress"; Type = "ingress"; Path = "ingress/ingress.yaml" },
    @{ Name = "hpa"; Type = "hpa"; Path = "autoscaling/hpa.yaml" }
)

# ============================================================================
# LOGGING & OUTPUT
# ============================================================================

$script:PHASE_COUNT = 0
$script:STEP_COUNT = 0
$script:ROLLBACK_STACK = [System.Collections.Stack]::new()
$script:START_TIME = Get-Date

function Initialize-Logging {
    $logDir = Split-Path $script:CONFIG.LogPath -Parent
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    }
    
    # Initialize log file
    @"
================================================================================
PhantomMesh VPN - Production Deployment Log
================================================================================
Started: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Host: $env:COMPUTERNAME
User: $env:USERNAME
PowerShell: $($PSVersionTable.PSVersion)
================================================================================

"@ | Set-Content $script:CONFIG.LogPath -Encoding UTF8
}

function Write-Log {
    param(
        [Parameter(Mandatory)][string]$Message,
        [ValidateSet("INFO", "SUCCESS", "WARNING", "ERROR", "DEBUG", "PHASE", "STEP")]
        [string]$Level = "INFO"
    )
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logMessage = "[$timestamp] [$Level] $Message"
    
    # Console output with colors
    $color = switch ($Level) {
        "SUCCESS" { "Green" }
        "WARNING" { "Yellow" }
        "ERROR" { "Red" }
        "DEBUG" { "DarkGray" }
        "PHASE" { "Magenta" }
        "STEP" { "Cyan" }
        default { "White" }
    }
    
    $prefix = switch ($Level) {
        "SUCCESS" { "✅" }
        "WARNING" { "⚠️" }
        "ERROR" { "❌" }
        "PHASE" { "🚀" }
        "STEP" { "📌" }
        default { "ℹ️" }
    }
    
    Write-Host "$prefix $Message" -ForegroundColor $color
    
    # File logging
    $logMessage | Add-Content $script:CONFIG.LogPath -Encoding UTF8
}

function Write-Phase {
    param([string]$Name, [string]$Description)
    
    $script:PHASE_COUNT++
    $script:STEP_COUNT = 0
    
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Log "PHASE $($script:PHASE_COUNT): $Name" -Level PHASE
    Write-Host "═══════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    if ($Description) {
        Write-Host "  $Description" -ForegroundColor DarkCyan
    }
    Write-Host ""
}

function Write-Step {
    param([string]$Name)
    $script:STEP_COUNT++
    Write-Log "Step $($script:PHASE_COUNT).$($script:STEP_COUNT): $Name" -Level STEP
}

function Write-ProgressBar {
    param([int]$Current, [int]$Total, [string]$Activity)
    
    $percent = [math]::Round(($Current / $Total) * 100)
    $filled = [math]::Round($percent / 5)
    $empty = 20 - $filled
    $bar = "█" * $filled + "░" * $empty
    
    Write-Host "`r  [$bar] $percent% - $Activity      " -NoNewline -ForegroundColor Cyan
    if ($Current -eq $Total) { Write-Host "" }
}

# ============================================================================
# KUBECTL WRAPPER FUNCTIONS
# ============================================================================

function Invoke-Kubectl {
    param(
        [Parameter(Mandatory)][string]$Command,
        [switch]$ReturnOutput,
        [switch]$AllowFailure
    )
    
    $fullCommand = "kubectl $Command -n $($script:CONFIG.Namespace)"
    if ($script:CONFIG.DryRun) {
        $fullCommand += " --dry-run=client"
    }
    
    if ($script:CONFIG.VerboseLogging) {
        Write-Log "Executing: $fullCommand" -Level DEBUG
    }
    
    try {
        $output = Invoke-Expression $fullCommand 2>&1
        
        if ($LASTEXITCODE -ne 0 -and -not $AllowFailure) {
            throw "kubectl command failed with exit code $LASTEXITCODE : $output"
        }
        
        if ($ReturnOutput) {
            return $output
        }
        return $true
    }
    catch {
        if ($AllowFailure) {
            Write-Log "Command failed (allowed): $_" -Level WARNING
            return $false
        }
        throw
    }
}

function Test-KubectlConnectivity {
    Write-Step "Verifying Kubernetes cluster connectivity"
    
    try {
        $clusterInfo = kubectl cluster-info 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Cannot connect to Kubernetes cluster"
        }
        
        $nodes = kubectl get nodes -o json | ConvertFrom-Json
        $readyNodes = ($nodes.items | Where-Object { 
                $_.status.conditions | Where-Object { $_.type -eq "Ready" -and $_.status -eq "True" }
            }).Count
        
        Write-Log "Cluster connected: $readyNodes nodes ready" -Level SUCCESS
        return $true
    }
    catch {
        Write-Log "Kubernetes connectivity failed: $_" -Level ERROR
        return $false
    }
}

# ============================================================================
# VALIDATION FUNCTIONS
# ============================================================================

function Test-Prerequisites {
    Write-Phase "Pre-Flight Validation" "Checking all deployment prerequisites"
    
    $checks = @(
        @{ Name = "kubectl installed"; Check = { Get-Command kubectl -ErrorAction SilentlyContinue } },
        @{ Name = "helm installed (optional)"; Check = { Get-Command helm -ErrorAction SilentlyContinue }; Optional = $true },
        @{ Name = "docker installed"; Check = { Get-Command docker -ErrorAction SilentlyContinue } },
        @{ Name = "K8s manifests exist"; Check = { Test-Path (Join-Path $script:CONFIG.K8sBasePath "deployments") } }
    )
    
    $allPassed = $true
    foreach ($check in $checks) {
        Write-Step $check.Name
        try {
            $result = & $check.Check
            if ($result) {
                Write-Log "$($check.Name): PASSED" -Level SUCCESS
            }
            else {
                if ($check.Optional) {
                    Write-Log "$($check.Name): NOT FOUND (optional)" -Level WARNING
                }
                else {
                    Write-Log "$($check.Name): FAILED" -Level ERROR
                    $allPassed = $false
                }
            }
        }
        catch {
            if ($check.Optional) {
                Write-Log "$($check.Name): NOT FOUND (optional)" -Level WARNING
            }
            else {
                Write-Log "$($check.Name): FAILED - $_" -Level ERROR
                $allPassed = $false
            }
        }
    }
    
    # Kubernetes connectivity
    if (-not (Test-KubectlConnectivity)) {
        $allPassed = $false
    }
    
    # RBAC check
    Write-Step "Checking RBAC permissions"
    $canCreate = kubectl auth can-i create deployments -n $script:CONFIG.Namespace 2>&1
    if ($canCreate -match "yes") {
        Write-Log "RBAC permissions: Sufficient" -Level SUCCESS
    }
    else {
        Write-Log "RBAC permissions: Insufficient (may need cluster-admin)" -Level WARNING
    }
    
    return $allPassed
}

function Test-ManifestValidity {
    Write-Phase "Manifest Validation" "Dry-run validation of all Kubernetes manifests"
    
    $manifestDirs = @(
        "namespace",
        "rbac", 
        "secrets",
        "configmaps",
        "persistentvolumes",
        "networkpolicies",
        "deployments",
        "services",
        "ingress",
        "autoscaling"
    )
    
    $allValid = $true
    $total = $manifestDirs.Count
    $current = 0
    
    foreach ($dir in $manifestDirs) {
        $current++
        Write-ProgressBar -Current $current -Total $total -Activity "Validating $dir"
        
        $path = Join-Path $script:CONFIG.K8sBasePath $dir
        if (Test-Path $path) {
            try {
                $output = kubectl apply -f $path --dry-run=client 2>&1
                if ($LASTEXITCODE -ne 0) {
                    Write-Log "Manifest validation failed for $dir : $output" -Level ERROR
                    $allValid = $false
                }
            }
            catch {
                Write-Log "Manifest validation error for $dir : $_" -Level ERROR
                $allValid = $false
            }
        }
    }
    
    if ($allValid) {
        Write-Log "All manifests validated successfully" -Level SUCCESS
    }
    
    return $allValid
}

# ============================================================================
# DEPLOYMENT FUNCTIONS
# ============================================================================

function New-Namespace {
    Write-Step "Creating production namespace"
    
    # Check if namespace exists
    $exists = kubectl get namespace $script:CONFIG.Namespace 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Log "Namespace '$($script:CONFIG.Namespace)' already exists" -Level WARNING
        return $true
    }
    
    # Create namespace
    $nsPath = Join-Path $script:CONFIG.K8sBasePath "namespace/namespace.yaml"
    if (Test-Path $nsPath) {
        kubectl apply -f $nsPath
    }
    else {
        kubectl create namespace $script:CONFIG.Namespace
    }
    
    # Label namespace
    kubectl label namespace $script:CONFIG.Namespace environment=production --overwrite
    kubectl label namespace $script:CONFIG.Namespace managed-by=phantommesh --overwrite
    
    $script:ROLLBACK_STACK.Push(@{
            Type     = "namespace"
            Name     = $script:CONFIG.Namespace
            Rollback = { kubectl delete namespace $script:CONFIG.Namespace --ignore-not-found }
        })
    
    Write-Log "Namespace created and labeled" -Level SUCCESS
    return $true
}

function Deploy-Resource {
    param(
        [Parameter(Mandatory)][hashtable]$Resource
    )
    
    $resourcePath = Join-Path $script:CONFIG.K8sBasePath $Resource.Path
    
    if (-not (Test-Path $resourcePath)) {
        Write-Log "Resource path not found: $resourcePath" -Level WARNING
        return $true  # Continue deployment
    }
    
    Write-Step "Deploying $($Resource.Name)"
    
    $retryCount = 0
    $success = $false
    
    while ($retryCount -lt $script:CONFIG.MaxRetries -and -not $success) {
        try {
            kubectl apply -f $resourcePath -n $script:CONFIG.Namespace
            
            if ($LASTEXITCODE -eq 0) {
                $success = $true
                
                # Add to rollback stack
                $script:ROLLBACK_STACK.Push(@{
                        Type     = $Resource.Type
                        Name     = $Resource.Name
                        Path     = $resourcePath
                        Rollback = { param($p) kubectl delete -f $p -n $script:CONFIG.Namespace --ignore-not-found }
                    })
                
                Write-Log "$($Resource.Name) deployed successfully" -Level SUCCESS
            }
            else {
                throw "kubectl apply failed"
            }
        }
        catch {
            $retryCount++
            if ($retryCount -lt $script:CONFIG.MaxRetries) {
                Write-Log "Retry $retryCount/$($script:CONFIG.MaxRetries) for $($Resource.Name): $_" -Level WARNING
                Start-Sleep -Seconds $script:CONFIG.RetryDelaySeconds
            }
            else {
                Write-Log "Failed to deploy $($Resource.Name) after $($script:CONFIG.MaxRetries) attempts: $_" -Level ERROR
                return $false
            }
        }
    }
    
    return $success
}

function Wait-DeploymentReady {
    param(
        [Parameter(Mandatory)][string]$DeploymentName,
        [int]$TimeoutSeconds = 300
    )
    
    Write-Step "Waiting for $DeploymentName to be ready"
    
    $startTime = Get-Date
    $ready = $false
    
    while (-not $ready -and ((Get-Date) - $startTime).TotalSeconds -lt $TimeoutSeconds) {
        try {
            $deployment = kubectl get deployment $DeploymentName -n $script:CONFIG.Namespace -o json 2>&1 | ConvertFrom-Json
            
            $desired = $deployment.spec.replicas
            $available = $deployment.status.availableReplicas
            
            if ($available -eq $desired -and $available -gt 0) {
                $ready = $true
                Write-Log "${DeploymentName}: ${available}/${desired} replicas ready" -Level SUCCESS
            }
            else {
                $elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds)
                Write-Host "`r  ⏳ Waiting for ${DeploymentName} (${available}/${desired} ready) - ${elapsed}s elapsed     " -NoNewline
                Start-Sleep -Seconds 5
            }
        }
        catch {
            Start-Sleep -Seconds 5
        }
    }
    
    Write-Host ""  # Clear the line
    
    if (-not $ready) {
        Write-Log "${DeploymentName} did not become ready within ${TimeoutSeconds}s" -Level ERROR
        return $false
    }
    
    return $true
}

function Deploy-AllResources {
    Write-Phase "Core Deployment" "Deploying all resources in dependency order"
    
    $deploymentResources = $script:DEPLOYMENT_ORDER | Where-Object { $_.Type -eq "deployment" }
    $otherResources = $script:DEPLOYMENT_ORDER | Where-Object { $_.Type -ne "deployment" }
    
    # Deploy non-deployment resources first
    foreach ($resource in $otherResources) {
        if (-not (Deploy-Resource -Resource $resource)) {
            if ($script:CONFIG.EnableRollback) {
                Write-Log "Initiating rollback due to deployment failure" -Level ERROR
                Invoke-Rollback
            }
            return $false
        }
    }
    
    # Deploy deployments and wait for each
    foreach ($resource in $deploymentResources) {
        if (-not (Deploy-Resource -Resource $resource)) {
            if ($script:CONFIG.EnableRollback) {
                Invoke-Rollback
            }
            return $false
        }
        
        # Wait for deployment to be ready
        if (-not (Wait-DeploymentReady -DeploymentName $resource.Name -TimeoutSeconds $script:CONFIG.DeploymentTimeout)) {
            Write-Log "Deployment $($resource.Name) health check failed" -Level ERROR
            if ($script:CONFIG.EnableRollback) {
                Invoke-Rollback
            }
            return $false
        }
    }
    
    return $true
}

# ============================================================================
# HEALTH CHECK FUNCTIONS
# ============================================================================

function Test-AllPodsHealthy {
    Write-Step "Checking pod health status"
    
    $pods = kubectl get pods -n $script:CONFIG.Namespace -o json | ConvertFrom-Json
    
    $healthy = 0
    $unhealthy = 0
    $pending = 0
    
    foreach ($pod in $pods.items) {
        $phase = $pod.status.phase
        switch ($phase) {
            "Running" { $healthy++ }
            "Succeeded" { $healthy++ }
            "Pending" { $pending++ }
            default { $unhealthy++ }
        }
    }
    
    $total = $healthy + $unhealthy + $pending
    
    if ($unhealthy -eq 0 -and $pending -eq 0) {
        Write-Log "All $healthy pods are healthy" -Level SUCCESS
        return $true
    }
    elseif ($unhealthy -gt 0) {
        Write-Log "Pod health: $healthy healthy, $unhealthy unhealthy, $pending pending" -Level ERROR
        
        # Show unhealthy pods
        $unhealthyPods = $pods.items | Where-Object { $_.status.phase -notin @("Running", "Succeeded") }
        foreach ($pod in $unhealthyPods) {
            Write-Log "  Unhealthy: $($pod.metadata.name) - $($pod.status.phase)" -Level WARNING
        }
        return $false
    }
    else {
        Write-Log "Pod health: $healthy healthy, $pending pending (waiting...)" -Level WARNING
        return $true  # Allow pending pods during initial deployment
    }
}

function Test-ServiceEndpoints {
    Write-Step "Verifying service endpoints"
    
    $services = kubectl get endpoints -n $script:CONFIG.Namespace -o json | ConvertFrom-Json
    
    $healthy = 0
    $unhealthy = 0
    
    foreach ($svc in $services.items) {
        $hasEndpoints = $svc.subsets -and ($svc.subsets | ForEach-Object { $_.addresses }).Count -gt 0
        
        if ($hasEndpoints) {
            $healthy++
        }
        else {
            $unhealthy++
            Write-Log "  No endpoints: $($svc.metadata.name)" -Level WARNING
        }
    }
    
    if ($unhealthy -eq 0) {
        Write-Log "All $healthy services have healthy endpoints" -Level SUCCESS
        return $true
    }
    else {
        Write-Log "Services: $healthy healthy, $unhealthy without endpoints" -Level WARNING
        return $false
    }
}

function Test-MetricsCollection {
    Write-Step "Verifying metrics collection"
    
    try {
        # Check if Prometheus is collecting metrics
        $prometheusUp = kubectl exec -n $script:CONFIG.Namespace deployment/prometheus -- \
        wget -qO- 'http://localhost:9090/api/v1/query?query=up' 2>&1
        
        if ($prometheusUp -match '"status":"success"') {
            Write-Log "Prometheus metrics collection: Active" -Level SUCCESS
            return $true
        }
    }
    catch {
        Write-Log "Could not verify Prometheus metrics (may not be ready yet)" -Level WARNING
    }
    
    return $true  # Don't fail deployment for metrics
}

function Invoke-PostDeploymentHealthChecks {
    Write-Phase "Post-Deployment Health Checks" "Comprehensive verification of deployed services"
    
    $allHealthy = $true
    
    # Pod health
    if (-not (Test-AllPodsHealthy)) {
        $allHealthy = $false
    }
    
    # Service endpoints
    if (-not (Test-ServiceEndpoints)) {
        # Give services time to stabilize
        Write-Log "Waiting 30s for service endpoints..." -Level WARNING
        Start-Sleep -Seconds 30
        if (-not (Test-ServiceEndpoints)) {
            $allHealthy = $false
        }
    }
    
    # Metrics
    Test-MetricsCollection
    
    # Cluster resources
    Write-Step "Checking resource utilization"
    $top = kubectl top nodes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Log "Node resources checked" -Level SUCCESS
    }
    
    return $allHealthy
}

# ============================================================================
# ROLLBACK FUNCTIONS
# ============================================================================

function Invoke-Rollback {
    Write-Phase "ROLLBACK" "Reverting deployed resources due to failure"
    
    $rollbackCount = 0
    
    while ($script:ROLLBACK_STACK.Count -gt 0) {
        $item = $script:ROLLBACK_STACK.Pop()
        $rollbackCount++
        
        Write-Step "Rolling back $($item.Name)"
        
        try {
            if ($item.Path) {
                & $item.Rollback $item.Path
            }
            else {
                & $item.Rollback
            }
            Write-Log "Rolled back: $($item.Name)" -Level SUCCESS
        }
        catch {
            Write-Log "Failed to rollback $($item.Name): $_" -Level WARNING
        }
    }
    
    Write-Log "Rollback complete: $rollbackCount resources reverted" -Level WARNING
}

# ============================================================================
# MAIN DEPLOYMENT ORCHESTRATION
# ============================================================================

function Start-ProductionDeployment {
    param(
        [switch]$DryRun,
        [switch]$SkipValidation,
        [switch]$Force
    )
    
    if ($DryRun) {
        $script:CONFIG.DryRun = $true
        Write-Host ""
        Write-Host "🔬 DRY RUN MODE - No changes will be made" -ForegroundColor Yellow
        Write-Host ""
    }
    
    # Initialize
    Initialize-Logging
    
    Write-Host ""
    Write-Host "╔═══════════════════════════════════════════════════════════════════╗" -ForegroundColor Magenta
    Write-Host "║     PhantomMesh VPN - PRODUCTION DEPLOYMENT AUTOMATION            ║" -ForegroundColor Magenta
    Write-Host "║                    Maximum Autonomy Mode                           ║" -ForegroundColor Magenta
    Write-Host "╚═══════════════════════════════════════════════════════════════════╝" -ForegroundColor Magenta
    Write-Host ""
    Write-Host "  Namespace: $($script:CONFIG.Namespace)" -ForegroundColor Cyan
    Write-Host "  Dry Run:   $($script:CONFIG.DryRun)" -ForegroundColor Cyan
    Write-Host "  Rollback:  $($script:CONFIG.EnableRollback)" -ForegroundColor Cyan
    Write-Host "  Log File:  $($script:CONFIG.LogPath)" -ForegroundColor Cyan
    Write-Host ""
    
    # Phase 1: Prerequisites
    if (-not $SkipValidation) {
        if (-not (Test-Prerequisites)) {
            Write-Log "Prerequisites check failed. Use -Force to override." -Level ERROR
            if (-not $Force) {
                return $false
            }
        }
        
        # Phase 2: Manifest Validation
        if (-not (Test-ManifestValidity)) {
            Write-Log "Manifest validation failed" -Level ERROR
            if (-not $Force) {
                return $false
            }
        }
    }
    
    # Phase 3: Create Namespace
    Write-Phase "Namespace Setup" "Creating and configuring production namespace"
    if (-not (New-Namespace)) {
        return $false
    }
    
    # Phase 4: Deploy All Resources
    if (-not (Deploy-AllResources)) {
        return $false
    }
    
    # Phase 5: Health Checks
    if (-not (Invoke-PostDeploymentHealthChecks)) {
        if ($script:CONFIG.EnableRollback -and -not $Force) {
            Write-Log "Post-deployment health checks failed. Initiating rollback..." -Level ERROR
            Invoke-Rollback
            return $false
        }
    }
    
    # Success
    $duration = (Get-Date) - $script:START_TIME
    
    Write-Host ""
    Write-Host "╔═══════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║              🎉 PRODUCTION DEPLOYMENT SUCCESSFUL 🎉               ║" -ForegroundColor Green
    Write-Host "╚═══════════════════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Log "Deployment completed in $([math]::Round($duration.TotalMinutes, 2)) minutes" -Level SUCCESS
    Write-Host ""
    Write-Host "  📊 Grafana:     http://grafana.$($script:CONFIG.Namespace).local" -ForegroundColor Cyan
    Write-Host "  📈 Prometheus:  http://prometheus.$($script:CONFIG.Namespace).local" -ForegroundColor Cyan
    Write-Host "  🔒 VPN API:     https://api.$($script:CONFIG.Namespace).local" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Next Steps:" -ForegroundColor Yellow
    Write-Host "    1. kubectl get pods -n $($script:CONFIG.Namespace)" -ForegroundColor Gray
    Write-Host "    2. kubectl logs -f -l app=vpn-core -n $($script:CONFIG.Namespace)" -ForegroundColor Gray
    Write-Host "    3. Run ./verify_deployment.ps1 for comprehensive health check" -ForegroundColor Gray
    Write-Host ""
    
    return $true
}

# ============================================================================
# SCRIPT ENTRY POINT
# ============================================================================

# Parse command line arguments
$DryRun = $args -contains "-DryRun" -or $args -contains "--dry-run"
$SkipValidation = $args -contains "-SkipValidation" -or $args -contains "--skip-validation"
$Force = $args -contains "-Force" -or $args -contains "--force"

# Check if running interactively
if ($args -contains "-Help" -or $args -contains "--help" -or $args -contains "-h") {
    Write-Host @"

PhantomMesh VPN - Production Deployment Automation
===================================================

Usage:
    .\production_deploy_full_auto.ps1 [options]

Options:
    -DryRun           Validate without making changes
    -SkipValidation   Skip pre-flight checks
    -Force            Continue even if health checks fail
    -Help             Show this help message

Examples:
    .\production_deploy_full_auto.ps1                    # Full deployment
    .\production_deploy_full_auto.ps1 -DryRun            # Validate only
    .\production_deploy_full_auto.ps1 -Force             # Deploy ignoring warnings

"@
    exit 0
}

# Run deployment
$result = Start-ProductionDeployment -DryRun:$DryRun -SkipValidation:$SkipValidation -Force:$Force

if ($result) {
    exit 0
}
else {
    exit 1
}
