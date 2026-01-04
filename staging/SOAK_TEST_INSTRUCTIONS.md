# 72-HOUR SOAK TEST LAUNCH INSTRUCTIONS
# =====================================

## To Start the Soak Test:

```powershell
# Option 1: Run in foreground (recommended for monitoring)
& "S:\PhantomMesh-VPN\staging\soak-test-monitor.ps1"

# Option 2: Run as background job
Start-Job -FilePath "S:\PhantomMesh-VPN\staging\soak-test-monitor.ps1"

# Option 3: Run in Docker staging environment
docker-compose -f docker-compose.staging.yml --profile soak up -d soak-tester
```

## Monitoring During Test:

- Prometheus: http://localhost:25540
- Check staging logs: docker-compose -f docker-compose.staging.yml logs -f
- View results: cat S:\PhantomMesh-VPN\staging\soak-test-results.json

## Success Criteria:

- Memory growth < 5%
- Error rate < 0.1%
- P99 latency < 200ms
- 0 pod restarts

## Stop Test Early:

```powershell
# Stop background job
Get-Job | Stop-Job | Remove-Job

# Or stop Docker soak tester
docker-compose -f docker-compose.staging.yml --profile soak down
```
