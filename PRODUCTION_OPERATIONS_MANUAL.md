# 📖 PhantomMesh VPN - Production Operations Manual

**Version:** 1.0  
**Status:** READY FOR PRODUCTION  
**Audience:** Operations, DevOps, SRE Teams  
**Created:** 2026-01-04

---

## Table of Contents

1. [Daily Operations](#daily-operations)
2. [Monitoring & Alerts](#monitoring--alerts)
3. [Common Procedures](#common-procedures)
4. [Troubleshooting Guide](#troubleshooting-guide)
5. [Performance Optimization](#performance-optimization)
6. [Maintenance Windows](#maintenance-windows)
7. [Quick Reference](#quick-reference)

---

## Daily Operations

### Morning Checklist (Start of Shift)

**Time: 8:00 AM Daily**

```bash
#!/bin/bash
# Daily operations checklist

echo "=== PHANTOMMESH DAILY OPERATIONS CHECKLIST ==="
echo "$(date)"

# 1. Cluster health
echo "[1] Checking Kubernetes cluster health..."
kubectl get nodes
kubectl get componentstatuses
if [ $? -ne 0 ]; then
  echo "❌ CRITICAL: Cluster issues detected"
  # Escalate immediately
fi

# 2. Pod status
echo "[2] Checking pod status..."
kubectl get pods -n phantommesh-prod --all-namespaces
FAILED=$(kubectl get pods -n phantommesh-prod --field-selector=status.phase!=Running | wc -l)
if [ "$FAILED" -gt 1 ]; then
  echo "⚠️ WARNING: $FAILED pods not running"
fi

# 3. Resource usage
echo "[3] Checking resource utilization..."
kubectl top nodes
kubectl top pods -n phantommesh-prod --sort-by=memory

# 4. Storage status
echo "[4] Checking storage..."
kubectl get pvc -n phantommesh-prod
kubectl get pv

# 5. Database health
echo "[5] Checking database..."
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SELECT version();"

# 6. Recent errors
echo "[6] Checking for recent errors..."
kubectl logs -n phantommesh-prod -l app=vpn-core --tail=20 | grep -i error

# 7. Certificate expiry
echo "[7] Checking TLS certificates..."
echo | openssl s_client -connect phantommesh.example.com:443 2>/dev/null | \
  openssl x509 -noout -dates

# 8. Backup status
echo "[8] Verifying last backup..."
aws s3 ls s3://phantommesh-backups/daily/ --recursive | tail -1

echo ""
echo "✅ Daily checklist complete - Ready for operations"
```

### Metrics Review (Every 4 hours)

```
Access Dashboards:
  - Prometheus:  http://localhost:24540
  - Grafana:     http://localhost:24541
  - Loki:        http://localhost:24550

Check These Metrics:
  1. HTTP Error Rate (should be < 0.1%)
  2. API Response Latency P99 (should be < 200ms)
  3. Database Query Latency (should be < 50ms)
  4. Memory Usage (should be < 85%)
  5. CPU Usage (should be < 80%)
  6. Pod Restart Count (should be 0)
  7. Connection Pool Usage (should be < 80%)
  8. Disk Space (should be < 85% used)

Alert Thresholds:
  🟢 Green   = Healthy
  🟡 Yellow  = Degraded, investigate
  🔴 Red     = Critical, page on-call
```

---

## Monitoring & Alerts

### Alert Categories

#### 🔴 Critical (P1) - Page Immediately

```
Alerts that page on-call within 2 minutes:

1. Service Down
   - Any core service pod all replicas down
   - Action: Scale or restart immediately
   - Runbook: See "Service Recovery" section

2. Database Unreachable
   - Database connection failure
   - Action: Check network, verify credentials
   - Runbook: See "Database Troubleshooting"

3. Data Corruption Detected
   - Data integrity validation failure
   - Action: Prepare for rollback/recovery
   - Runbook: See "Data Recovery"

4. Security Incident
   - Unauthorized access detected
   - DDoS attack detected
   - Malware signature matched
   - Action: Isolate affected systems
   - Runbook: See "Security Incident Response"

5. Certificate Expiry (< 7 days)
   - TLS certificate about to expire
   - Action: Rotate certificate immediately
   - Runbook: See "Certificate Rotation"
```

#### 🟡 Warning (P2) - Investigate Within 1 Hour

```
Alerts that require timely investigation:

1. High Error Rate (> 1%)
   - Check recent deployments
   - Review logs for patterns
   - Identify affected users
   - Action: Fix or rollback

2. High Latency (P99 > 200ms)
   - Check resource usage
   - Identify slow queries
   - Analyze traffic patterns
   - Action: Optimize or scale

3. Memory Pressure (> 85%)
   - Check for memory leaks
   - Review pod metrics
   - Prepare to scale if needed
   - Action: Monitor + optimize

4. Disk Space Critical (> 90%)
   - Review log rotation
   - Check backup storage
   - Clean old data
   - Action: Free space or expand

5. Replica Lag (> 500ms)
   - Check network connectivity
   - Verify database performance
   - Monitor CPU/IO
   - Action: Investigate + monitor
```

#### 🟢 Info (P3) - Review During Shift

```
Informational alerts for awareness:

1. Pod Restart (< 5/hour)
   - Normal from updates
   - Monitor if increasing

2. Slightly High CPU (70-80%)
   - May indicate increased traffic
   - No immediate action needed

3. Backup Completion
   - Verification that backups succeeded
   - Check backup size for anomalies
```

### Setting Up Alerting

```yaml
# Prometheus AlertManager configuration
global:
  resolve_timeout: 5m

route:
  group_by: ["alertname", "cluster"]
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h

  routes:
    # Critical alerts - page immediately
    - match:
        severity: critical
      receiver: "pagerduty"
      continue: true

    # Warning alerts - send to ops channel
    - match:
        severity: warning
      receiver: "slack-ops"
      continue: true

    # Info - send to logging
    - match:
        severity: info
      receiver: "slack-info"

receivers:
  - name: "pagerduty"
    pagerduty_configs:
      - service_key: "${PAGERDUTY_SERVICE_KEY}"

  - name: "slack-ops"
    slack_configs:
      - api_url: "${SLACK_OPS_WEBHOOK}"
        channel: "#phantommesh-ops"

  - name: "slack-info"
    slack_configs:
      - api_url: "${SLACK_INFO_WEBHOOK}"
        channel: "#phantommesh-info"
```

---

## Common Procedures

### Scale Service Up/Down

```bash
# View current replicas
kubectl get deployment -n phantommesh-prod vpn-core

# Scale to specific number
kubectl scale deployment vpn-core --replicas=5 -n phantommesh-prod

# Wait for scaling to complete
kubectl rollout status deployment/vpn-core -n phantommesh-prod

# Verify new replicas are ready
kubectl get pods -n phantommesh-prod -l app=vpn-core
```

### Restart Service

```bash
# Graceful restart (with rolling update)
kubectl rollout restart deployment/vpn-core -n phantommesh-prod

# Wait for restart to complete
kubectl rollout status deployment/vpn-core -n phantommesh-prod

# Verify all pods restarted
kubectl get pods -n phantommesh-prod -l app=vpn-core -o wide
```

### View Service Logs

```bash
# Last 100 lines
kubectl logs deployment/vpn-core -n phantommesh-prod --tail=100

# Real-time streaming
kubectl logs -f deployment/vpn-core -n phantommesh-prod

# Last 1 hour
kubectl logs deployment/vpn-core -n phantommesh-prod \
  --since=1h --all-containers=true

# Multiple replicas (show which pod each line is from)
kubectl logs deployment/vpn-core -n phantommesh-prod \
  --all-containers=true -p
```

### Check Pod Health

```bash
# Get pod details
kubectl describe pod <pod-name> -n phantommesh-prod

# Check resource usage
kubectl top pod <pod-name> -n phantommesh-prod

# Access pod shell
kubectl exec -it <pod-name> -n phantommesh-prod -- /bin/bash

# View pod events (why it restarted, etc.)
kubectl get events -n phantommesh-prod --sort-by='.lastTimestamp'
```

### Update Configuration

```bash
# Edit ConfigMap
kubectl edit configmap phantom-mesh-config -n phantommesh-prod

# Verify changes
kubectl get configmap phantom-mesh-config -n phantommesh-prod -o yaml

# Restart pods to pick up new config (rolling restart)
kubectl rollout restart deployment/vpn-core -n phantommesh-prod

# Alternative: Set environment variable
kubectl set env deployment/vpn-core \
  SOME_VAR=value \
  -n phantommesh-prod
```

### Create Database Backup

```bash
#!/bin/bash
# Manual backup creation

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="phantommesh_${TIMESTAMP}.dump"

# Create backup
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  pg_dump -U phantommesh -Fc -v phantommesh > $BACKUP_FILE

# Verify backup
echo "Backup created: $BACKUP_FILE"
ls -lh $BACKUP_FILE

# Upload to S3
aws s3 cp $BACKUP_FILE s3://phantommesh-backups/manual/ \
  --sse aws:kms --storage-class STANDARD_IA

# Cleanup local copy
rm $BACKUP_FILE

echo "Backup uploaded and verified"
```

### Restore Database

```bash
#!/bin/bash
# Restore from backup - CAUTION: This overwrites data!

# 1. Find backup file
aws s3 ls s3://phantommesh-backups/

# 2. Download backup
aws s3 cp s3://phantommesh-backups/phantommesh_20260104.dump .

# 3. STOP APPLICATIONS (to prevent data conflicts)
kubectl scale deployment vpn-core --replicas=0 -n phantommesh-prod
kubectl scale deployment api-gateway --replicas=0 -n phantommesh-prod

# 4. Drop existing data (WARNING: Data loss!)
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "DROP DATABASE phantommesh;"

# 5. Create new database
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  createdb -U phantommesh phantommesh

# 6. Restore data
kubectl exec -i postgres-0 -n phantommesh-prod -- \
  pg_restore -U phantommesh -d phantommesh < phantommesh_20260104.dump

# 7. Verify restoration
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SELECT COUNT(*) FROM users;"

# 8. Restart applications
kubectl scale deployment vpn-core --replicas=3 -n phantommesh-prod
kubectl scale deployment api-gateway --replicas=3 -n phantommesh-prod

# 9. Monitor for issues
watch kubectl get pods -n phantommesh-prod
```

---

## Troubleshooting Guide

### Pod is Stuck in Pending

```bash
# 1. Describe the pod to see events
kubectl describe pod <pod-name> -n phantommesh-prod

# 2. Check for common causes:

# Insufficient resources
kubectl top nodes
kubectl get resourcequotas -n phantommesh-prod

# PVC not binding
kubectl get pvc -n phantommesh-prod
kubectl describe pvc <pvc-name> -n phantommesh-prod

# Image pull failure
kubectl get events -n phantommesh-prod | grep ImagePull

# Node not ready
kubectl get nodes -o wide

# 3. Solutions:
# Scale down other pods to free resources
kubectl scale deployment/non-critical-app --replicas=0

# Check storage class
kubectl get storageclass
kubectl describe storageclass standard

# Force pod recreation
kubectl delete pod <pod-name> -n phantommesh-prod
```

### Pod is Restarting Constantly

```bash
# 1. Check logs before crash
kubectl logs <pod-name> -n phantommesh-prod --previous

# 2. Common causes:
# Out of memory
kubectl top pod <pod-name> -n phantommesh-prod
kubectl set resources deployment <app> --limits=memory=2Gi -n phantommesh-prod

# Port conflict
kubectl logs <pod-name> -n phantommesh-prod | grep -i "port\|address"
kubectl get pods -n phantommesh-prod -o wide | grep <node>

# Health check failure
kubectl describe pod <pod-name> -n phantommesh-prod | grep -i "livenessProbe\|readinessProbe"

# Database connection
kubectl logs <pod-name> -n phantommesh-prod | grep -i "connection\|database"
kubectl exec -it postgres-0 -n phantommesh-prod -- psql -c "SELECT 1;"

# 3. Fix options:
# Increase resources
kubectl set resources deployment vpn-core --limits=memory=2Gi -n phantommesh-prod

# Adjust health checks
kubectl edit deployment vpn-core -n phantommesh-prod

# Scale down then up
kubectl scale deployment vpn-core --replicas=0 -n phantommesh-prod
sleep 30
kubectl scale deployment vpn-core --replicas=3 -n phantommesh-prod
```

### High Memory Usage

```bash
# 1. Identify culprit
kubectl top pods -n phantommesh-prod --sort-by=memory

# 2. Check for memory leak
# Monitor over time - if memory continuously increases, leak is likely
watch "kubectl top pod <pod-name> -n phantommesh-prod"

# 3. View memory trends in Grafana
# Go to: http://localhost:24541
# Dashboard: "PhantomMesh Memory Usage"

# 4. Solutions:
# Increase memory limit (temporary)
kubectl set resources deployment vpn-core \
  --limits=memory=4Gi -n phantommesh-prod

# Restart pod (temporary)
kubectl rollout restart deployment/vpn-core -n phantommesh-prod

# Report bug (permanent)
# Create issue in GitHub with logs and metrics

# Scale horizontally
kubectl scale deployment vpn-core --replicas=5 -n phantommesh-prod
```

### High CPU Usage

```bash
# 1. Identify CPU consumer
kubectl top pods -n phantommesh-prod --sort-by=cpu

# 2. Check for busy loop
kubectl logs <pod-name> -n phantommesh-prod --tail=50

# 3. Profile if needed
# Install kubectl-debug
kubectl debug -n phantommesh-prod <pod-name> -it --image=nicolaka/netshoot

# 4. Solutions:
# Check if legitimate spike (high traffic)
# Check metrics: http://localhost:24540

# If sustained high CPU:
# Scale up (add more replicas)
kubectl scale deployment vpn-core --replicas=5 -n phantommesh-prod

# Or upgrade instance type (if resource constrained)
```

### API Returns 5xx Errors

```bash
# 1. Check API logs
kubectl logs deployment/api-gateway -n phantommesh-prod --tail=100

# 2. Check dependent services
kubectl get pods -n phantommesh-prod -l app=vpn-core
kubectl get pods -n phantommesh-prod -l app=postgres

# 3. Verify connectivity
kubectl exec -it <api-pod> -n phantommesh-prod -- \
  curl -v http://vpn-core:8080/health

# 4. Check database
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SELECT 1;"

# 5. Solutions:
# Restart API gateway
kubectl rollout restart deployment/api-gateway -n phantommesh-prod

# Scale up if overwhelmed
kubectl scale deployment/api-gateway --replicas=5 -n phantommesh-prod

# Check if recent code deployment caused issue
kubectl rollout history deployment/api-gateway -n phantommesh-prod
kubectl rollout undo deployment/api-gateway -n phantommesh-prod  # Rollback if needed
```

### Database Connection Issues

```bash
# 1. Verify database is running
kubectl get pods -n phantommesh-prod -l app=postgres

# 2. Check database logs
kubectl logs postgres-0 -n phantommesh-prod --tail=50

# 3. Test connectivity from pod
kubectl exec -it <app-pod> -n phantommesh-prod -- \
  psql -h postgres-svc -U phantommesh -c "SELECT 1;"

# 4. Check connection pool
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SELECT count(*) FROM pg_stat_activity;"

# 5. Solutions:
# Restart database
kubectl rollout restart statefulset/postgres -n phantommesh-prod

# Scale up if connection pool exhausted
# Adjust POSTGRES_MAX_CONNECTIONS in ConfigMap

# Check network connectivity
kubectl exec -it <pod> -n phantommesh-prod -- \
  nslookup postgres-svc.phantommesh-prod.svc.cluster.local
```

---

## Performance Optimization

### Query Optimization

```bash
# 1. Find slow queries in logs
kubectl logs postgres-0 -n phantommesh-prod | grep "duration:"

# 2. Enable query logging
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SET log_min_duration_statement = 1000;"

# 3. Analyze slow query
EXPLAIN ANALYZE SELECT * FROM users WHERE id = 12345;

# 4. Create index if needed
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);

# 5. Monitor impact
# Watch metrics in Grafana for latency improvement
```

### Connection Pool Tuning

```bash
# Current settings
kubectl get configmap phantom-mesh-config -n phantommesh-prod -o yaml | \
  grep -i "pool\|connection"

# Adjust connection pool size
kubectl set env deployment/api-gateway \
  DATABASE_POOL_SIZE=100 \
  DATABASE_MAX_OVERFLOW=20 \
  -n phantommesh-prod

# Monitor pool usage
kubectl exec -it postgres-0 -n phantommesh-prod -- \
  psql -U phantommesh -c "SELECT count(*) FROM pg_stat_activity;"
```

### Cache Optimization

```bash
# Check Redis memory usage
kubectl exec -it redis-0 -n phantommesh-prod -- redis-cli INFO memory

# Check hit rate
kubectl exec -it redis-0 -n phantommesh-prod -- \
  redis-cli INFO stats | grep "keyspace_hits\|keyspace_misses"

# Adjust TTL if needed
# Lower TTL = more hits but stale data
# Higher TTL = better performance but risks stale data
```

---

## Maintenance Windows

### Scheduling a Maintenance Window

```bash
# 1. Announce to users (1 week before)
# Update status page
# Send email notification
# Post in status channel

# 2. Schedule maintenance
DATE="2026-01-15 02:00 UTC"
DURATION="2 hours"

# 3. Day before: Final verification
# Test all procedures
# Verify rollback process
# Brief team

# 4. Maintenance window procedures (see Deployment Runbook)
# Follow standard update process
# Monitor closely
# Communicate status every 15 minutes

# 5. Post-maintenance
# Verify all systems operational
# Monitor for 24 hours
# Send completion notification
```

### Regular Maintenance Tasks

```
Weekly:
  - [ ] Review error logs
  - [ ] Check backup integrity
  - [ ] Update OS patches (non-disruptive)
  - [ ] Clear old logs
  - [ ] Validate disaster recovery procedure

Monthly:
  - [ ] Database optimization (ANALYZE, VACUUM)
  - [ ] Certificate expiry review
  - [ ] Dependency vulnerability scan
  - [ ] Capacity planning review
  - [ ] Document any changes

Quarterly:
  - [ ] Full security audit
  - [ ] Disaster recovery drill
  - [ ] Upgrade base images
  - [ ] Review and update runbooks
  - [ ] Team training refresh
```

---

## Quick Reference

### Essential Commands

```bash
# Cluster info
kubectl cluster-info
kubectl get nodes -o wide
kubectl top nodes

# Pods
kubectl get pods -n phantommesh-prod
kubectl describe pod <name> -n phantommesh-prod
kubectl logs pod <name> -n phantommesh-prod
kubectl exec -it pod/<name> -n phantommesh-prod -- bash

# Services
kubectl get svc -n phantommesh-prod
kubectl port-forward svc/prometheus 9090:9090 -n phantommesh-prod

# Deployments
kubectl rollout status deployment/<name> -n phantommesh-prod
kubectl rollout history deployment/<name> -n phantommesh-prod
kubectl rollout undo deployment/<name> -n phantommesh-prod

# Scale
kubectl scale deployment/<name> --replicas=3 -n phantommesh-prod

# Resources
kubectl top pods -n phantommesh-prod --sort-by=memory
kubectl top pods -n phantommesh-prod --sort-by=cpu

# Debugging
kubectl describe node <name>
kubectl get events -n phantommesh-prod --sort-by='.lastTimestamp'
kubectl logs -f <pod> -n phantommesh-prod --all-containers=true
```

### Important URLs

```
Prometheus:    http://prometheus-svc:9090       (metrics)
Grafana:       http://grafana-svc:3000          (dashboards)
Loki:          http://loki-svc:3100             (logs)
API:           https://phantommesh.example.com  (production)
Health Check:  https://phantommesh.example.com/health
Metrics:       https://phantommesh.example.com/metrics
```

### Emergency Contacts

```
On-Call Engineer:   [Primary contact]
Engineering Lead:   [Escalation 1]
CTO:               [Escalation 2]
CEO:               [Escalation 3]

Incident Channel:   #phantommesh-incidents (Slack)
Status Page:        https://status.phantommesh.example.com
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-04  
**Next Review:** Quarterly

_This manual should be kept up-to-date as procedures evolve._
