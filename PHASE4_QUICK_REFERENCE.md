# 🚀 PHASE 4 QUICK REFERENCE CARD

**Print This. Keep at Desk.** 📋

---

## ⚡ WEEK 1 DAILY CHECKLIST

### Monday: Security Audit Day

```
☐ 9:00 AM   Run kube-bench: ./phase4_execute.sh audit
☐ 10:00 AM  Scan container images with Trivy
☐ 11:00 AM  Audit RBAC configuration
☐ 12:00 PM  Review findings, fix CRITICAL items
☐ 2:00 PM   Generate security audit report
☐ 3:00 PM   Team review & sign-off
☐ 4:00 PM   Document all findings
```

### Tuesday-Wednesday: Staging Deployment

```
☐ 9:00 AM   Deploy staging: ./phase4_execute.sh staging
☐ 10:00 AM  Replicate production data to staging
☐ 11:00 AM  Run load tests: ./phase4_execute.sh load-test
☐ 12:00 PM  Review test results
☐ 1:00 PM   START 72-hour soak test: ./phase4_execute.sh soak-test
☐ 2:00 PM   Monitor soak test metrics
☐ Daily     Check soak test dashboard (auto-monitoring)
```

### Thursday: Production Prep

```
☐ 9:00 AM   Set up blue-green: ./phase4_execute.sh blue-green
☐ 10:00 AM  Smoke test green environment
☐ 11:00 AM  Configure traffic mirroring
☐ 12:00 PM  Set up rollback automation
☐ 1:00 PM   Create deployment runbook (if needed)
☐ 2:00 PM   Brief incident response team
☐ 3:00 PM   Final sign-off meeting
```

### Friday: Final Verification

```
☐ 9:00 AM   Security audit final review
☐ 10:00 AM  Staging validation complete?
☐ 11:00 AM  Blue-green setup verified?
☐ 12:00 PM  Rollback automation tested?
☐ 1:00 PM   Team briefing 2.0 (final checks)
☐ 2:00 PM   Deployment window scheduled (Tuesday 2 AM)
☐ 3:00 PM   Week 1 sign-off complete
```

---

## 🚀 WEEK 2: DEPLOYMENT DAY (Tuesday 2-6 AM UTC)

### Timeline

```
2:00 AM  ├─ Deploy script: ./phase4_execute.sh deploy
2:30 AM  ├─ Green receives 100% shadow traffic
3:00 AM  ├─ Canary 5% (5% real users on green)
3:30 AM  ├─ Canary 25%
4:00 AM  ├─ Canary 50%
4:30 AM  ├─ Canary 100% (all users on green)
5:00 AM  ├─ Verify: No issues, all metrics good
5:30 AM  ├─ Decommission blue
6:00 AM  └─ DEPLOYMENT COMPLETE ✅
```

### Monitoring During Deployment

```
Open 3 terminals:

Terminal 1: kubectl logs -n production-green deployment/phantommesh -f
Terminal 2: kubectl logs -n production deployment/phantommesh -f
Terminal 3: Watch Grafana: http://grafana:3000/d/production
```

---

## 📊 SUCCESS METRICS

### Week 1 Audit

```
Security Audit:
✅ 0 CRITICAL findings (required)
✅ < 5 HIGH vulnerabilities (acceptable)
✅ All findings documented
```

### Week 1 Staging

```
Load Test:
✅ P99 < 100ms (target: 200ms)
✅ Error rate < 0.1% (target: <1%)
✅ Peak throughput: 1,050 req/sec

72-Hour Soak:
✅ Memory usage stable (< 5% growth)
✅ No pod restarts
✅ Error rate consistent < 0.1%
```

### Week 2 Deployment

```
Canary Deployment:
✅ 0 errors during 5% stage
✅ 0 errors during 25% stage
✅ 0 errors during 50% stage
✅ 0 errors during 100% stage
✅ Total deployment errors: < 5% of traffic

Post-Deployment (24 hours):
✅ Error rate: < 0.1%
✅ P99 latency: < 100ms
✅ Zero unexpected pod restarts
✅ User reports: 0
```

---

## 🔧 COMMAND QUICK REFERENCE

### Security Audit

```bash
./phase4_execute.sh audit                    # Full security audit
cat audit-reports/audit_*/AUDIT_SUMMARY.md   # Review results
```

### Staging

```bash
./phase4_execute.sh staging                  # Deploy to staging
./phase4_execute.sh load-test                # Run load test
./phase4_execute.sh soak-test                # Start 72-hour test
```

### Blue-Green

```bash
./phase4_execute.sh blue-green               # Set up blue-green
./phase4_execute.sh deploy                   # Execute canary deployment
./phase4_execute.sh health                   # Health check
```

### Manual Kubectl (if needed)

```bash
# Check pod status
kubectl get pods -n production

# View logs
kubectl logs -n production deployment/phantommesh -f

# Check metrics
kubectl top pods -n production

# Manual rollback (if automation fails)
kubectl patch virtualservice phantommesh-canary -n production \
  -p '{"spec":{"http":[{"route":[{"destination":{"host":"phantommesh-blue"},"weight":100}]}]}}'
```

---

## 🚨 EMERGENCY PROCEDURES

### If Something Goes Wrong

#### High Error Rate (> 0.5% for 5 minutes)

