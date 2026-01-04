# 🚀 PHASE 4 EXECUTION RUNBOOK - PRODUCTION DEPLOYMENT

**Status:** Ready to Execute  
**Timeline:** Weeks 1-2 (This Week to Next Week)  
**Owner:** DevOps Lead + Security Engineer  
**Critical Path:** Week 1 = Audit & Staging, Week 2 = Deployment

---

## 📋 PHASE 4 OVERVIEW

**Objective:** Safely deploy PhantomMesh to production with zero critical security findings

**Success Criteria:**

- ✅ Security audit: 0 critical findings
- ✅ Staging validation: < 0.1% error rate
- ✅ Production deployment: < 5% cutover errors
- ✅ Post-deployment: < 10ms latency impact

---

## 🔐 WEEK 1: SECURITY AUDIT & STAGING SETUP

### Monday: Security Audit Execution

#### Task 1.1: Run kube-bench (CIS Kubernetes Benchmark)

**Time:** 30 minutes | **Owner:** DevOps Lead

```bash
# Install kube-bench
docker pull aquasec/kube-bench:latest

# Run comprehensive audit
docker run --pid host --userns host --cap-drop ALL \
  -v /etc:/etc:ro \
  -v /lib:/lib:ro \
  -v /sys:/sys:ro \
  -v /var/run:/var/run:ro \
  aquasec/kube-bench:latest \
  benchmark -j json > kube-bench-results.json

# Review results
cat kube-bench-results.json | jq '.Results[] | select(.Status=="FAIL")'

# ACTION: Document all FAIL items
# - CRITICAL (red): Must fix before deployment
# - WARNING (yellow): Fix within 2 weeks
```

**Expected Output:** JSON file with CIS benchmark results  
**Acceptance:** 0 CRITICAL issues, acceptable WARNING count

---

#### Task 1.2: Container Image Scanning (Trivy)

**Time:** 45 minutes | **Owner:** Security Engineer

```bash
# Install Trivy
wget https://github.com/aquasecurity/trivy/releases/download/v0.48.0/trivy_0.48.0_Linux-64bit.tar.gz
tar zxvf trivy_0.48.0_Linux-64bit.tar.gz

# Scan all container images
./trivy image --severity HIGH,CRITICAL \
  iamthegreatdestroyer/phantom-node:latest > phantom-node-scan.json

./trivy image --severity HIGH,CRITICAL \
  iamthegreatdestroyer/agent-swarm:latest > agent-swarm-scan.json

./trivy image --severity HIGH,CRITICAL \
  iamthegreatdestroyer/discovery:latest > discovery-scan.json

# Review critical vulnerabilities
cat phantom-node-scan.json | jq '.Results[] | select(.Severity=="CRITICAL")'

# ACTION: For each CRITICAL finding:
# - Update base image OR
# - Apply patch OR
# - Document exception with risk acceptance
```

**Expected Output:** JSON files with vulnerability data  
**Acceptance:** < 5 HIGH vulnerabilities, 0 CRITICAL vulnerabilities

---

#### Task 1.3: RBAC Audit

**Time:** 30 minutes | **Owner:** DevOps Lead

```bash
# Audit all service accounts
kubectl get sa -A -o json | jq '.items[] | {namespace: .metadata.namespace, name: .metadata.name}'

# Check cluster-admin assignments
kubectl get clusterrolebindings -o json | jq '.items[] | select(.roleRef.name=="cluster-admin")'

# Review namespace-level permissions
kubectl get rolebindings -A -o json | jq '.items[] | {namespace: .metadata.namespace, role: .roleRef.name}'

# ACTION: Document each role assignment
# - Verify principle of least privilege
# - Remove unnecessary admin access
# - Create focused service accounts per application
```

**Expected Output:** RBAC audit report  
**Acceptance:** No unexpected cluster-admin bindings, all roles justified

---

#### Task 1.4: Network Policy Validation

**Time:** 20 minutes | **Owner:** DevOps Lead

```bash
# Review network policies
kubectl get networkpolicies -A
kubectl get networkpolicies -A -o json > network-policies-backup.json

# Test: Can pods communicate as expected?
# This requires manual testing in staging

# ACTION: Verify policies
# - Default deny: YES
# - Explicit allows documented: YES
# - Egress to external APIs: Documented and limited
```

