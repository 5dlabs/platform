# Repository Clone Logic Implementation

## Overview

The repository clone logic has been updated to intelligently handle multiple tasks on the same service without unnecessary cloning. This implementation addresses the user's requirement that "The only time we should ever really be cloning is during the first task."

## How It Works

### Service Tracking

The system now tracks which service is currently in the workspace using a `.current-service` file. This allows the init container to make smart decisions about when to clone.

### Decision Flow

```
1. Check if .current-service file exists
   - If NO: This is the first task → Clone repository
   - If YES: Read the service name
     
2. Compare current service with incoming service
   - If SAME: Continue with existing code → Skip clone
   - If DIFFERENT: Switch to new service → Clone new repository

3. Save the current service name for next task
```

### Implementation Details

#### 1. Template Data (taskrun.rs:669-675)
```rust
let data = json!({
    "task_id": tr.spec.task_id,
    "service_name": tr.spec.service_name,  // Added service_name
    "repository": tr.spec.repository.as_ref(),
    "export_script": export_script,
    "attempts": tr.status.as_ref().map_or(1, |s| s.attempts),
});
```

#### 2. Smart Clone Logic (init-container.sh.hbs:42-95)
The init container now includes sophisticated logic to determine when to clone:

```bash
# Check if we're continuing work on the same service
SKIP_CLONE=false
if [ -f "/workspace/.current-service" ]; then
    CURRENT_SERVICE=$(cat /workspace/.current-service)
    if [ "$CURRENT_SERVICE" = "{{service_name}}" ]; then
        echo "Continuing work on service: $CURRENT_SERVICE"
        echo "Using existing code from previous tasks"
        SKIP_CLONE=true
    else
        echo "Switching from service '$CURRENT_SERVICE' to '{{service_name}}'"
        # Backup previous service files
        mkdir -p "/workspace/.backup-${CURRENT_SERVICE}"
        find . -maxdepth 1 -not -name '.backup*' -not -name '.task' -not -name '.' \
            -exec mv {} "/workspace/.backup-${CURRENT_SERVICE}/" \; 2>/dev/null || true
    fi
fi

# Save current service for next task
echo "{{service_name}}" > /workspace/.current-service
```

### Key Features

1. **Service Continuity**: When working on the same service, the existing code is preserved and reused.

2. **Service Switching**: When switching to a different service, the old service files are backed up to `.backup-{service-name}` before cloning the new repository.

3. **Branch Verification**: When continuing with the same service, the system verifies we're on the correct branch and switches if needed.

4. **Remote URL Verification**: For new services, if a git repository already exists, the remote URL is verified to ensure it matches the expected repository.

### Benefits

1. **Faster Task Execution**: Subsequent tasks on the same service skip the clone step entirely.

2. **Preserved Local Changes**: Any uncommitted changes from previous tasks are preserved when working on the same service.

3. **No Merge Conflicts**: By avoiding unnecessary git operations, we eliminate potential merge conflicts.

4. **Network Efficiency**: Reduces network traffic by avoiding redundant clones.

5. **Clean Service Switching**: When switching services, the workspace is properly cleaned and the old service is backed up.

### File Management

The following files are added to `.gitignore` to prevent them from being committed:
- `.current-service` - Tracks the current service
- `.backup-*` - Backup directories for previous services

### Testing

To test the implementation:

1. **First Task**: Submit a task for a service. The repository should be cloned.
   ```bash
   orchestrator task submit 1 --service todo-api --repo https://github.com/org/todo-api
   ```

2. **Subsequent Task (Same Service)**: Submit another task for the same service. The clone should be skipped.
   ```bash
   orchestrator task submit 2 --service todo-api --repo https://github.com/org/todo-api
   ```

3. **Service Switch**: Submit a task for a different service. The old service should be backed up and the new one cloned.
   ```bash
   orchestrator task submit 3 --service auth-api --repo https://github.com/org/auth-api
   ```

### Monitoring

Look for these messages in the init container logs:

- `"Continuing work on service: todo-api"` - Indicates clone was skipped
- `"First task - no previous service found"` - Indicates first task
- `"Switching from service 'todo-api' to 'auth-api'"` - Indicates service switch

## Summary

This implementation provides intelligent repository management that:
- Only clones on the first task for a service
- Preserves code between tasks on the same service
- Handles service switching gracefully
- Reduces network usage and execution time
- Prevents unnecessary git conflicts

The logic is transparent to Claude agents - they simply work with whatever code is in the workspace, whether it was freshly cloned or preserved from previous tasks.