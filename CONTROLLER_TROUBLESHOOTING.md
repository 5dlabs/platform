# Controller Troubleshooting Analysis

## Problem Summary

**Issue**: DocsRun controller is processing resources (creates ConfigMaps, adds finalizers) but does NOT create Kubernetes Jobs, despite appearing to reconcile successfully.

**Symptoms**:
- ✅ DocsRun CRD applied successfully
- ✅ ConfigMap created with comprehensive container script
- ✅ Finalizer added to DocsRun resource
- ❌ No Kubernetes Job created
- ❌ No logs showing job creation attempt
- ❌ Hard to delete DocsRuns (stuck finalizers)
- ❌ No error messages in controller logs

## Changes Made Today vs. Yesterday

### 1. **Controller-Based Cleanup Logic** (HIGH SUSPICION)

**What changed**: Added event-driven cleanup with background spawned tasks

```rust
// NEW: Added cleanup configuration
pub struct CleanupConfig {
    pub enabled: bool,
    pub completed_job_delay_minutes: u64,
    pub failed_job_delay_minutes: u64,
    pub delete_configmap: bool,
}

// NEW: Background cleanup spawning in status.rs
pub fn schedule_job_cleanup(/* ... */) -> Result<()> {
    tokio::spawn(async move {
        // Background cleanup logic
    });
    Ok(())
}

// NEW: Cleanup called during reconciliation
monitor_job_status(/* ... */).await?;
```

**Risk Analysis**:
- ⚠️ **Background spawn might be blocking reconciliation**
- ⚠️ **Cleanup logic errors not visible in main thread logs**
- ⚠️ **Finalizer management might be interfering with normal flow**

### 2. **Configuration Loading Method** (MEDIUM SUSPICION)

**What changed**: Switched from async API calls to sync file reading

```rust
// OLD (working):
ControllerConfig::from_configmap(client, &namespace, "config-name").await?

// NEW (current):
ControllerConfig::from_mounted_file("/config/config.yaml")
```

**Risk Analysis**:
- ⚠️ **File permission issues**
- ⚠️ **Sync vs async context problems**
- ⚠️ **Different error handling paths**

### 3. **Serde Configuration** (LOW SUSPICION - FIXED)

**What changed**: Added `#[serde(default)]` and `Default` trait for `CleanupConfig`

```rust
// FIXED: This was causing panics, now resolved
pub struct ControllerConfig {
    #[serde(default)] // ADDED
    pub cleanup: CleanupConfig,
}

impl Default for CleanupConfig { /* ADDED */ }
```

**Status**: ✅ **RESOLVED** - This was causing the "resource name may not be empty" error

### 4. **ConfigMap Template References** (LOW SUSPICION - FIXED)

**What changed**: Fixed empty ConfigMap name references in Helm templates

```yaml
# FIXED: Was causing installation failures
name: {{ include "orchestrator.fullname" . }}-task-controller-config
```

**Status**: ✅ **RESOLVED** - Controller now starts properly

## Evidence Analysis

### What's Working ✅

1. **Controller Startup**: Logs show successful initialization
   ```
   INFO orchestrator_core: Starting task controller in namespace: orchestrator
   INFO kube_runtime::controller: press ctrl+c to shut down gracefully (x2)
   ```

2. **Resource Processing**: ConfigMap creation proves controller is processing DocsRuns
   ```
   configmap/docs-generator-docs-v1-files created (5 files, comprehensive content)
   ```

3. **Finalizer Management**: DocsRun has finalizer indicating controller interaction
   ```yaml
   finalizers:
   - docsruns.orchestrator.io/finalizer
   ```

### What's Missing ❌

1. **Job Creation**: No Kubernetes Jobs created despite ConfigMap preparation
2. **Creation Logs**: No logs indicating job creation attempts
3. **Error Logs**: No visible errors in controller logs
4. **Status Updates**: DocsRun status field remains empty

### Critical Observations

1. **Silent Failure**: Controller processes up to ConfigMap creation, then stops
2. **Background Tasks**: Cleanup logic spawns background tasks that might hide errors
3. **Log Visibility**: Background spawned tasks don't show logs in main controller thread
4. **Finalizer Stuck**: Difficult to delete DocsRuns suggests cleanup logic issues

## Hypothesis: Background Cleanup Interference

**Primary Theory**: The new cleanup logic is interfering with normal reconciliation flow.

### Supporting Evidence:
- ConfigMap created (early reconciliation step) ✅
- Job creation missing (later reconciliation step) ❌
- Background `tokio::spawn` in reconciliation path
- Finalizer management complexity

### Potential Issues:
1. **Panic in background task** preventing job creation
2. **Resource contention** between main thread and cleanup thread
3. **Error swallowing** in spawned cleanup tasks
4. **Finalizer logic** blocking normal flow

## Code Path Analysis

### Normal Flow (Expected):
```
DocsRun Created → Reconcile → Create ConfigMap → Create Job → Update Status → Add Finalizer
```

### Current Flow (Observed):
```
DocsRun Created → Reconcile → Create ConfigMap → ??? → Add Finalizer → STOP
```

### Suspected Issue Location:
The gap between "Create ConfigMap" and "Create Job" - likely in the resource creation logic where cleanup scheduling happens.

## Testing Strategy

### 1. **Isolate Cleanup Logic**
- Temporarily disable cleanup entirely
- Test if Jobs are created without cleanup interference

### 2. **Add Targeted Logging**
- Add logs immediately before job creation
- Add logs in cleanup spawning logic
- Use `tracing::error!` for all potential failure points

### 3. **Check Resource Creation Path**
- Examine `reconcile_create_or_update` function
- Look for cleanup calls that might be blocking
- Verify job creation code path is reached

### 4. **Test Without Background Spawning**
- Replace `tokio::spawn` with direct cleanup calls
- See if removing async spawning resolves the issue

## Action Plan

1. **Immediate**: Disable cleanup logic to test core functionality
2. **Debug**: Add comprehensive logging around job creation
3. **Isolate**: Test each component change individually
4. **Verify**: Confirm job creation works without cleanup

## Files to Examine

### Primary Suspects:
- `orchestrator/core/src/controllers/task_controller/status.rs` - Cleanup logic
- `orchestrator/core/src/controllers/task_controller/resources.rs` - Job creation
- `orchestrator/core/src/controllers/task_controller/reconcile.rs` - Main flow

### Configuration:
- `orchestrator/core/src/controllers/task_controller/config.rs` - Config loading
- `infra/charts/orchestrator/templates/task-controller-config.yaml` - Config source

## Next Steps

**Phase 1**: Disable cleanup and test
**Phase 2**: Add comprehensive logging
**Phase 3**: Test each change in isolation
**Phase 4**: Identify root cause and implement proper fix

---

*This analysis suggests the background cleanup logic is the most likely culprit, followed by configuration loading changes.*