**Expected Output:** Network policy review  
**Acceptance:** All policies documented, tested

---

#### Task 1.5: Secrets Audit

**Time:** 30 minutes | **Owner:** Security Engineer

```bash
# Find all secrets
kubectl get secrets -A -o json | jq '.items[] | {name: .metadata.name, namespace: .metadata.namespace}'

# Identify hardcoded secrets (grep codebase)
grep -r "password\|secret\|token\|key" src/ --include="*.rs" --include="*.py" | grep -v "test\|example\|comment"

# ACTION: For each secret:
# - Ensure encrypted at rest: kubectl describe secret
# - Verify RBAC access: who can read this?
# - Plan rotation: kubectl replace secret
```

**Expected Output:** Secrets inventory and access matrix  
**Acceptance:** All secrets encrypted, access controlled, rotation planned

---

#### Task 1.6: Generate Security Audit Report

**Time:** 30 minutes | **Owner:** DevOps Lead

Create file: `docs/PHASE4_SECURITY_AUDIT_REPORT.md`

```markdown
# Security Audit Report - Phase 4 Pre-Deployment

Date: [TODAY]
Reviewer: [NAME]

## Summary

- CIS Benchmark: [PASS/FAIL] - [CRITICAL count] critical items
- Container Vulnerabilities: [CRITICAL count] critical, [HIGH count] high
- RBAC: [PASS/FAIL] - Least privilege verified
- Network Policies: [PASS/FAIL] - Default deny configured
- Secrets: [PASS/FAIL] - All encrypted, rotation planned

## Critical Findings (MUST FIX)

1. [Item]
2. [Item]

## Action Items (BEFORE DEPLOYMENT)

1. [ ] Fix critical finding #1
2. [ ] Fix critical finding #2
3. [ ] Verify fixes

## Approved By

- DevOps Lead: ******\_\_\_\_******
- Security Lead: ******\_\_\_\_******
- CTO: ******\_\_\_\_******
```

**Deliverable:** Signed-off security audit report  
**Success:** All critical findings resolved

---

### Tuesday-Wednesday: Staging Environment Setup

#### Task 2.1: Create Staging Namespace

**Time:** 1 hour | **Owner:** DevOps Lead

```bash
# Create isolated staging namespace
kubectl create namespace staging
kubectl label namespace staging environment=staging tier=non-production

# Apply network policy: staging cannot talk to production
cat << EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-staging-to-prod
  namespace: staging
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          environment: production
    ports:
    - protocol: TCP
      port: 443
  # Allow egress to external (approved) services
  - to:
    - ipBlock:
        cidr: 0.0.0.0/0
    ports:
    - protocol: TCP
      port: 53  # DNS
    - protocol: TCP
      port: 443 # HTTPS
EOF

kubectl get namespace staging --show-labels
```

**Acceptance:** Staging namespace created, isolated from production

---

#### Task 2.2: Deploy to Staging

**Time:** 2 hours | **Owner:** DevOps Lead

```bash
# Use Helm to deploy to staging
helm install phantommesh-staging ./k8s/helm/phantommesh \
  -n staging \
  --values k8s/overlays/staging/values.yaml \
  --set environment=staging \
  --set replicaCount=2

# Wait for rollout
kubectl rollout status deployment/phantommesh-staging -n staging --timeout=5m

# Verify pods are running
kubectl get pods -n staging
kubectl describe pod -n staging

# Check services
kubectl get svc -n staging
```

**Acceptance:** All pods running, services accessible

---

#### Task 2.3: Load Production Data to Staging

**Time:** 2 hours | **Owner:** Database Admin

```bash
# Backup production database
kubectl exec -n production [pod-name] -- \
  pg_dump -U postgres phantommesh > production-backup.sql

# Import to staging
kubectl exec -n staging [pod-name] -- \
  psql -U postgres < production-backup.sql

# Verify data integrity
kubectl exec -n staging [pod-name] -- \
  psql -U postgres -c "SELECT COUNT(*) FROM users;"

# Compare with production
kubectl exec -n production [pod-name] -- \
  psql -U postgres -c "SELECT COUNT(*) FROM users;"
# Should be identical
```