```
1. Check: kubectl logs -n production-green deployment/phantommesh | tail -50
2. Decision: Fix or rollback?
   - FIX: kubectl rollout restart deployment/phantommesh -n production-green
   - ROLLBACK: ./phase4_execute.sh rollback (auto-trigger should handle)
3. Notify: Post in #phantommesh-incident
4. Investigate: Don't redeploy until root cause found
```

#### High Latency (P99 > 150ms)

```
1. Check: kubectl top pods -n production-green
2. If high memory: Likely memory leak
3. If high CPU: Likely algorithm issue
4. Rollback to blue: ./phase4_execute.sh rollback
5. Investigate in staging before redeploying
```

#### Pod Crashes (Restart loops)

```
1. Check: kubectl describe pod -n production-green [POD_NAME]
2. View logs: kubectl logs -n production-green [POD_NAME]
3. If OOMKilled: Memory limit too low
4. If CrashLoopBackOff: Code issue
5. Rollback: ./phase4_execute.sh rollback
6. Fix in staging, redeploy in Phase 4 Week 2
```

#### Database Connection Failure

```
1. Check: kubectl exec -n production-green [POD] -- psql -h [HOST] -U postgres -c "SELECT 1;"
2. Verify: Database is accessible and has correct credentials
3. If failed: Database might be down or network issue
4. Rollback: Blue environment uses known-good database
5. Investigate: Check DB health and credentials
```

---

## 📞 ESCALATION PATH

### P1 (Critical) - Page immediately

- Error rate > 1%
- P99 latency > 500ms
- Pod crashes (CrashLoopBackOff)
- Database connectivity lost
- **Action:** Rollback to blue, page on-call engineer

### P2 (High) - Within 15 minutes

- Error rate > 0.5% (but < 1%)
- P99 latency > 150ms
- Memory/CPU trending high
- **Action:** Investigate, decide fix vs rollback

### P3 (Info) - Log for post-mortem

- Error rate 0.1-0.5%
- P99 latency 100-150ms
- Minor metric anomalies
- **Action:** Monitor, document findings

---

## 📋 SIGN-OFF CHECKLIST

### Week 1 Sign-Off (Friday)

```
☐ Security audit: Signed off by Security Lead
☐ Staging validation: Signed off by SRE Lead
☐ Soak test (72h): Confirmed passing
☐ Blue-green setup: Verified and tested
☐ Rollback automation: Tested and ready
☐ Team briefing: All procedures understood
☐ Deployment window: Scheduled (Tuesday 2 AM UTC)

Final Approval:
- DevOps Lead: ____________________
- SRE Lead: _______________________
- CTO: ___________________________
```

### Week 2 Sign-Off (Post-Deployment)

```
☐ Traffic successfully routed to green (100%)
☐ Error rate remained < 0.1% throughout deployment
☐ P99 latency remained < 100ms
☐ Zero pod crashes during deployment
☐ Blue environment decommissioned
☐ Post-deployment monitoring active (24 hours)
☐ User impact: Zero incidents reported

Final Approval:
- DevOps Lead: ____________________
- VP Engineering: __________________
```

---

## 📈 MONITORING DASHBOARDS

**During Deployment (Keep Open):**

```
1. Grafana Production: http://grafana:3000/d/production
   - P99 latency
   - Error rate
   - Request throughput
   - CPU/Memory usage

2. Kubectl Monitoring:
   kubectl get pods -n production -w

3. Log Monitoring:
   kubectl logs -n production-green -f

4. Status Page: https://phantommesh.statuspage.io
```

---

## 🎓 KEY TAKEAWAYS

1. **Automation First:** Use `./phase4_execute.sh` for all major operations
2. **Safety First:** Always verify blue environment is stable before deploying green
3. **Canary Progression:** Never jump from 0% to 100% - use stages
4. **Monitoring:** Have 3 terminals open during deployment
5. **Communication:** Update #phantommesh-deployment every 15 minutes
6. **Rollback Ready:** If anything feels wrong, rollback immediately (30 sec recovery)

---

## 📞 CONTACTS

**Incident Response:**

- On-Call Engineer: [PHONE/SLACK]
- DevOps Lead: [PHONE/SLACK]
- CTO: [PHONE/SLACK]

**Escalation:**

- P1: All three
- P2: On-Call + DevOps Lead
- P3: DevOps Lead (log for review)

**Communication Channels:**

- Real-time: #phantommesh-deployment (Slack)
- Status: phantommesh.statuspage.io
- Post-incident: #phantommesh-postmortem

---

## ✅ FINAL CHECKLIST

**Before Starting Week 1:**

- [ ] All scripts downloaded: `phantom-mesh-vpn/scripts/phase4_execute.sh`
- [ ] Audit directory created: `mkdir -p audit-reports`
- [ ] Backups directory created: `mkdir -p backups`
- [ ] This card printed and posted: ✓

**Before Starting Week 2:**

- [ ] Week 1 sign-offs obtained
- [ ] Incident response team confirmed ready
- [ ] All monitoring dashboards open
- [ ] Slack notification channels tested
- [ ] Phone numbers verified for escalation

---

**Status: READY TO EXECUTE PHASE 4** 🚀

_Last updated: January 4, 2026_  
_Print and keep at desk during Phase 4_  
_Detailed procedures: See PHASE4_EXECUTION_RUNBOOK.md_
