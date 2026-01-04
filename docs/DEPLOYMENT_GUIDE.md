# PhantomMesh VPN Production Deployment Guide

## Phase P1-006: Production Deployment & Load Testing

**Status:** IN PROGRESS  
**Last Updated:** January 3, 2026  
**Version:** 1.0.0

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Pre-Deployment Checklist](#pre-deployment-checklist)
3. [Deployment Steps](#deployment-steps)
4. [Post-Deployment Validation](#post-deployment-validation)
5. [Monitoring & Alerting Setup](#monitoring--alerting-setup)
6. [Load Testing](#load-testing)
7. [Scaling Configuration](#scaling-configuration)
8. [Disaster Recovery](#disaster-recovery)
9. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Hardware Requirements

- Kubernetes cluster (v1.25+) with 3+ nodes
- Minimum node spec: 2 CPU, 4GB RAM, 50GB storage
- Recommended: 4 CPU, 8GB RAM, 100GB storage per node
- Production: 8 CPU, 16GB RAM, 200GB storage per node

### Software Requirements

- kubectl v1.25+
- Helm 3.10+
- Docker 20.10+
- Python 3.9+ (for load testing)

### Network Requirements

- Cluster networking (CNI plugin installed)
- Ingress controller deployed
- Service mesh (optional but recommended: Istio/Linkerd)
- External DNS or static IP for ingress

### Cloud Provider (if applicable)

- VPC with proper subnetting
- Security groups configured
- Load balancers available
- Persistent volume storage available

### Credentials & Access

- Kubernetes cluster admin access
- Container registry credentials
- Database credentials ready
- API keys for external services

---

## Pre-Deployment Checklist

### Cluster Preparation

- [ ] Kubernetes cluster running and accessible
- [ ] kubectl configured and tested
- [ ] Helm repository added: `helm repo add phantom-mesh https://...`
- [ ] Default storage class configured
- [ ] CNI plugin installed and verified
- [ ] Metrics server installed (for HPA)
- [ ] Ingress controller deployed

### Configuration Validation

- [ ] All config files reviewed and validated
- [ ] Production secrets created
- [ ] TLS certificates ready
- [ ] Database credentials secured
- [ ] API keys stored securely
- [ ] RBAC roles defined
- [ ] Network policies reviewed

### Resource Planning

- [ ] Node capacity calculated
- [ ] Storage requirements verified
- [ ] Network bandwidth assessed
- [ ] Cost estimation completed
- [ ] Backup strategy defined
- [ ] Disaster recovery tested

### Team Preparation

- [ ] Team trained on K8s operations
- [ ] On-call rotation established
- [ ] Runbooks created
- [ ] Escalation path defined
- [ ] Communication channels setup

---

## Deployment Steps

### Step 1: Create Namespace & Secrets

```bash
# Create production namespace
kubectl create namespace phantom-mesh
kubectl label namespace phantom-mesh environment=production

# Create TLS certificates secret
kubectl create secret tls phantom-tls \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  -n phantom-mesh

# Create database credentials
kubectl create secret generic phantom-db-secret \
  --from-literal=username=phantom_prod_user \
  --from-literal=password=$(openssl rand -base64 32) \
  -n phantom-mesh

# Create Redis credentials
kubectl create secret generic phantom-redis-secret \
  --from-literal=password=$(openssl rand -base64 32) \
  -n phantom-mesh

# Verify secrets created
kubectl get secrets -n phantom-mesh
```

### Step 2: Deploy Using Kustomize

```bash
# Navigate to overlay directory
cd k8s/overlays/prod

# Validate manifests
kubectl kustomize . --enable-alpha-plugins | less

# Apply production deployment
kubectl apply -k .

# Watch deployment progress
kubectl rollout status deployment/phantom-automation -n phantom-mesh --timeout=5m
kubectl rollout status deployment/phantom-vpn-core -n phantom-mesh --timeout=5m
kubectl rollout status deployment/phantom-discovery -n phantom-mesh --timeout=5m
```

### Step 3: Verify Pod Deployment

```bash
# Check pod status
kubectl get pods -n phantom-mesh -w

# Expected output: 9 pods running
# - 3 phantom-automation
# - 3 phantom-vpn-core
# - 2 phantom-discovery
# - 1 prometheus
# - 1 grafana
# (adjust based on actual deployment)

# View pod details
kubectl describe pod <pod-name> -n phantom-mesh

# Check logs
kubectl logs deployment/phantom-automation -n phantom-mesh -f
```

### Step 4: Configure Ingress & Networking

```bash
# Apply ingress configuration
kubectl apply -f k8s/overlays/prod/ingress-prod.yaml

# Verify ingress created
kubectl get ingress -n phantom-mesh

# Get external IP/hostname
kubectl get ingress phantom-ingress -n phantom-mesh -o jsonpath='{.status.loadBalancer.ingress[0]}'

# Update DNS records to point to ingress
# (Implementation specific to your DNS provider)
```

### Step 5: Setup Monitoring & Alerting

```bash
# Deploy Prometheus
kubectl apply -f k8s/overlays/prod/config/prometheus-prod.yaml

# Deploy Alert Manager
kubectl apply -f k8s/overlays/prod/config/alertmanager-prod.yaml

# Deploy Grafana
kubectl apply -f k8s/overlays/prod/config/grafana-dashboards.yaml

# Verify monitoring stack
kubectl get pods -n phantom-mesh | grep -E "prometheus|grafana|alertmanager"

# Access Prometheus
kubectl port-forward svc/prometheus 9090:9090 -n phantom-mesh
# Open http://localhost:9090

# Access Grafana
kubectl port-forward svc/grafana 3000:3000 -n phantom-mesh
# Open http://localhost:3000 (default: admin/admin)
```

### Step 6: Configure Auto-Scaling

```bash
# Verify HPA created
kubectl get hpa -n phantom-mesh

# Check HPA status
kubectl describe hpa phantom-automation-hpa -n phantom-mesh

# Monitor scaling events
kubectl get events -n phantom-mesh --sort-by='.lastTimestamp'
```

---

## Post-Deployment Validation

### Health Checks

```bash
# Check all pods are running
kubectl get pods -n phantom-mesh --field-selector=status.phase=Running | wc -l
# Should show 9 pods (or your configured number)

# Check pod restarts
kubectl get pods -n phantom-mesh -o jsonpath='{.items[*].status.containerStatuses[*].restartCount}' | tr ' ' '\n' | sort | uniq -c
# Should show "9      0" (no restarts)

# Check container logs for errors
for pod in $(kubectl get pods -n phantom-mesh -o name); do
  echo "=== $pod ==="
  kubectl logs $pod -n phantom-mesh --tail=10
done
```

### API Health Checks

```bash
# Port forward to automation service
kubectl port-forward svc/phantom-automation 8080:8080 -n phantom-mesh

# Check health endpoints
curl http://localhost:8080/health
# Expected: {"status": "healthy"}

curl http://localhost:8080/ready
# Expected: {"ready": true}

# Test threat processing endpoint
curl -X POST http://localhost:8080/api/v1/automation/process-threat \
  -H "Content-Type: application/json" \
  -d '{
    "threat_signal": {
      "protocol": "tcp",
      "port": 8080,
      "confidence": 0.85
    }
  }'
```

### Database Connectivity

```bash
# Test database connection
kubectl exec -it deployment/phantom-automation -n phantom-mesh -- \
  python -c "
import psycopg2
conn = psycopg2.connect(
    host='phantom-postgres.phantom-mesh.svc.cluster.local',
    database='phantom_mesh_prod',
    user='phantom_prod_user',
    password='<password>'
)
print('Database connection successful')
conn.close()
"
```

### Metrics Collection

```bash
# Verify Prometheus is scraping targets
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets | length'
# Should show > 10 targets

# Check metric availability
curl http://localhost:9090/api/v1/query?query=up
```

---

## Monitoring & Alerting Setup

### Access Monitoring Stack

```bash
# Prometheus
kubectl port-forward -n phantom-mesh svc/prometheus 9090:9090 &
# http://localhost:9090

# Grafana
kubectl port-forward -n phantom-mesh svc/grafana 3000:3000 &
# http://localhost:3000

# AlertManager
kubectl port-forward -n phantom-mesh svc/alertmanager 9093:9093 &
# http://localhost:9093
```

### Configure Alerting Channels

**Slack:**

```bash
# Create Slack webhook in Slack app config
# Update alertmanager-prod.yaml with webhook URL
kubectl set env deployment/alertmanager \
  SLACK_WEBHOOK_URL="https://hooks.slack.com/..." \
  -n phantom-mesh
```

**PagerDuty:**

```bash
# Configure in alertmanager-prod.yaml
# Add integration key from PagerDuty
kubectl set env deployment/alertmanager \
  PAGERDUTY_KEY="..." \
  -n phantom-mesh
```

**Email:**

```bash
# Configure SMTP in alertmanager-prod.yaml
kubectl set env deployment/alertmanager \
  SMTP_HOST="smtp.example.com" \
  SMTP_PORT="587" \
  SMTP_USER="alerts@example.com" \
  SMTP_PASSWORD="..." \
  -n phantom-mesh
```

### Import Grafana Dashboards

```bash
# Dashboard files are in k8s/overlays/prod/dashboards/

# Import via UI:
# 1. Grafana → Dashboards → New → Import
# 2. Upload JSON file
# 3. Select datasource (Prometheus)
# 4. Save

# Or via API:
curl -X POST http://admin:admin@localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @phantommesh-dashboard.json
```

---

## Load Testing

### Prepare Load Testing Environment

```bash
# Install load testing tools
pip install locust requests

# Configure load test parameters
cat > load_test_config.yaml << EOF
target_url: "http://phantom-automation:8080"
test_duration_seconds: 300
ramp_up_seconds: 60
initial_rate: 100
peak_rate: 1000
spike_rate: 2000
spike_duration: 60
EOF
```

### Run Load Tests

```bash
# Test 1: Ramp-up (0 → 1000 req/s over 60s)
python tests/load/load_test_runner.py --test ramp-up

# Test 2: Sustained load (1000 req/s for 5min)
python tests/load/load_test_runner.py --test sustained

# Test 3: Spike test (sudden 2x increase)
python tests/load/load_test_runner.py --test spike

# Test 4: 24-hour soak test
python tests/load/load_test_runner.py --test soak --duration 86400

# Run full test suite
python tests/load/load_test_runner.py --suite full
```

### Monitor During Load Test

```bash
# Terminal 1: Run load test
python tests/load/load_test_runner.py --suite full

# Terminal 2: Monitor pods
watch kubectl top pods -n phantom-mesh

# Terminal 3: Watch Prometheus
curl "http://localhost:9090/api/v1/query?query=container_cpu_usage_seconds_total" | jq .

# Terminal 4: View logs
kubectl logs -f deployment/phantom-automation -n phantom-mesh
```

### Analyze Load Test Results

```bash
# Review load test report
cat load_test_results_*.json | jq '.[]'

# Check latency percentiles
cat load_test_results_*.json | jq '.latency_stats'

# Verify throughput
cat load_test_results_*.json | jq '.throughput'

# Check error rates
cat load_test_results_*.json | jq '.error_rate'
```

---

## Scaling Configuration

### Manual Scaling

```bash
# Scale automation pods
kubectl scale deployment phantom-automation --replicas=5 -n phantom-mesh

# Scale VPN core pods
kubectl scale deployment phantom-vpn-core --replicas=5 -n phantom-mesh

# Verify scaling
kubectl get hpa -n phantom-mesh
```

### Auto-Scaling Configuration

```bash
# Current HPA settings are in k8s/overlays/prod/resources/hpa-*.yaml

# To adjust auto-scaling thresholds:
kubectl patch hpa phantom-automation-hpa -p '{"spec":{"maxReplicas":20}}' -n phantom-mesh

# Monitor auto-scaling decisions
kubectl describe hpa phantom-automation-hpa -n phantom-mesh
kubectl get events -n phantom-mesh --sort-by='.lastTimestamp' | grep -i scale
```

---

## Disaster Recovery

### Backup Procedures

```bash
# Backup database
kubectl exec -it deployment/postgres -n phantom-mesh -- \
  pg_dump -U phantom_prod_user phantom_mesh_prod > backup.sql

# Backup persistent volumes
for pvc in $(kubectl get pvc -n phantom-mesh -o name); do
  kubectl get $pvc -n phantom-mesh -o yaml > ${pvc##*/}.yaml
done

# Upload backups to S3
aws s3 cp backup.sql s3://phantom-mesh-backups/$(date +%Y%m%d_%H%M%S)_backup.sql
```

### Recovery Procedures

```bash
# Restore database from backup
kubectl cp backup.sql deployment/postgres:/tmp/backup.sql -n phantom-mesh
kubectl exec -it deployment/postgres -n phantom-mesh -- \
  psql -U phantom_prod_user phantom_mesh_prod < /tmp/backup.sql

# Verify restoration
kubectl exec -it deployment/postgres -n phantom-mesh -- \
  psql -U phantom_prod_user -d phantom_mesh_prod -c "SELECT COUNT(*) FROM incidents;"
```

### Pod Recovery

```bash
# If pod fails, Kubernetes will auto-restart
# Monitor restart counts
kubectl get pods -n phantom-mesh --watch

# If deployment is stuck, trigger rollout restart
kubectl rollout restart deployment/phantom-automation -n phantom-mesh

# Rollback to previous version if needed
kubectl rollout history deployment/phantom-automation -n phantom-mesh
kubectl rollout undo deployment/phantom-automation -n phantom-mesh
```

---

## Troubleshooting

### Pod Stuck in Pending

```bash
# Check pod status
kubectl describe pod <pod-name> -n phantom-mesh

# Common issues:
# 1. Insufficient resources
kubectl top nodes  # Check node capacity

# 2. PVC not bound
kubectl get pvc -n phantom-mesh

# 3. Node selector issue
kubectl get nodes --show-labels
```

### High Latency

```bash
# Check pod resource usage
kubectl top pod <pod-name> -n phantom-mesh

# Check node resource usage
kubectl top node

# Check database connection pool
kubectl exec -it deployment/phantom-automation -n phantom-mesh -- \
  python -c "import psycopg2; conn = psycopg2.pool.SimpleConnectionPool(...)"
```

### Memory Leak

```bash
# Check memory trends over time
kubectl top pod <pod-name> -n phantom-mesh --containers

# Get detailed memory stats from Prometheus
curl "http://localhost:9090/api/v1/query?query=container_memory_usage_bytes{pod_name='phantom-automation'}"

# If memory leak confirmed, restart pod
kubectl delete pod <pod-name> -n phantom-mesh
```

### Database Connection Issues

```bash
# Test connectivity
kubectl run -it --image=psycopg2 test-db -n phantom-mesh -- \
  python -c "import psycopg2; psycopg2.connect(dbname='phantom_mesh_prod', user='phantom_prod_user')"

# Check connection pool status
kubectl exec -it deployment/phantom-automation -n phantom-mesh -- \
  python -c "
import psycopg2.pool
# Check active connections
"
```

### Metrics Not Appearing

```bash
# Verify Prometheus targets are scraping
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets'

# Check target details
curl http://localhost:9090/api/v1/targets?state=any

# Force Prometheus reload
kubectl exec -it prometheus-0 -n phantom-mesh -- \
  kill -HUP 1
```

---

## Success Criteria

✅ **Deployment Complete When:**

- [ ] All 9 pods running without errors
- [ ] Health endpoints returning healthy status
- [ ] Database connectivity verified
- [ ] Metrics being collected in Prometheus
- [ ] Grafana dashboards displaying data
- [ ] HPA responding to load changes
- [ ] Load tests passing (1000+ req/s sustained)
- [ ] <200ms p95 latency maintained
- [ ] <0.1% error rate
- [ ] Team trained and on-call ready

---

## Next Steps

1. **Week 1:** Complete deployment and validation
2. **Week 2:** Extended load testing and optimization
3. **Week 3:** Performance tuning and hardening
4. **Week 4:** Production readiness sign-off

---

**Document Version:** 1.0  
**Last Updated:** January 3, 2026  
**Owner:** APEX Platform Team