**Acceptance:** Data successfully replicated to staging

---

#### Task 2.4: Run Staging Load Test

**Time:** 4 hours (includes test execution) | **Owner:** SRE

```powershell
# Deploy load test harness to staging
kubectl apply -f k8s/tests/load_test_deployment.yaml -n staging

# Run ramp-up test (0→1000 req/sec)
# Monitor using: kubectl logs -n staging deployment/load-test -f

# Execute test
python tests/load/optimize_tail_latency.py --environment=staging

# Collect results
kubectl cp staging/load-test:/results/load_test_results.json ./staging-results.json

# Analyze
# - Error rate < 0.1%
# - P99 < 100ms
# - No memory leaks (check over 4 hour duration)
```

**Acceptance Criteria:**

- Error rate < 0.1%
- P99 latency < 100ms
- Memory usage stable (no growth > 10% over 4 hours)
- CPU usage reasonable (< 75% sustained)

---

#### Task 2.5: Run 72-Hour Soak Test

**Time:** 72 hours (passive monitoring) | **Owner:** SRE

```bash
# Configure sustained load
kubectl set env deployment/load-test-soak \
  LOAD_LEVEL=1000 \
  DURATION=72h \
  -n staging

# Monitor continuously
# Create dashboard: grafana/monitoring/soak-test.json

# Automated checks (every 6 hours)
while true; do
  error_rate=$(kubectl logs deployment/load-test-soak -n staging | \
    grep "error_rate" | tail -1 | awk '{print $2}')

  if (( $(echo "$error_rate > 1.0" | bc -l) )); then
    echo "ALERT: Error rate exceeded 1%"
    # Page on-call engineer
  fi

  sleep 6h
done

# After 72 hours: Analyze results
```

**Acceptance:**

- ✅ Error rate remained < 1% throughout
- ✅ P99 latency remained < 150ms
- ✅ Memory usage stable (< 5% growth over 72h)
- ✅ No pod restarts
- ✅ No data corruption

**Sign-off:** `Soak test passed - Ready for production`

---

### Thursday: Blue-Green Deployment Preparation

#### Task 3.1: Create Blue Environment (Current Prod)

**Time:** 30 minutes | **Owner:** DevOps Lead

```bash
# Document current production state
kubectl get all -n production -o json > blue-environment-backup.json
kubectl get secrets -n production -o json > blue-secrets-backup.json

# Label current environment as "BLUE"
kubectl label nodes -l environment=production color=blue

# Verify blue environment is stable
kubectl get nodes -L color
kubectl get pods -n production -o wide

# Record current metrics baseline
BLUE_LATENCY=$(kubectl logs -n production [pod] | grep "p99_latency" | tail -1)
BLUE_ERROR_RATE=$(kubectl logs -n production [pod] | grep "error_rate" | tail -1)

echo "BLUE baseline - P99: $BLUE_LATENCY, Error Rate: $BLUE_ERROR_RATE"
```

**Deliverable:** Blue environment documented and baselined

---

#### Task 3.2: Create Green Environment (New Prod)

**Time:** 2 hours | **Owner:** DevOps Lead

```bash
# Deploy to green nodes (fresh infrastructure)
# Pre-provision new nodes or use separate node pool

# Deploy new version to green
helm install phantommesh-green ./k8s/helm/phantommesh \
  -n production-green \
  --values k8s/overlays/prod/values.yaml \
  -f k8s/overlays/prod/canary/green-values.yaml

# Verify green is running
kubectl get pods -n production-green
kubectl rollout status deployment/phantommesh-green -n production-green --timeout=10m

# Run quick smoke tests against green
# - Health check: curl https://green.phantommesh/health
# - API test: curl https://green.phantommesh/api/version
# - Database connectivity: check logs for "DB connected"

# Verify green metrics are good (before traffic)
GREEN_LATENCY=$(kubectl logs -n production-green [pod] | grep "p99_latency" | tail -1)
GREEN_ERROR_RATE=$(kubectl logs -n production-green [pod] | grep "error_rate" | tail -1)

echo "GREEN baseline - P99: $GREEN_LATENCY, Error Rate: $GREEN_ERROR_RATE"
```

