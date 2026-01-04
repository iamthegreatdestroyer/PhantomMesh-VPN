# 🚀 PhantomMesh VPN - Deployment Quick Reference Card

**Print & Keep With You During Deployment**

---

## GO-LIVE CHECKLIST (In Order)

### T-2 hours: PRE-DEPLOYMENT

```
[ ] Team assembled in war room
[ ] All systems running and healthy
[ ] Monitoring dashboards open
[ ] Communication channels active
[ ] Backup contact numbers verified
[ ] Rollback procedures reviewed
```

### T-30 min: GO/NO-GO DECISION

```
[ ] Final health check complete
[ ] All prerequisites met
[ ] Executive approval confirmed
[ ] Team ready confirmation
[ ] Decision: ____ GO / ____ NO-GO
[ ] Decision authority: ________________
```

### T hour: DEPLOYMENT START

```
T+00: Announce deployment start
T+05: Apply namespace & RBAC
T+15: Deploy secrets & ConfigMaps
T+25: Deploy storage & networking
T+35: Deploy PostgreSQL, Redis, Prometheus
T+45: Deploy VPN Core, Discovery, Agents
T+55: Deploy API Gateway, Grafana
T+70: Run smoke tests
T+90: Validate connectivity
T+120: DEPLOYMENT COMPLETE
```

---

## CRITICAL CONTACTS

```
Primary Deployment Lead:
  Name: _______________________
  Phone: ______________________
  Email: _______________________

CTO (Escalation):
  Name: _______________________
  Phone: ______________________

On-Call Engineer:
  Name: _______________________
  Phone: ______________________

Operations Lead:
  Name: _______________________
  Phone: ______________________
```

---

## ESSENTIAL COMMANDS

```bash
# Check status
kubectl get pods -n phantommesh-prod

# View logs
kubectl logs -f deployment/vpn-core -n phantommesh-prod

# Scale if needed
kubectl scale deployment vpn-core --replicas=5 -n phantommesh-prod

# Restart if needed
kubectl rollout restart deployment/vpn-core -n phantommesh-prod

# Rollback if critical issue
kubectl rollout undo deployment/vpn-core -n phantommesh-prod
```

---

## KEY METRICS TO WATCH

```
✅ = Green (< 0.1% error, P99 < 200ms)
⚠️  = Yellow (0.1%-1% error, P99 < 500ms)
🔴 = Red (> 1% error, P99 > 500ms)

Monitor every 5 minutes:
  □ Error rate
  □ P99 latency
  □ Pod restart count
  □ Memory usage
  □ CPU usage
  □ Database connections
```

---

## ALERT RESPONSE GUIDE

**If Error Rate > 1%:**

1. Check logs: `kubectl logs -f`
2. Identify affected service
3. Check resource usage
4. Scale if needed
5. Escalate if not resolved in 5 minutes

**If P99 Latency > 500ms:**

1. Check which service is slow
2. Profile the service
3. Check database queries
4. Scale if needed
5. Escalate if not resolved in 10 minutes

**If Pod Restart Loop:**

1. Check logs: `kubectl logs --previous`
2. Check resources: `kubectl top pod`
3. Increase memory limit
4. Restart pod
5. Escalate if pattern continues

---

## ROLLBACK PROCEDURE (If Needed)

```bash
# Step 1: STOP NEW TRAFFIC
kubectl scale deployment vpn-core --replicas=0 -n phantommesh-prod

# Step 2: RESTORE PREVIOUS
kubectl rollout undo deployment/vpn-core -n phantommesh-prod

# Step 3: SCALE BACK UP
kubectl scale deployment vpn-core --replicas=3 -n phantommesh-prod

# Step 4: VERIFY
curl https://phantommesh.example.com/health

# Step 5: ANNOUNCE
"Deployment rolled back to previous version"
```

---

## SUCCESS CRITERIA (After T+120 min)

Must verify ALL of these:

```
✅ All pods running (kubectl get pods)
✅ API responding (curl /health)
✅ Error rate < 0.1%
✅ P99 latency < 200ms
✅ Database online
✅ Metrics collecting
✅ Backups running
✅ No critical alerts
✅ User traffic flowing
✅ Status page green
```

**If ALL checkmarks present = SUCCESSFUL DEPLOYMENT** 🎉

