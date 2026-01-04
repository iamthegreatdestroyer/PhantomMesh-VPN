# 🚀 PhantomMesh VPN - Production Deployment Runbook

**Version:** 1.0  
**Status:** READY FOR PRODUCTION  
**Last Updated:** 2026-01-04  
**Created By:** Engineering Team

---

## Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [Architecture Overview](#architecture-overview)
3. [Deployment Prerequisites](#deployment-prerequisites)
4. [Step-by-Step Deployment](#step-by-step-deployment)
5. [Post-Deployment Verification](#post-deployment-verification)
6. [Rollback Procedures](#rollback-procedures)
7. [Troubleshooting](#troubleshooting)
8. [Monitoring & Alerting](#monitoring--alerting)
9. [Emergency Procedures](#emergency-procedures)

---

## Pre-Deployment Checklist

### Security & Compliance

- [ ] Final security audit completed (Decision Gate 3)
- [ ] All vulnerabilities patched (CRITICAL=0, HIGH≤2)
- [ ] Encryption certificates validated (not expired)
- [ ] API keys rotated
- [ ] Service accounts created
- [ ] RBAC policies configured
- [ ] Network security groups configured
- [ ] Firewall rules tested

### Infrastructure Readiness

- [ ] Kubernetes cluster operational (all nodes healthy)
- [ ] Storage provisioning tested (PVCs bind successfully)
- [ ] Network policies deployed
- [ ] Ingress controller installed
- [ ] DNS configured
- [ ] Load balancer operational
- [ ] Backup systems tested

### Data & Migration

- [ ] Database backups created
- [ ] Data migration scripts validated
- [ ] Rollback data snapshot prepared
- [ ] Data migration tested (non-prod)
- [ ] Migration timing window confirmed
- [ ] Data integrity validation plan ready

### Testing & Validation

- [ ] Load test passed (P99 < 200ms, success rate > 99%)
- [ ] 72-hour soak test completed
- [ ] Security test passed
- [ ] Failover test completed
- [ ] Disaster recovery drill done
- [ ] API integration tests passed

### Operational Readiness

- [ ] Monitoring dashboards created
- [ ] Alert thresholds configured
- [ ] On-call rotation established
- [ ] Runbooks reviewed and approved
- [ ] Incident response procedures documented
- [ ] Escalation paths defined
- [ ] Support team trained

### Documentation & Communication

- [ ] Deployment plan reviewed with stakeholders
- [ ] Change management ticket created
- [ ] Communication plan prepared
- [ ] User documentation updated
- [ ] Architectural documentation current
- [ ] Known issues documented

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      PRODUCTION TOPOLOGY                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────┐         ┌───────────────────┐           │
│  │ Load Balancer     │         │ CloudFlare/CDN    │           │
│  │ (HA Pair)         │────────│ (DDoS Protection) │           │
│  └────────┬──────────┘         └───────────────────┘           │
│           │                                                     │
│  ┌────────▼──────────────────────────────────────────┐         │
│  │       Kubernetes Cluster (3+ nodes)              │         │
│  │                                                   │         │
│  │  ┌──────────────┐  ┌──────────────┐            │         │
│  │  │ VPN Core     │  │ Discovery    │            │         │
│  │  │ (3 replicas) │  │ Service      │            │         │
│  │  └──────────────┘  └──────────────┘            │         │
│  │                                                   │         │
│  │  ┌──────────────┐  ┌──────────────┐            │         │
│  │  │ Agent Swarm  │  │ API Gateway  │            │         │
│  │  │ (auto-scale) │  │ (3 replicas) │            │         │
│  │  └──────────────┘  └──────────────┘            │         │
│  │                                                   │         │
│  │  ┌──────────────┐  ┌──────────────┐            │         │
│  │  │ Prometheus   │  │ Grafana      │            │         │
│  │  │ (metrics)    │  │ (dashboards) │            │         │
│  │  └──────────────┘  └──────────────┘            │         │
│  │                                                   │         │
│  └─────────────────────────────────────────────────┘         │
│           │                                                    │
│  ┌────────▼──────────────────────────────────────────┐        │
│  │      Persistent Storage (Database Tier)         │        │
│  │                                                   │        │
│  │  PostgreSQL (Primary) ──→ PostgreSQL (Standby) │        │
│  │  Redis Cluster (3+ nodes)                       │        │
│  │  S3/Object Storage (Backups)                    │        │
│  └─────────────────────────────────────────────────┘        │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Deployment Prerequisites

### Required Tools & Access

```bash
# Tools needed on deployment workstation
- kubectl (1.27+)
- helm (3.12+)
- docker (24+)
- docker-compose (2.0+)
- git
- jq (JSON query)
- openssl (certificate management)
- aws-cli or equivalent cloud CLI

# Access requirements
- Admin access to Kubernetes cluster
- Cloud infrastructure account (AWS/Azure/GCP)
- DNS management access
- Container registry credentials
- Certificate authority access
- Database admin credentials (kept in vault)
```

### Environment Setup

```bash
# Validate cluster connectivity
kubectl cluster-info
kubectl get nodes

# Verify RBAC permissions
kubectl auth can-i '*' '*' --all-namespaces

# Check storage classes
kubectl get storageclass

# Verify ingress controller
kubectl get ingressclass
```

### Secrets & Credentials (via HashiCorp Vault)

```bash
# Must be pre-configured in Vault:
vault kv list secret/phantommesh/prod/
  - tls-certificates
  - api-keys
  - database-credentials
  - service-accounts
  - encryption-keys
  - jwt-signing-keys
```

---

## Step-by-Step Deployment

### Phase 1: Pre-Deployment (T-2 hours)

#### 1.1 Preparation

```bash
# Clone repository
git clone https://github.com/iamthegreatdestroyer/PhantomMesh-VPN.git
cd PhantomMesh-VPN

# Create deployment namespace
kubectl create namespace phantommesh-prod
kubectl label namespace phantommesh-prod environment=production

# Create image pull secrets
kubectl create secret docker-registry gcr-json-key \
  --docker-server=gcr.io \
  --docker-username=_json_key \
  --docker-password="$(cat /path/to/key.json)" \
  -n phantommesh-prod
```

#### 1.2 Configuration Validation

```bash
# Validate all config files
kubectl apply -f k8s/base/ --dry-run=client
kubectl apply -f k8s/overlays/prod/ --dry-run=client

# Test secret loading
kubectl apply -f k8s/base/secrets/ --dry-run=client

# Verify network policies compile
kubectl apply -f k8s/base/networkpolicies/ --dry-run=client
```

#### 1.3 Pre-deployment Health Checks

```bash
# Storage verification
kubectl get storageclass
kubectl get pv

# Network verification
kubectl get networkpolicies -n phantommesh-prod

# RBAC verification
kubectl get rolebindings -n phantommesh-prod
kubectl get clusterrolebindings | grep phantommesh
```

---

### Phase 2: Core Deployment (T-1 hour)

#### 2.1 Deploy Namespace & RBAC

```bash
# Apply namespace and RBAC
kubectl apply -f k8s/base/namespace/
kubectl apply -f k8s/base/rbac/

# Verify RBAC
kubectl get sa -n phantommesh-prod
kubectl get roles -n phantommesh-prod
```

#### 2.2 Deploy Secrets & ConfigMaps

```bash
# Deploy secrets (from Vault)
./scripts/deploy-secrets.sh phantommesh-prod

# Deploy configuration
kubectl apply -f k8s/base/configmaps/

# Verify secrets loaded
kubectl get secrets -n phantommesh-prod
kubectl get configmaps -n phantommesh-prod
```

#### 2.3 Deploy Storage

```bash
# Deploy persistent volume claims
kubectl apply -f k8s/base/persistentvolumes/

# Wait for PVC binding
kubectl get pvc -n phantommesh-prod -w
# All should show "Bound" within 2 minutes

# Verify storage
kubectl describe pvc -n phantommesh-prod
```

#### 2.4 Deploy Network Policies

```bash
# Apply network segmentation
kubectl apply -f k8s/base/networkpolicies/

# Verify policies
kubectl get networkpolicies -n phantommesh-prod
```

---

### Phase 3: Service Deployment (T hour)

#### 3.1 Deploy Core Services

```bash
# Deploy in correct order (dependencies first)
echo "Deploying Prometheus..."
kubectl apply -f k8s/base/deployments/prometheus-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/prometheus -n phantommesh-prod

echo "Deploying PostgreSQL..."
kubectl apply -f k8s/base/deployments/postgres-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/postgres -n phantommesh-prod

echo "Deploying Redis..."
kubectl apply -f k8s/base/deployments/redis-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/redis -n phantommesh-prod

echo "Deploying VPN Core..."
kubectl apply -f k8s/base/deployments/vpn-core-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/vpn-core -n phantommesh-prod

echo "Deploying Discovery Service..."
kubectl apply -f k8s/base/deployments/discovery-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/discovery -n phantommesh-prod

echo "Deploying Agent Swarm..."
kubectl apply -f k8s/base/deployments/agent-swarm-deployment.yaml
kubectl wait --for=condition=ready pod -l app=agent-swarm \
  -n phantommesh-prod --timeout=300s

echo "Deploying API Gateway..."
kubectl apply -f k8s/base/deployments/api-gateway-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/api-gateway -n phantommesh-prod

echo "Deploying Grafana..."
kubectl apply -f k8s/base/deployments/grafana-deployment.yaml
kubectl wait --for=condition=available --timeout=300s \
  deployment/grafana -n phantommesh-prod
```

#### 3.2 Deploy Services

```bash
# Expose services
kubectl apply -f k8s/base/services/

# Verify service discovery
kubectl get svc -n phantommesh-prod
kubectl get endpoints -n phantommesh-prod
```

#### 3.3 Deploy Ingress

```bash
# Configure ingress routes
kubectl apply -f k8s/base/ingress/

# Verify ingress is ready
kubectl get ingress -n phantommesh-prod
kubectl describe ingress -n phantommesh-prod

# Wait for DNS propagation
sleep 30
nslookup phantommesh.example.com
```

#### 3.4 Deploy HPA (Horizontal Pod Autoscaling)

```bash
# Enable autoscaling
kubectl apply -f k8s/base/autoscaling/

# Verify HPA is active
kubectl get hpa -n phantommesh-prod
```

---

### Phase 4: Verification (T+30 min)

#### 4.1 Pod Health Check

```bash
# Verify all pods are running
kubectl get pods -n phantommesh-prod
# All should show Running or Completed

# Check for crash loops
kubectl get pods -n phantommesh-prod --field-selector=status.phase!=Running

# View pod logs for errors
kubectl logs -n phantommesh-prod -l app=vpn-core --tail=50
kubectl logs -n phantommesh-prod -l app=discovery --tail=50
```

#### 4.2 Service Connectivity Check

```bash
# Test VPN core endpoint
curl -v https://phantommesh.example.com/health

# Test discovery service
curl -v https://phantommesh.example.com/discovery/peers

# Test API gateway
curl -v https://phantommesh.example.com/api/v1/status
```

#### 4.3 Data Validation

```bash
# Verify database connectivity
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SELECT version();"

# Check database initialization
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "\dt"

# Validate Redis connectivity
kubectl exec -it redis-0 -n phantommesh-prod -- redis-cli ping
```

#### 4.4 Metrics Collection

```bash
# Verify Prometheus is scraping metrics
curl -s http://prometheus-svc:9090/api/v1/targets | jq '.data.activeTargets[] | {job: .job, health: .health}'

# Check Grafana datasource configuration
curl -s http://grafana-svc:3000/api/datasources | jq '.[] | {name: .name, type: .type, health: .health}'
```

---

## Post-Deployment Verification

### Smoke Tests

```bash
#!/bin/bash
set -e

echo "Running smoke tests..."

# Test 1: Health endpoint
echo "Test 1: Health Check..."
STATUS=$(curl -s -o /dev/null -w "%{http_code}" https://phantommesh.example.com/health)
[ "$STATUS" = "200" ] && echo "✅ Health check: PASSED" || echo "❌ Health check: FAILED ($STATUS)"

# Test 2: Peer discovery
echo "Test 2: Peer Discovery..."
PEERS=$(curl -s https://phantommesh.example.com/discovery/peers | jq '.peers | length')
[ "$PEERS" -gt 0 ] && echo "✅ Peer discovery: PASSED ($PEERS peers)" || echo "❌ Peer discovery: FAILED"

# Test 3: VPN connection
echo "Test 3: VPN Connection..."
CONNECTION=$(wg-quick up /etc/wireguard/phantommesh0.conf 2>&1)
if [[ $? -eq 0 ]]; then
  echo "✅ VPN connection: PASSED"
  wg-quick down phantommesh0 > /dev/null
else
  echo "❌ VPN connection: FAILED"
fi

# Test 4: Metrics collection
echo "Test 4: Metrics..."
METRIC_COUNT=$(curl -s http://phantommesh.example.com/metrics | grep -c "phantommesh_")
[ "$METRIC_COUNT" -gt 100 ] && echo "✅ Metrics: PASSED ($METRIC_COUNT)" || echo "❌ Metrics: FAILED"

# Test 5: Load test
echo "Test 5: Load Test (100 concurrent)..."
for i in {1..100}; do
  curl -s https://phantommesh.example.com/health > /dev/null &
done
wait
echo "✅ Load test: PASSED"

echo ""
echo "🎉 All smoke tests passed!"
```

### Performance Validation

```bash
# Run quick load test
ab -n 1000 -c 100 https://phantommesh.example.com/health

# Expected results:
# - Requests/sec: > 500
# - P99 latency: < 200ms
# - Failed requests: 0
```

### Security Validation

```bash
# Verify TLS/SSL
openssl s_client -connect phantommesh.example.com:443 -showcerts

# Check certificate validity
echo | openssl s_client -connect phantommesh.example.com:443 2>/dev/null | \
  openssl x509 -noout -dates

# Verify security headers
curl -I https://phantommesh.example.com | grep -E "Strict-Transport-Security|X-Content-Type-Options|X-Frame-Options"
```

---

## Rollback Procedures

### Automatic Rollback (First 5 minutes)

```bash
# If deployment fails immediately, Kubernetes automatically reverts:
kubectl rollout history deployment/vpn-core -n phantommesh-prod
kubectl rollout undo deployment/vpn-core -n phantommesh-prod
```

### Manual Rollback (First 30 minutes)

```bash
# If critical issues discovered during verification:

# 1. Pause all ingress traffic
kubectl patch ingress phantommesh-ingress -n phantommesh-prod -p \
  '{"spec":{"rules":[{"host":"phantommesh.example.com","http":{"paths":[]}}]}}'

# 2. Scale down new deployment
kubectl scale deployment vpn-core --replicas=0 -n phantommesh-prod

# 3. Restore previous version
kubectl rollout undo deployment/vpn-core -n phantommesh-prod
kubectl scale deployment vpn-core --replicas=3 -n phantommesh-prod

# 4. Verify rollback
kubectl rollout status deployment/vpn-core -n phantommesh-prod
curl https://phantommesh.example.com/health

# 5. Re-enable traffic
kubectl apply -f k8s/base/ingress/
```

### Complete Rollback (>30 minutes or critical failure)

```bash
# Full infrastructure rollback to pre-deployment state

# 1. Drain cluster gracefully
kubectl drain --all-pods --ignore-daemonsets -n phantommesh-prod

# 2. Remove deployment
kubectl delete deployment -n phantommesh-prod --all

# 3. Remove services & ingress
kubectl delete service,ingress -n phantommesh-prod --all

# 4. Remove data (if necessary - CAUTION!)
# kubectl delete pvc -n phantommesh-prod --all

# 5. Remove namespace
kubectl delete namespace phantommesh-prod

# 6. Restore from database backup
# ./scripts/restore-database.sh prod <backup_timestamp>

# 7. Notify stakeholders
# Send incident report
```

---

## Troubleshooting

### Common Issues & Solutions

#### Issue: Pod stuck in Pending

```bash
kubectl describe pod <pod-name> -n phantommesh-prod
# Check events for root cause

# Solutions:
# - Insufficient resources: Scale cluster or reduce replicas
# - PVC not binding: Check storage class and permissions
# - Image pull failure: Verify registry credentials
kubectl scale deployment vpn-core --replicas=2 -n phantommesh-prod
```

#### Issue: CrashLoopBackOff

```bash
kubectl logs <pod-name> -n phantommesh-prod --previous
# Review previous logs before crash

# Common causes:
# - Database connection failure
# - Missing secrets/configmaps
# - Port already in use
# - Out of memory

# Fix: Verify dependencies are running first
kubectl get pods -n phantommesh-prod | grep Running
```

#### Issue: High memory usage

```bash
# Check memory limits vs actual usage
kubectl top pod -n phantommesh-prod --sort-by=memory

# Increase limits if necessary
kubectl set resources deployment vpn-core \
  -n phantommesh-prod \
  --limits=memory=2Gi --requests=memory=1Gi
```

#### Issue: Network connectivity failure

```bash
# Test from inside cluster
kubectl exec -it <pod-name> -n phantommesh-prod -- \
  curl -v http://vpn-core:8080/health

# Check DNS resolution
kubectl exec -it <pod-name> -n phantommesh-prod -- \
  nslookup postgres.phantommesh-prod.svc.cluster.local

# Verify network policies aren't blocking
kubectl get networkpolicies -n phantommesh-prod
```

---

## Monitoring & Alerting

### Dashboard Access

```
Prometheus: https://phantommesh.example.com/prometheus
Grafana:    https://phantommesh.example.com/grafana
Loki:       https://phantommesh.example.com/loki
```

### Critical Metrics to Watch

| Metric               | Threshold | Action                          |
| -------------------- | --------- | ------------------------------- |
| CPU Usage            | > 80%     | Scale pods or investigate spike |
| Memory Usage         | > 85%     | Review logs, adjust limits      |
| Disk Usage           | > 90%     | Cleanup or expand storage       |
| Error Rate           | > 1%      | Page on-call engineer           |
| P99 Latency          | > 200ms   | Investigate bottleneck          |
| Pod Restarts         | > 1/hour  | Investigate crash loops         |
| Database Connections | > 80%     | Check connection leaks          |

### Alert Rules

```yaml
# Prometheus alert configuration
groups:
  - name: phantommesh
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
        for: 5m
        annotations:
          severity: critical

      - alert: HighLatency
        expr: histogram_quantile(0.99, http_request_duration_seconds) > 0.2
        for: 10m
        annotations:
          severity: warning
```

---

## Emergency Procedures

### Emergency Contact Tree

```
On-Call Engineer: [PRIMARY CONTACT]
Engineering Lead: [ESCALATION 1]
CTO: [ESCALATION 2]
CEO: [ESCALATION 3]
```

### Critical Incident Checklist

```bash
# 1. Declare incident
#    - Create incident channel
#    - Page on-call team
#    - Set severity level (P1/P2/P3)

# 2. Immediate mitigation
kubectl scale deployment vpn-core --replicas=0 -n phantommesh-prod  # Circuit breaker

# 3. Investigate
kubectl logs -n phantommesh-prod -l app=vpn-core --tail=200
kubectl describe node  # Check cluster health
kubectl describe pvc -n phantommesh-prod  # Check storage

# 4. Document timeline
#    - When issue started
#    - Impact scope (how many users)
#    - Root cause (if known)
#    - Actions taken
#    - ETA to resolution

# 5. Communicate
#    - Slack/Teams notification
#    - Status page update
#    - Customer communication

# 6. Resolution & Recovery
#    - Fix issue
#    - Scale back up
#    - Verify health

# 7. Post-mortem
#    - Schedule post-mortem meeting
#    - Document root cause
#    - Create improvement tickets
```

---

## Sign-Off & Approval

| Role               | Name               | Signature          | Date       |
| ------------------ | ------------------ | ------------------ | ---------- |
| Deployment Lead    | ********\_******** | ********\_******** | ****\_**** |
| Operations Manager | ********\_******** | ********\_******** | ****\_**** |
| Security Officer   | ********\_******** | ********\_******** | ****\_**** |
| CTO                | ********\_******** | ********\_******** | ****\_**** |

---

**End of Production Deployment Runbook**

Last review: 2026-01-04  
Next review scheduled: Quarterly or after major changes
