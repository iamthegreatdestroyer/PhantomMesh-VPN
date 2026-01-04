# 🔄 PhantomMesh VPN - Production Migration & Cutover Plan

**Document Version:** 1.0  
**Status:** READY FOR EXECUTION  
**Created:** 2026-01-04  
**Target Go-Live:** [TO BE SCHEDULED]

---

## Table of Contents

1. [Overview](#overview)
2. [Pre-Migration Planning](#pre-migration-planning)
3. [Data Migration Strategy](#data-migration-strategy)
4. [Infrastructure Cutover](#infrastructure-cutover)
5. [User Migration](#user-migration)
6. [Rollback Plan](#rollback-plan)
7. [Success Criteria](#success-criteria)
8. [Communication Plan](#communication-plan)

---

## Overview

### Scope

```
Systems Migrating:
  ✅ VPN Core Infrastructure
  ✅ API Gateway & Services
  ✅ Agent Swarm & Discovery
  ✅ Monitoring & Observability
  ✅ Database & Storage
  ✅ User Management
  ✅ Configuration & Secrets

Parallel Systems:
  ✅ Legacy system remains operational during transition
  ✅ Dual-write validation period
  ✅ Gradual traffic migration
  ✅ Zero-downtime deployment approach
```

### Migration Timeline

**Phase 1: Preparation (T-7 days)**

- Infrastructure readiness validation
- Data migration testing
- User communication launch
- Team training completion

**Phase 2: Pre-Cutover (T-3 days)**

- Final backups created
- Rollback procedures tested
- On-call team briefing
- Last-minute checks

**Phase 3: Migration Window (T-day)**

- Infrastructure cutover (1 hour)
- Data migration (30 minutes)
- DNS switchover (5 minutes)
- Validation & verification (2 hours)
- User rollout (ongoing)

**Phase 4: Stabilization (T+7 days)**

- 24/7 monitoring
- Performance optimization
- Issue resolution
- Legacy system retirement planning

---

## Pre-Migration Planning

### 1. Infrastructure Validation

#### Kubernetes Cluster Readiness

```bash
# Pre-migration checklist (execute 7 days before)

# 1. Cluster health
kubectl get nodes -o wide
kubectl get componentstatuses
kubectl get persistentvolumes

# 2. Resource capacity
kubectl top nodes
kubectl top pods --all-namespaces

# 3. Network connectivity
kubectl run -it test-pod --image=busybox -- \
  wget -q -O- http://kubernetes.default.svc

# 4. Storage verification
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: storage-test
spec:
  containers:
  - name: test
    image: busybox
    command: ['sh', '-c', 'dd if=/dev/zero of=/data/test bs=1M count=100']
    volumeMounts:
    - name: data
      mountPath: /data
  volumes:
  - name: data
    emptyDir: {}
EOF

# 5. Database connection test
kubectl run -it db-test --image=postgres:15 -- \
  psql -h prod-postgres-svc -U phantommesh -c "SELECT 1"
```

#### Network & Security Readiness

```bash
# TLS certificate validation
openssl x509 -in /path/to/cert.pem -text -noout | grep -E "Issuer:|Not Before|Not After"

# DNS readiness
nslookup phantommesh.example.com

# Firewall rules test (to be executed by Network Ops)
- Inbound: 443 (HTTPS) - OPEN
- Inbound: 51820 (WireGuard) - OPEN
- Outbound: 53 (DNS) - OPEN
- Outbound: 123 (NTP) - OPEN
```

### 2. Data Migration Readiness

#### Database Backup

```bash
#!/bin/bash
# Run 3 days before migration

# 1. Full backup
pg_dump -h staging-db -U phantommesh -v \
  -Fc phantommesh > backup_$(date +%Y%m%d_%H%M%S).dump

# 2. Backup verification
pg_restore --list backup_*.dump | wc -l

# 3. Test restore in isolated environment
pg_restore -h test-db -U phantommesh -d phantommesh_test backup_*.dump

# 4. Data integrity check
psql -h test-db -U phantommesh phantommesh_test -c \
  "SELECT COUNT(*) FROM users; SELECT COUNT(*) FROM sessions;"

# 5. Store backup securely
aws s3 cp backup_*.dump s3://phantommesh-backups/pre-migration/ \
  --sse aws:kms --storage-class GLACIER
```

#### Data Validation Rules

```sql
-- Pre-migration data quality checks
SELECT
  'Users' as table_name,
  COUNT(*) as record_count,
  COUNT(DISTINCT id) as unique_ids,
  COUNT(CASE WHEN email IS NULL THEN 1 END) as null_emails
FROM users;

SELECT
  'Sessions' as table_name,
  COUNT(*) as record_count,
  COUNT(CASE WHEN expires_at < NOW() THEN 1 END) as expired_sessions,
  COUNT(CASE WHEN user_id IS NULL THEN 1 END) as orphaned_sessions
FROM sessions;

SELECT
  'Configurations' as table_name,
  COUNT(*) as record_count,
  COUNT(DISTINCT organization_id) as unique_orgs
FROM configurations;
```

### 3. Team Preparation

#### Training Schedule (T-14 to T-7)

```
Monday:   Architecture walkthrough (all ops team)
Tuesday:  Deployment procedure training (deployment team)
Wednesday: Troubleshooting & incident response (ops team)
Thursday: Rollback procedures & testing (backup team)
Friday:   Full simulation exercise (all teams)
```

#### Role Assignments

```
Migration Lead:
  - Overall coordination
  - Executive communication
  - Decision authority
  - Contact: [PRIMARY]

Infrastructure Lead:
  - Kubernetes operations
  - Database migration
  - Storage management
  - Contact: [BACKUP 1]

Application Lead:
  - Service deployment
  - API validation
  - User functionality
  - Contact: [BACKUP 2]

Security Lead:
  - Compliance monitoring
  - Incident detection
  - Audit trail maintenance
  - Contact: [BACKUP 3]

Network/DNS Lead:
  - DNS cutover
  - Firewall rules
  - CDN configuration
  - Contact: [BACKUP 4]

Communications Lead:
  - User notifications
  - Status page updates
  - Executive reporting
  - Contact: [BACKUP 5]
```

---

## Data Migration Strategy

### 1. Pre-Cutover Phase (Parallel Systems)

#### Dual-Write Implementation

```
Days 1-7 Before Cutover:

┌──────────────────┐
│  User Request    │
└────────┬─────────┘
         │
    ┌────▼─────┐
    │ API Proxy │
    └────┬─────┘
         │
    ┌────┴────┬──────────┐
    │          │          │
    ▼          ▼          ▼
┌─────────┐ ┌────────┐ ┌────────────┐
│ Legacy  │ │ Staging│ │ Validation │
│ System  │ │ System │ │ System     │
└─────────┘ └────────┘ └────────────┘

Data Flow:
1. All writes go to legacy system (primary)
2. All writes replicated to staging (secondary)
3. Validation system compares both
4. Report discrepancies for manual resolution
```

#### Implementation Steps

```
Step 1: Enable dual-write in API layer
  - Code: api/middleware/dual_write.py
  - Fallback to legacy if staging fails
  - Log all dual-write operations

Step 2: Start replication (read-only)
  - Logical replication from Legacy → Staging
  - Monitor replication lag (< 100ms target)
  - Alert if lag exceeds 1 second

Step 3: Data validation
  - Compare record counts hourly
  - Compare checksums of key tables
  - Manual review of discrepancies
  - Resolution and re-sync

Step 4: Warm up staging system
  - Read traffic gradually shifted
  - Start with 10% of reads
  - Increase 10% daily
  - Monitor performance metrics

Status Check (after 7 days):
  ✅ Replication lag stable
  ✅ Data consistency verified
  ✅ Staging performance validated
  ✅ User connections tested
```

### 2. Migration Day (Cutover)

#### Phase 2A: Final Preparation (T-2 hours)

```bash
# 1. Freeze legacy system writes
mysql> SET GLOBAL read_only = ON;

# 2. Stop replication
STOP REPLICA;

# 3. Verify final consistency
CHECKSUM TABLE users, sessions, configurations;

# 4. Create final backup
pg_dump -h legacy-db -U admin -Fc legacy_db > final_backup.dump

# 5. Copy to staging
scp final_backup.dump ops@staging:/backups/

# 6. Restore to staging
pg_restore -h staging-db -U phantommesh -d phantommesh final_backup.dump

# 7. Verify restoration
SELECT COUNT(*) FROM users;  -- Should match legacy count
```

#### Phase 2B: Infrastructure Cutover (T hour - 1 hour window)

```
Timeline (60 minute window):

00:00 - 00:05: DNS Preparation
  - Update TTL to 60 seconds (currently 3600)
  - Flush DNS caches at major ISPs
  - Notify CDN of upcoming change

00:05 - 00:10: Load Balancer Reconfig
  - Add staging systems to load balancer
  - Weight = 0% initially
  - Gradual weight increase

00:10 - 00:20: Gradual Traffic Shift
  Weight    Traffic Sent To
  Legacy    Staging
  100%  → 0%
  90%   → 10%
  80%   → 20%
  70%   → 30%
  60%   → 40%
  50%   → 50%
  40%   → 60%
  30%   → 70%
  20%   → 80%
  10%   → 90%
  0%    → 100%

  Monitoring: Error rate should remain < 0.1%

00:20 - 00:30: API Gateway Cutover
  - Update API endpoints
  - Validate DNS resolution
  - Check certificate validity

00:30 - 00:50: Application Validation
  - Execute smoke tests
  - Verify user login
  - Check VPN tunnel creation
  - Validate metrics collection
  - Check backup operations

00:50 - 01:00: Legacy System Drain
  - Set connection_max_age = 0 on legacy
  - Wait for existing connections to drain
  - Monitor connection count → 0
  - Prepare for rollback by keeping legacy live
```

#### Phase 2C: Monitoring & Validation (T+1 to T+3 hours)

```
Automated Checks (every 30 seconds):

Health Metrics:
  ✅ HTTP 200 response on /health
  ✅ Database query latency < 100ms
  ✅ API response time < 500ms
  ✅ Error rate < 0.1%
  ✅ Pod restart count = 0
  ✅ Memory usage < 80%
  ✅ CPU usage < 80%

Functional Tests:
  ✅ User authentication working
  ✅ VPN config generation working
  ✅ Peer discovery responding
  ✅ Metrics being collected
  ✅ Logs being aggregated
  ✅ Backups executing

Alert Conditions (page on-call if any triggered):
  ❌ Error rate > 1%
  ❌ Latency P99 > 1000ms
  ❌ Pod restart rate > 1/hour
  ❌ Database connection failure
  ❌ Certificate validation failure
  ❌ API endpoint down
```

### 3. Post-Cutover Stabilization

#### Hour 1-4: Active Monitoring

```
Assign: Two on-call engineers monitoring continuously
Interval: Check metrics every 5 minutes
Focus:
  - User login success rate
  - VPN connection establishment
  - Error logs analysis
  - Performance metrics trending

Actions if issues detected:
  - Slack alert to team
  - Page additional engineer if P2
  - Page engineering lead if P1
- Consider rollback if critical issue detected
```

#### Hour 4-24: Escalated Monitoring

```
Assign: One on-call + support team
Interval: Check metrics every 15 minutes
Focus:
  - Sustained performance metrics
  - User-reported issues tracking
  - Error pattern analysis
  - Compare against baseline

Actions:
  - Daily report to stakeholders
  - Address any emerging issues
  - Optimize if performance degraded
```

#### Day 2-7: Standard Monitoring

```
Assign: Standard on-call rotation
Monitoring: Automated alerts + daily review
Focus:
  - Performance stabilization
  - User adoption metrics
  - Issue trend analysis
  - Optimization opportunities

Deliverables:
  - Daily status report (first 3 days)
  - End of week summary
  - Lessons learned document
```

---

## User Migration

### 1. Communication Strategy

#### Pre-Migration (T-14 to T-7)

```
Day 1: Announcement
  - Email: "Upcoming System Upgrade"
  - Highlight: Improved performance, new features
  - Timeline: Show T-day schedule
  - Action: None required from users

Day 5: Technical Details
  - Blog post: Architecture improvements
  - FAQ: Common questions addressed
  - Support: Help desk prepared for inquiries

Day 10: Migration Day Notification
  - Email: Final reminder with schedule
  - Expected downtime: 5-10 minutes during transition
  - No action required
  - Support: Phone lines staffed
```

#### Migration Day

```
T-2 hours: Pre-notification
  - Slack: "Maintenance window begins in 2 hours"
  - Email: Final heads-up
  - SMS: For premium customers (optional)

T-30 min: Active maintenance window
  - Status page: "Maintenance in progress"
  - Email: Automatic notification
  - Slack: Live updates in #status channel

T+15 min: Cutover complete
  - Status page: "Systems back online"
  - Email: Confirmation sent
  - Monitor for issues from users

T+4 hours: All-clear notification
  - Email: "Upgrade completed successfully"
  - Statistics: Performance improvements noted
  - Thanks: Appreciation for patience
```

### 2. User Verification Checklist

For a representative sample of users:

```
□ User can log in
□ Can download new VPN config (if applicable)
□ VPN tunnel connects successfully
□ Traffic flows through tunnel
□ Can access resources as before
□ Performance meets expectations
□ No error messages in logs
□ Support team has received no complaints
```

---

## Rollback Plan

### Automatic Rollback (Immediate - 0 to 5 minutes)

```
If critical system fails during migration:

1. Kubernetes automatically detects pod failure
2. Service mesh routes to healthy instances
3. If all instances unhealthy: automatic pod restart
4. If pods persist failing: services degrade gracefully
5. Staging system removed from load balancer
6. Legacy system becomes primary again
```

### Manual Rollback (5 to 30 minutes)

```
If manual intervention needed:

Step 1: Pause traffic to staging
  kubectl scale deployment vpn-core --replicas=0 -n phantommesh-prod

Step 2: Restore legacy database from backup
  mysql -h legacy-db < /backups/pre-migration.sql

Step 3: Verify legacy system health
  curl -v https://phantommesh-legacy.example.com/health

Step 4: Update DNS to point to legacy
  # Update in DNS provider UI or via CLI
  aws route53 change-resource-record-sets ...

Step 5: Verify user access
  # Test login, VPN tunnel, etc.

Step 6: Communicate with users
  - Email: "We have reverted to the previous system..."
  - Status page: "Maintenance canceled - on previous version"
  - Support: Stand by for user inquiries
```

### Full Rollback (Disaster scenario)

```
If entire migration needs to be cancelled:

1. Execute manual rollback procedure (above)
2. Decommission staging infrastructure
3. Identify root cause through investigation
4. Schedule new migration attempt (after 1 week minimum)
5. Post-mortem meeting to review failure
6. Update procedures based on lessons learned
```

### Rollback Success Criteria

```
Migration is considered successful if:
  ✅ Zero data loss
  ✅ All users can authenticate
  ✅ VPN connections working
  ✅ Error rate < 0.1%
  ✅ P99 latency < 200ms
  ✅ All services responding

If any of above not met within 1 hour:
  → Rollback to legacy system
```

---

## Success Criteria

### Technical Success

```
Metric              Target    Actual    Status
────────────────────────────────────────────
Error Rate          < 0.1%    [Monitor] ✅
P50 Latency         < 50ms    [Monitor] ✅
P99 Latency         < 200ms   [Monitor] ✅
Availability        99.99%    [Monitor] ✅
Database Lag        < 100ms   [Monitor] ✅
API Response Time   < 500ms   [Monitor] ✅
Memory Usage        < 80%     [Monitor] ✅
CPU Usage           < 80%     [Monitor] ✅
Disk Usage          < 85%     [Monitor] ✅
Pod Restart Count   0/hour    [Monitor] ✅
```

### User Experience Success

```
□ Zero unexpected downtime
□ Seamless authentication experience
□ VPN connection establishment < 5 seconds
□ No user-facing errors
□ Improved performance vs legacy
□ All features working as before
□ Support team receives < 10 issues
```

### Business Success

```
□ Zero customer escalations
□ No media/public relations impact
□ Successful go-live on schedule
□ Team morale positive
□ Knowledge captured for documentation
□ Rollback procedures validated
□ Post-mortem completed
□ Lessons learned implemented
```

---

## Communication Plan

### Stakeholder List

```
Internal:
  - Engineering team (deployment)
  - Operations team (on-call)
  - Product team (go/no-go decision)
  - Executive team (status updates)

External:
  - Customers (via email/status page)
  - Support team (for escalations)
  - Vendors (if needed)
```

### Communication Schedule

```
T-14 days: Executive review & approval
T-7 days:  Final team meeting & training
T-1 day:   All teams ready confirmation
T-4 hours: Kick-off meeting
T-2 hours: Go/no-go decision (final)
T hour:    Migration begins
T+1 hour:  Status update
T+4 hours: All-clear confirmation
T+1 day:   Success report
T+5 days:  Final retrospective
```

### Status Page Template

```
Title: System Maintenance - Production Infrastructure Upgrade
Status: [INVESTIGATING] → [MAINTENANCE] → [VERIFYING] → [OPERATIONAL]
Duration: [Start Time] to [Estimated End Time]
Impact: Users may experience intermittent connectivity (5-10 min)

Updates:
[13:00] Maintenance window opened - Infrastructure cutover in progress
[13:15] DNS updates in progress - gradual traffic migration starting
[13:45] All services responding normally - validation in progress
[14:00] Upgrade completed successfully - monitoring closely
[14:30] All systems operating within normal parameters
[15:00] Maintenance completed - Thank you for your patience
```

---

## Appendix: Checklist

### Pre-Migration (T-7 days)

- [ ] Final security audit completed
- [ ] Infrastructure readiness validated
- [ ] Database backups tested
- [ ] Rollback procedures tested
- [ ] Team training completed
- [ ] On-call schedule finalized
- [ ] Communication plan finalized
- [ ] Vendor notified (if applicable)

### Pre-Cutover (T-1 day)

- [ ] Final backups created
- [ ] DNS TTL reduced to 60 seconds
- [ ] Load balancer tested
- [ ] Staging system warmed up
- [ ] Smoke tests passing
- [ ] Incident response team assembled
- [ ] Executive approval obtained

### Migration Day

- [ ] Team assembled 1 hour early
- [ ] Communications channels open
- [ ] Monitoring dashboards active
- [ ] Backup systems ready
- [ ] Rollback procedures available
- [ ] Status page updated
- [ ] Customer notifications sent

### Post-Migration (T+7 days)

- [ ] All metrics stable
- [ ] Zero P1/P2 issues outstanding
- [ ] Legacy system decommissioned
- [ ] Documentation updated
- [ ] Post-mortem meeting completed
- [ ] Lessons learned documented
- [ ] Team celebration (well-deserved!)

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-04  
**Next Review:** After each migration phase

_This plan should be reviewed and updated based on lessons learned from each step._