**Acceptance:**

- ✅ Green pods healthy
- ✅ Smoke tests passing
- ✅ Baseline metrics captured

---

#### Task 3.3: Set Up Traffic Mirroring

**Time:** 1 hour | **Owner:** DevOps Lead

```yaml
# Enable shadow traffic (send copy of prod traffic to green)
cat << EOF | kubectl apply -f -
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: phantommesh-canary
  namespace: production
spec:
  hosts:
  - phantommesh.example.com
  http:
  - match:
    - uri:
        prefix: /
    route:
    - destination:
        host: phantommesh-blue
        port:
          number: 443
      weight: 100
    - destination:
        host: phantommesh-green
        port:
          number: 443
      weight: 0
    mirror:
      host: phantommesh-green
      port:
        number: 443
    mirrorPercent: 100
EOF
```

**Result:** Green environment receives copy of all traffic (no user impact)

---

#### Task 3.4: Create Rollback Automation

**Time:** 1 hour | **Owner:** DevOps Lead

```bash
# Create automated rollback triggers

cat << 'EOF' > /opt/phantommesh/rollback-trigger.sh
#!/bin/bash

# Monitor green environment
# If error rate > 0.5% OR P99 latency > 150ms for 5 minutes → ROLLBACK

while true; do
  ERROR_RATE=$(kubectl logs -n production-green deployment/phantommesh | grep "error_rate" | tail -1 | awk '{print $2}')
  P99=$(kubectl logs -n production-green deployment/phantommesh | grep "p99" | tail -1 | awk '{print $2}')

  if (( $(echo "$ERROR_RATE > 0.5" | bc -l) )) || (( $(echo "$P99 > 150" | bc -l) )); then
    FAIL_COUNT=$((FAIL_COUNT + 1))
    if [ $FAIL_COUNT -gt 5 ]; then
      echo "ROLLBACK TRIGGERED: Error rate=$ERROR_RATE P99=$P99"

      # Switch traffic back to blue
      kubectl patch virtualservice phantommesh-canary -n production --type merge \
        -p '{"spec":{"http":[{"route":[{"destination":{"host":"phantommesh-blue"},"weight":100},{"destination":{"host":"phantommesh-green"},"weight":0}]}]}}'

      # Alert team
      curl -X POST $SLACK_WEBHOOK -d '{"text":"PHANTOMMESH ROLLBACK: Reverted to blue environment"}'

      exit 1
    fi
  else
    FAIL_COUNT=0
  fi

  sleep 60
done
EOF

chmod +x /opt/phantommesh/rollback-trigger.sh

# Start monitoring service
systemctl enable phantommesh-rollback-monitor
systemctl start phantommesh-rollback-monitor
```

**Acceptance:** Automated rollback trigger ready

---

### Friday: Pre-Deployment Verification

#### Task 4.1: Deployment Readiness Checklist

**Time:** 2 hours | **Owner:** DevOps Lead

```markdown
# Phase 4 Week 1 Completion Checklist

## Security Audit ✅

- [x] CIS Benchmark: 0 CRITICAL items
- [x] Container Scan: < 5 HIGH vulnerabilities
- [x] RBAC: Least privilege verified
- [x] Network Policies: Default deny + explicit allows
- [x] Secrets: All encrypted, rotation planned
- [x] Audit Report: Signed off

## Staging Validation ✅

- [x] Staging deployment: Successful
- [x] Production data replicated: Verified
- [x] Load test results: P99 < 100ms, Error rate < 0.1%
- [x] 72-hour soak test: Passed (memory stable, no crashes)

## Production Preparation ✅

- [x] Blue environment: Documented and baselined
- [x] Green environment: Deployed and tested
- [x] Traffic mirroring: Configured
- [x] Rollback automation: Active

## Team Readiness ✅

- [x] Incident response team: Briefed
- [x] On-call engineer: Assigned
- [x] Rollback procedures: Documented
- [x] Communication plan: Scheduled (Slack, email, status page)

## Final Sign-off

- DevOps Lead: ******\_\_\_****** Date: \_\_\_\_
- Security Lead: ******\_\_****** Date: \_\_\_\_
- VP Engineering: ******\_****** Date: \_\_\_\_

## Proceed to Week 2: APPROVED ☑️
```

