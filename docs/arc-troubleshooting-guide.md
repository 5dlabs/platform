# ARC Troubleshooting Guide

## Current Status: ✅ WORKING

After deep investigation, ARC is functioning correctly. Jobs are being picked up and executed successfully.

## Understanding Ephemeral Runners

### What's Happening
1. **Ephemeral Mode**: `RUNNER_FEATURE_FLAG_EPHEMERAL=true`
2. **Behavior**: Runners are single-use and restart after each job
3. **Visual Effect**: Makes it seem like runners are unstable or not working

### Timeline of a Job
```
T+0s:   Job queued → Runner picks it up
T+30s:  Job completes → Runner terminates
T+35s:  New runner pod created to maintain replica count
T+60s:  New runner registers with GitHub and starts listening
```

## Common "Issues" That Are Actually Normal

### 1. "Runners keep disappearing"
**Cause**: Ephemeral runners terminate after completing jobs
**Solution**: This is expected behavior. Check `kubectl get runners -n arc-systems` to see new runners

### 2. "kubectl/helm not found"
**Cause**: PATH environment variable not propagated correctly
**Solution**: Use `export PATH="/shared:$PATH"` in each workflow step

### 3. "Black screen in GitHub Actions UI"
**Cause**: Sometimes occurs when runners are initializing
**Solution**: Wait 30-60 seconds for runner to fully start

### 4. "Runners show offline in GitHub"
**Cause**: Old ephemeral runners that completed jobs
**Solution**: GitHub will clean these up automatically

## Debugging Commands

### Check Runner Status
```bash
# See all runners
kubectl get runners -n arc-systems

# Check runner pods
kubectl get pods -n arc-systems -l runner-deployment-name=platform-runner-deployment

# View runner logs
kubectl logs -n arc-systems -l runner-deployment-name=platform-runner-deployment --tail=50

# Check GitHub registration
gh api repos/5dlabs/platform/actions/runners --jq '.runners[] | {name: .name, status: .status}'
```

### Monitor Job Pickup
```bash
# Watch for job activity in real-time
kubectl logs -n arc-systems -l runner-deployment-name=platform-runner-deployment -f | grep -E "(Job|Running|Listening)"
```

### Check ARC Controller
```bash
# Controller logs
kubectl logs -n arc-systems deployment/arc-actions-runner-controller -c manager --tail=100

# Check for errors
kubectl logs -n arc-systems deployment/arc-actions-runner-controller -c manager | grep ERROR
```

## Configuration Options

### Option 1: Keep Ephemeral (Current)
**Pros**: Clean environment for each job, more secure
**Cons**: Runner cycling can be confusing, slight delay between jobs

### Option 2: Disable Ephemeral
```yaml
env:
  - name: RUNNER_FEATURE_FLAG_EPHEMERAL
    value: "false"
```
**Pros**: Runners stay running, faster job pickup
**Cons**: State can persist between jobs, less secure

## Working CI/CD Configuration

### Current Working Setup
1. Deploy job runs on `[self-hosted, k8s-runner]`
2. Each step exports PATH: `export PATH="/shared:$PATH"`
3. Tools available at `/shared/kubectl` and `/shared/helm`

### Alternative: Use GitHub Path
```yaml
- name: Setup tools
  run: |
    mkdir -p $HOME/bin
    ln -s /shared/kubectl $HOME/bin/kubectl
    ln -s /shared/helm $HOME/bin/helm
    echo "$HOME/bin" >> $GITHUB_PATH
```

## Metrics to Confirm ARC is Working

1. **Runner Count**: Should match replica count (currently 2)
   ```bash
   kubectl get runners -n arc-systems | grep Running | wc -l
   ```

2. **Job Success Rate**: Check recent workflows
   ```bash
   gh run list --limit 10 --json conclusion | jq '[.[] | select(.conclusion == "success")] | length'
   ```

3. **Runner Uptime**: For non-ephemeral runners
   ```bash
   kubectl get pods -n arc-systems -o jsonpath='{.items[*].metadata.name} {.items[*].status.startTime}'
   ```

## When to Actually Worry

Only investigate further if:
1. ❌ No runners show as "Running" in `kubectl get runners`
2. ❌ Workflows stay "Queued" for more than 5 minutes
3. ❌ Controller shows authentication errors
4. ❌ Runner pods are in CrashLoopBackOff
5. ❌ GitHub shows 0 online runners for extended periods

## Quick Health Check Script

```bash
#!/bin/bash
echo "=== ARC Health Check ==="
echo "Runners: $(kubectl get runners -n arc-systems | grep Running | wc -l) running"
echo "Pods: $(kubectl get pods -n arc-systems | grep platform-runner | grep Running | wc -l) running"
echo "GitHub: $(gh api repos/5dlabs/platform/actions/runners --jq '[.runners[] | select(.status == "online")] | length') online"
echo "Recent jobs: $(gh run list --limit 5 --json status --jq '[.[] | select(.status == "completed")] | length') completed of last 5"
```

## Conclusion

ARC is working correctly. The ephemeral runner behavior can make it appear unstable, but this is by design for security and cleanliness. Jobs are being picked up and executed successfully.