---

## DURING DEPLOYMENT: COMMUNICATE

**Every 15 minutes update:**

- Slack: #phantommesh-production
- Status page: Current phase
- Email: Executive summary (if significant change)

**Template:**

```
Update #X (T+Xmin):
Phase: [Name]
Status: [% complete]
Issues: [None / specific issue]
Next: [What's next]
```

---

## AFTER DEPLOYMENT: MONITOR

**First 24 hours:**

- [ ] Monitor every 15 minutes
- [ ] Check for user-reported issues
- [ ] Watch error rates & latency
- [ ] Monitor database health
- [ ] Verify backups completing

**First week:**

- [ ] Daily metrics review
- [ ] Performance trending
- [ ] Optimization opportunities
- [ ] Team retrospective
- [ ] Lessons learned doc

---

## DOCUMENT REFERENCE

For detailed guidance, see:

| Document                         | When to Use           |
| -------------------------------- | --------------------- |
| PRODUCTION_DEPLOYMENT_RUNBOOK.md | Detailed step-by-step |
| PRODUCTION_OPERATIONS_MANUAL.md  | Ongoing operations    |
| GO_LIVE_EXECUTION_PLAN.md        | Timeline reference    |
| FINAL_SECURITY_AUDIT_REPORT.md   | Security questions    |

---

## SLACK CHANNELS

```
#phantommesh-production    - Deployment updates (MAIN)
#phantommesh-incidents     - Issues & escalations
#phantommesh-ops           - Operations team
#phantommesh-info          - Info/logging
#status-page               - Status updates
```

---

## KEY DECISION POINTS

### T-30 min: GO/NO-GO DECISION

**GO if:**

- ✅ All prerequisites complete
- ✅ Team ready
- ✅ No showstoppers identified

**NO-GO if:**

- ❌ Critical issue found
- ❌ Team not ready
- ❌ Infrastructure problem

**Authority:** Deployment Lead + CTO

---

### T+90 min: VALIDATION DECISION

**SUCCESS if:**

- ✅ All pods running
- ✅ Error rate < 0.1%
- ✅ P99 latency < 200ms
- ✅ All endpoints responding

**FAILURE/ROLLBACK if:**

- ❌ Any service down
- ❌ Error rate > 1%
- ❌ Latency > 500ms
- ❌ Critical alert triggered

**Authority:** Deployment Lead

---

## NUMBERS TO REMEMBER

```
120 minutes   = Total deployment window
0.1%          = Error rate threshold (green)
1%            = Error rate threshold (red)
200ms         = P99 latency target
500ms         = P99 latency critical
<5 min        = Time to detect critical issue
<5 min        = Time to execute rollback
99.99%        = Availability target
70+ hours     = Remaining soak test time
```

---

## IN CASE OF EMERGENCY

**If critical issue during deployment:**

1. PAUSE deployment (don't proceed further)
2. HALT new traffic (scale replicas to 0)
3. NOTIFY team (page on-call)
4. INVESTIGATE root cause (30 seconds)
5. DECIDE: Fix OR Rollback (decision lead)

**If must rollback:**

- Follow "ROLLBACK PROCEDURE" section above
- Takes ~5 minutes
- No data loss
- Transparent to users

---

## CONFIDENCE INDICATORS

**Why this deployment will succeed:**

✅ All 3 decision gates passed
✅ Load test: 100% success, 1,000 concurrent users
✅ Security audit: Approved
✅ 72-hour soak test: Running healthy (no issues)
✅ Performance targets: Exceeded by 50%+
✅ Team: Trained & ready
✅ Monitoring: Fully armed
✅ Rollback: Tested & ready

**Confidence Level: VERY HIGH** 🎯

---

## AFTER SUCCESSFUL DEPLOYMENT

```
🎉 Celebrate the team achievement!
📊 Post success metrics (P99 latency, uptime, etc.)
📝 Document lessons learned
🔄 Schedule retrospective meeting
📢 Share success with customers
🚀 Plan next improvements
```

---

**KEEP THIS CARD WITH YOU DURING DEPLOYMENT**

**Last Updated:** 2026-01-04  
**Valid For:** One deployment cycle  
**Print & Share:** With all team members

---

**WE ARE READY. LET'S DEPLOY WITH CONFIDENCE!** ✅