---

#### Task 4.2: Schedule Deployment Window

**Time:** 30 minutes | **Owner:** DevOps Lead

```
PHASE 4 WEEK 2: PRODUCTION DEPLOYMENT

Deployment Window: Tuesday 2:00 AM - 6:00 AM UTC
├─ Off-peak hours selected
├─ Incident response team on call
├─ Slack channel #phantommesh-deployment open
└─ Status page: phantommesh.statuspage.io updated

Pre-deployment: Monday 4:00 PM
├─ Final system check
├─ Team briefing (30 min)
├─ Communication test (all channels working)
└─ Verify rollback ready

Deployment Steps:
├─ 2:00 AM - Start traffic mirror (green gets 100% shadow)
├─ 2:15 AM - Monitor green for 15 minutes
├─ 2:30 AM - 5% traffic to green (95% blue)
├─ 3:00 AM - 25% traffic to green (75% blue)
├─ 3:30 AM - 50% traffic to green (50% blue)
├─ 4:00 AM - 100% traffic to green
├─ 4:15 AM - Monitor for 15 minutes
├─ 4:30 AM - Decommission blue environment
└─ 6:00 AM - Deployment complete

Post-deployment: All week
├─ Continuous monitoring (24/7)
├─ Daily health checks
├─ Weekly review meetings
└─ Any rollback available for 7 days (keep blue running)
```

---

#### Task 4.3: Brief Incident Response Team

**Time:** 1 hour | **Owner:** DevOps Lead

**Meeting Agenda:**

```
Duration: 1 hour
Attendees: On-call engineer, DevOps lead, CTO, Product lead

1. Deployment overview (10 min)
   - What's changing: New version, optimized performance
   - Rollback available: 30 seconds if needed
   - Success criteria: < 0.1% error rate, < 100ms P99

2. Potential issues (10 min)
   - Database migration: 0-downtime strategy documented
   - Dependency updates: All tested in staging
   - New APIs: Backward compatible

3. Rollback procedure (15 min)
   - Automatic trigger: > 0.5% error for 5 minutes
   - Manual trigger: Run /opt/phantommesh/rollback.sh
   - Verification: Check blue environment receiving 100% traffic
   - Communication: Announce in #phantommesh-incident

4. Escalation path (10 min)
   - P1 (Critical): Page CTO, post to #incidents
   - P2 (Warning): Notify on-call, assess
   - P3 (Info): Document for post-mortem

5. Communication plan (10 min)
   - #phantommesh-deployment: Real-time updates
   - Status page: User-facing status
   - Email: Stakeholder notification
   - Slack threads: Team discussion
```

**Deliverable:** Team signed off, understands procedures

---

## 📊 WEEK 1 SUCCESS CRITERIA

- ✅ Security audit complete: 0 CRITICAL findings
- ✅ Staging validation passed: All tests passing
- ✅ 72-hour soak test passed: No issues
- ✅ Blue-green setup ready: Can switch in 30 seconds
- ✅ Rollback automation: Tested and ready
- ✅ Team briefed: All procedures understood
- ✅ Deployment window scheduled: Tuesday 2 AM UTC

**Status:** ✅ **READY FOR WEEK 2 DEPLOYMENT**

---

## 🚀 WEEK 2: PRODUCTION DEPLOYMENT

### Monday: Final Preparations

#### Task 5.1: Final System Check (4 hours before deployment)

```bash
# Verify everything one more time

# Blue (current prod): Stable?
kubectl get nodes -L color | grep blue
kubectl top nodes -l color=blue
kubectl get pods -n production --sort-by=.metadata.creationTimestamp

# Green (new prod): Ready?
kubectl get pods -n production-green --sort-by=.metadata.creationTimestamp
kubectl logs -n production-green -l app=phantommesh | head -50

# Load balancer: Routing to blue?
kubectl get virtualservice -n production

# Monitoring: All dashboards up?
# - Grafana: Check production dashboard
# - Prometheus: Verify scraping all targets
# - Alerting: All rules active?

# Incident response: Team on call?
# - Verify on-call engineer assigned
# - Slack notifications enabled
# - PagerDuty escalations configured
```

