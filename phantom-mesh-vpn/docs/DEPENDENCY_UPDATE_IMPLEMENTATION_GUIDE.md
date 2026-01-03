# P0-004: Dependency Update Implementation Guide

**Version:** 1.0  
**Date:** January 3, 2026  
**Status:** Active  
**Owner:** DevOps/Security Team

---

## 1. Overview

This guide provides step-by-step instructions for applying the recommended dependency updates identified in P0-003, with emphasis on security, testing, and documentation.

---

## 2. Current Dependency Status

### Immediate Actions Required

- ✅ **None** — All dependencies are secure
- ⏳ **Monitor:** x25519-dalek for 2.0.0 stable release

### Next Scheduled Actions

- **Monthly:** Security audit (cargo audit, pip-audit)
- **Quarterly:** Full dependency review
- **Annually:** Major version assessment

---

## 3. Update Procedures

### 3.1 Pre-Update Checklist

Before applying any update:

```
□ Review DEPENDENCY_ANALYSIS.md for current status
□ Check security advisories (NIST, OSV)
□ Read release notes of target version
□ Plan maintenance window
□ Notify stakeholders
□ Create git feature branch
□ Back up dependency files
```

### 3.2 Updating x25519-dalek (When 2.0.0 Stable Released)

**Step 1: Preparation**

```bash
cd phantom-mesh-vpn
git checkout -b feat/update-x25519-dalek-2.0.0
```

**Step 2: Create Backup**

```bash
cp Cargo.toml Cargo.toml.backup
cp Cargo.lock Cargo.lock.backup.lock
```

**Step 3: Update Dependency**

Edit `Cargo.toml`:

```toml
# Before
x25519-dalek = "2.0.0-rc3"

# After
x25519-dalek = "2.0.0"
```

**Step 4: Update Cargo Lockfile**

```bash
cargo update -p x25519-dalek
```

**Step 5: Verify Compilation**

```bash
cargo check --all-targets
```

**Step 6: Run Full Test Suite**

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Benchmarks (if applicable)
cargo bench --no-run
```

**Step 7: Run Quality Checks**

```bash
# Lint check
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Doc test
cargo test --doc
```

**Step 8: Performance Verification**

```bash
# Build release binary
cargo build --release

# Compare binary size (if significant)
ls -lh target/release/phantom-node
ls -lh target/release/libphantom_mesh.*
```

**Step 9: Documentation**

Create changelog entry in `CHANGELOG.d/`:

```markdown
# 2.0.0 Update - x25519-dalek

**Date:** $(date)  
**Executor:** [Your Name]  
**Version:** x25519-dalek 2.0.0

## Changes

- Updated x25519-dalek from 2.0.0-rc3 to 2.0.0 (stable)
- All tests passing
- No API breaking changes detected
- Binary size change: TBD

## Tests Performed

- ✅ cargo test --lib
- ✅ cargo test --test '\*'
- ✅ cargo clippy
- ✅ cargo fmt
- ✅ cargo doc

## Performance Impact

- Build time: [Before → After]
- Binary size: [Before → After]
- Runtime: No expected impact

## Risk Assessment

- Risk Level: LOW
- Breaking Changes: None detected
- Rollback Procedure: Revert Cargo.toml and `cargo update`

EOF
```

**Step 10: Commit Changes**

```bash
git add Cargo.toml Cargo.lock
git add docs/DEPENDENCY_UPDATES.md
git commit -m "chore(deps): update x25519-dalek to 2.0.0 stable

- Update from RC version to stable release
- All tests passing
- No breaking changes"
```

**Step 11: Create Pull Request**

```bash
git push origin feat/update-x25519-dalek-2.0.0
# Create PR with security team review required
```

**Step 12: Deployment**

After PR approval:

```bash
git checkout main
git merge --ff-only feat/update-x25519-dalek-2.0.0
git push origin main

# Tag release
git tag -a v0.1.1 -m "Update x25519-dalek to 2.0.0 stable"
git push origin v0.1.1
```

---

### 3.3 Security Patch Updates (Emergency Process)

When a critical CVE is discovered:

```bash
# 1. Create emergency branch
git checkout -b hotfix/cve-2026-XXXXX

# 2. Update affected package
cargo update -p <package>

# 3. Run verification
bash scripts/updates/p0-004-apply-updates.sh
python scripts/updates/verify_updates.py

# 4. Expedited review (skip normal review)
git commit -m "security: patch CVE-2026-XXXXX in <package>"
git push origin hotfix/cve-2026-XXXXX

# 5. Fast-track merge and deploy
git checkout main
git merge --ff-only hotfix/cve-2026-XXXXX
git push origin main

# 6. Deploy to production
# (Follow your deployment procedure)
```

---

### 3.4 Routine Monthly Updates

```bash
# 1. Run security audit
cargo audit > audit_results.txt
pip-audit > audit_results_python.txt

# 2. Check for outdated packages
cargo outdated --root-deps-only
pip list --outdated

# 3. If updates available and low-risk
cargo update
pip install --upgrade -r requirements.txt

# 4. Run full test suite
cargo test --all
pytest tests/ -v