---

#### Task 5.2: Team Briefing (30 minutes before deployment)

```
Attendees: DevOps, SRE, On-call, CTO, Product

Agenda:
- Deployment window: 2 AM UTC, ~4 hours
- Success criteria: < 0.1% error rate throughout
- Rollback: Available at any time (< 30 sec)
- Communication: Updates every 15 min to #phantommesh-deployment
- Questions? Ask now!
- GO/NO-GO decision: All agree? YES
```

---

### Tuesday: Production Deployment (2:00 AM - 6:00 AM UTC)

#### 2:00 AM - Start Traffic Mirror

```bash
# Green receives 100% shadow traffic (users unaffected)
# Monitor green metrics for 15 minutes
# Watch for: errors, latency, resource usage
```

**Success:** Green metrics stable  
**On Failure:** Stop here, investigate, reschedule

---

#### 2:30 AM - Canary 5% (5% green, 95% blue)

```bash
kubectl patch virtualservice phantommesh-canary -n production --type merge \
  -p '{"spec":{"http":[{"route":[{"destination":{"host":"phantommesh-blue"},"weight":95},{"destination":{"host":"phantommesh-green"},"weight":5}]}]}}'

# Monitor: 5% of real users now on green
# Duration: 30 minutes
# Watch: Any issues? Error rate increasing?
```

**Success:** No issues, error rate stable  
**On Failure:** ROLLBACK to 0% green

---

#### 3:00 AM - Canary 25% (25% green, 75% blue)

```bash
kubectl patch virtualservice phantommesh-canary -n production --type merge \
  -p '{"spec":{"http":[{"route":[{"destination":{"host":"phantommesh-blue"},"weight":75},{"destination":{"host":"phantommesh-green"},"weight":25}]}]}}'

# Monitor: 25% of users now on green
# Duration: 30 minutes
# Watch: Any patterns emerging?
```

**Success:** No issues, metrics good  
**On Failure:** ROLLBACK to 5% green, investigate

---

#### 3:30 AM - Canary 50% (50% green, 50% blue)

```bash
kubectl patch virtualservice phantommesh-canary -n production --type merge \
  -p '{"spec":{"http":[{"route":[{"destination":{"host":"phantommesh-blue"},"weight":50},{"destination":{"host":"phantommesh-green"},"weight":50}]}]}}'

# Monitor: Half of users on new version
# Duration: 30 minutes
# This is the critical point - half the traffic
```

**Success:** No increase in errors, latency acceptable  
**On Failure:** ROLLBACK to 25% green

---

#### 4:00 AM - Full Cutover (100% green, 0% blue)

```bash
kubectl patch virtualservice phantommesh-canary -n production --type merge \
  -p '{"spec":{"http":[{"route":[{"destination":{"host":"phantommesh-blue"},"weight":0},{"destination":{"host":"phantommesh-green"},"weight":100}]}]}}'

# Monitor: ALL users on green version
# Duration: 15 minutes critical monitoring
# Then continue 24-hour monitoring
```

**Success:** All traffic on green, zero issues  
**On Failure:** ROLLBACK to blue (< 30 sec)

---

#### 4:30 AM - Decommission Blue

```bash
# Only after green stable for 30+ minutes

# Scale down blue environment
kubectl scale deployment phantommesh-blue --replicas=0 -n production

# Verify: All traffic healthy on green
kubectl get pods -n production-green

# Archive blue backup
kubectl get all -n production -o json > backups/blue-production-[DATE].json
```

---

#### 6:00 AM - Deployment Complete ✅

```
Summary Report:
- Start time: 2:00 AM UTC
- End time: 6:00 AM UTC
- Total duration: 4 hours
- Traffic cutover: Successful
- Errors: < 0.1% throughout
- P99 latency: < 100ms throughout
- Rollbacks: 0
- Status: ✅ PRODUCTION LIVE
```