# 5. Verify and document
python scripts/updates/verify_updates.py
git add -A
git commit -m "chore(deps): monthly dependency updates"
```

---

## 4. Verification Procedures

### 4.1 Automated Verification

Run the verification script:

```bash
cd phantom-mesh-vpn
python scripts/updates/verify_updates.py
```

**Expected Output:**

```
cargo_check: ✅ PASS
cargo_test: ✅ PASS
cargo_clippy: ✅ PASS
cargo_fmt: ✅ PASS
python_imports: ✅ PASS
dependencies: ✅ PASS

Overall Status: PASS
```

### 4.2 Manual Verification

Key tests to verify:

1. **Build Verification**

   ```bash
   cargo build --release
   cargo build --release --all-features
   ```

2. **Test Verification**

   ```bash
   cargo test --lib --all
   cargo test --test '*'
   cargo test --doc
   ```

3. **Performance Verification**

   ```bash
   cargo bench --no-run
   # Compare against baseline if critical
   ```

4. **Security Verification**
   ```bash
   cargo clippy -- -D warnings
   cargo audit
   pip-audit
   ```

---

## 5. Rollback Procedures

If update causes issues:

### 5.1 Immediate Rollback

```bash
# Option 1: Git rollback (if not yet deployed)
git revert <commit-hash>
git push origin main

# Option 2: Dependency rollback
# Edit Cargo.toml to previous version
# Run cargo update
# Re-test and push
```

### 5.2 Production Rollback

```bash
# Deploy previous stable version
git checkout <previous-tag>
# Follow deployment procedure
# Mark incident in postmortem
```

### 5.3 Post-Rollback

1. Investigate root cause
2. File issue if it's a dependency bug
3. Wait for fix or alternative solution
4. Document lessons learned

---

## 6. Documentation Requirements

After each update, document:

1. **What Changed**

   - Old versions → New versions
   - Release notes summary
   - Breaking changes (if any)

2. **Why It Changed**

   - Security patches
   - Performance improvements
   - Feature additions
   - Bug fixes

3. **How It Was Tested**

   - Test results
   - Performance impact
   - Risk assessment

4. **Deployment Details**
   - Date/time deployed
   - Who deployed it
   - Any issues encountered

---

## 7. Monitoring Integration

### 7.1 GitHub Actions Integration

Create `.github/workflows/dependency-check.yml`:

```yaml
name: Dependency Security Check
on: [push, pull_request]
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run cargo audit
        run: cargo audit
      - name: Run pip-audit
        run: pip-audit
```

### 7.2 Scheduled Audits

Create `.github/workflows/weekly-audit.yml`:

```yaml
name: Weekly Dependency Audit
on:
  schedule:
    - cron: "0 0 * * 0" # Sunday midnight
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run full audit
        run: bash scripts/audits/p0-003-dependency-audit.sh
      - name: Upload results
        uses: actions/upload-artifact@v3
```

---

## 8. Escalation Procedures

### 8.1 Critical CVE (CVSS >= 8.0)

1. **Alert:** Immediate notification to security team
2. **Assessment:** Within 15 minutes
3. **Patch:** Within 1 hour
4. **Deploy:** Within 4 hours
5. **Verify:** Continuous monitoring

### 8.2 High CVE (CVSS 7.0-7.9)

1. **Alert:** Within 1 hour
2. **Assessment:** Within 4 hours
3. **Patch:** Within 1 week
4. **Deploy:** Within 2 weeks

### 8.3 Medium CVE (CVSS 4.0-6.9)

1. **Alert:** Within 1 day
2. **Assessment:** Within 1 week
3. **Patch:** Within 1 month
4. **Deploy:** Within 1-3 months

---

## 9. Team Responsibilities

| Role              | Responsibility                           |
| ----------------- | ---------------------------------------- |
| **Developer**     | Apply updates, run tests, commit changes |
| **Code Reviewer** | Review changes, verify test results      |
| **Security Team** | Assess CVEs, approve security patches    |
| **DevOps**        | Deploy updates, monitor production       |
| **Tech Lead**     | Coordinate updates, resolve conflicts    |

---

## 10. Communication Template

When deploying updates:

```
Subject: Dependency Update Deployment - [Date]

Team,

We have deployed the following dependency updates:

Updates Applied:
- Package X: Version A → Version B
- Package Y: Version C → Version D

Testing:
- All unit tests: PASSED
- Integration tests: PASSED
- Security audit: PASSED

Impact:
- No breaking changes
- [Performance change if significant]
- [Known issues if any]

Rollback:
- If issues occur, we can rollback to [previous version]
- Rollback time: ~[5] minutes

Questions: Contact [Team Lead]
```

---

## 11. Quick Reference

### Key Commands

```bash
# Check for vulnerabilities
cargo audit
pip-audit

# Check for outdated packages
cargo outdated
pip list --outdated

# Update dependencies
cargo update
pip install --upgrade -r requirements.txt

# Verify installation
python scripts/updates/verify_updates.py

# Apply all updates
bash scripts/updates/p0-004-apply-updates.sh
```

### Important Files

- `Cargo.toml` — Rust dependencies
- `Cargo.lock` — Rust lock file
- `pyproject.toml` — Python project config
- `requirements.txt` — Python dependencies
- `scripts/updates/` — Update scripts
- `docs/DEPENDENCY_*.md` — Documentation

---

## 12. Approval & Review

**Prepared by:** DevOps/Security Team  
**Date:** January 3, 2026  
**Status:** Active

**Last Review:** January 3, 2026  
**Next Review:** April 3, 2026

---

_This guide ensures safe, documented, and reversible dependency updates._