---

## 📈 POST-DEPLOYMENT MONITORING (Weeks 1-2)

### Continuous Monitoring Protocol

```
Hour 0-6 (Deployment day):
├─ 15-minute dashboard checks
├─ All hands on deck
├─ Any anomaly → immediate investigation

Hour 6-24 (First day):
├─ Hourly checks
├─ System stable?
├─ Error rate steady?
├─ User reports?

Day 2-7 (Week 1):
├─ Daily morning check
├─ Weekly metrics review
├─ Blue environment: Keep for 7 days (rollback available)

Week 2+:
├─ Weekly review meetings
├─ Blue environment: Decommission after 2 weeks
├─ Archive all backup data
└─ Begin Phase 5 observability work
```

### Daily Health Check Template

```bash
#!/bin/bash

echo "=== PhantomMesh Production Health Check ==="
echo "Time: $(date)"

# Pod health
echo "Pod Status:"
kubectl get pods -n production | tail -5

# Error rate
echo "Error Rate (last hour):"
kubectl logs -n production deployment/phantommesh --tail=1000 | grep "error_rate" | tail -1

# Latency
echo "P99 Latency (last hour):"
kubectl logs -n production deployment/phantommesh --tail=1000 | grep "p99" | tail -1

# Resource usage
echo "Resource Usage:"
kubectl top pods -n production

# Recent errors
echo "Recent Errors:"
kubectl logs -n production deployment/phantommesh --tail=100 | grep "ERROR" | tail -5

echo "Status: HEALTHY ✅"
```

**Run this daily:** 9 AM UTC for 2 weeks

---

## 🚨 CONTINGENCY: ROLLBACK PROCEDURES

### Automated Rollback (Happens automatically)

```
Trigger 1: Error rate > 0.5% for 5 minutes
Trigger 2: P99 latency > 150ms for 5 minutes
Trigger 3: Pod restart loops (>3 restarts/minute)

When triggered:
1. All traffic → blue (instant)
2. PagerDuty alert → on-call (immediate)
3. Slack notification (automatic)
4. Investigation begins
```

### Manual Rollback (If needed)

```bash
# Step 1: Evaluate situation (2 minutes)
kubectl logs -n production-green deployment/phantommesh | tail -100
kubectl logs -n production-blue deployment/phantommesh | tail -100

# Step 2: If serious: ROLLBACK
/opt/phantommesh/rollback-manual.sh

# Step 3: Verify blue receiving 100% traffic
kubectl get virtualservice -n production

# Step 4: Notify team
# Post to #phantommesh-incident: "ROLLBACK INITIATED: Reason: [specific issue]"

# Step 5: Begin investigation
# Don't re-deploy until root cause found
```

---

## 📋 WEEK 2 COMPLETION CHECKLIST

- [ ] Deployment executed successfully
- [ ] All traffic routed to green
- [ ] Error rate < 0.1%
- [ ] P99 latency < 100ms
- [ ] No pod crashes
- [ ] Post-deployment monitoring active
- [ ] Team debriefing completed
- [ ] Blue environment archived
- [ ] Phase 4 sign-off obtained

**Success:** ✅ **PRODUCTION DEPLOYMENT COMPLETE**

---

## 🎓 LESSONS LEARNED (Post-Deployment)

Schedule 1-hour meeting: Friday end of week

**Retrospective Template:**

```markdown
## What Went Well

- [Item]
- [Item]

## What Could Be Better

- [Item]
- [Item]

## Action Items for Phase 5

- [ ] Improve [process]
- [ ] Automate [procedure]
- [ ] Document [finding]

## Metrics Review

- Actual errors vs. expected: \_\_\_\_
- Actual latency vs. baseline: \_\_\_\_
- Customer impact: \_\_\_\_
- Team learning: \_\_\_\_
```

---

**Status:** Ready to Execute Week 1  
**Owner:** DevOps Lead + Security Engineer  
**Timeline:** Monday - Friday (5 days Week 1), Tuesday deployment Week 2  
**Success Criteria:** 0 critical security findings, < 5% deployment errors, < 10ms impact

**This is your operational blueprint. Execute systematically. 🚀**
