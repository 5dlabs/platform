# TaskRun Context Preservation with --continue

## Simplified Approach

**Confirmed**: `--continue` flag works automatically without user interaction. This eliminates the need for complex session ID management.

## Implementation Strategy

### Core Concept
- **First attempt**: Claude starts normally, conversation saved automatically
- **Retry attempts**: Add `--continue` flag to pick up previous conversation
- **Directory-based isolation**: Each TaskRun works in task-specific directory to prevent collisions

### Directory Structure for Isolation
```
/workspace/{service}/.task/{task_id}/
```

This ensures:
- Task 123 conversations stored in `/workspace/debug-api/.task/123/`
- Task 124 conversations stored in `/workspace/debug-api/.task/124/`
- No collision between concurrent agents on same service
- Conversation continuity preserved across multiple attempts for same task

### Implementation

**TaskRun Controller Changes**:
```rust
// In build_agent_startup_script()
let attempts = tr.status.as_ref().map(|s| s.attempts).unwrap_or(0);
if attempts > 1 {
    args.push("--continue".to_string());
}
```

**Claude Working Directory**:
```rust
"workingDir": format!("/workspace/{}", tr.spec.service_name)
// Task files stored in: /workspace/{service}/.task/{task_id}/
// Claude conversations persist in workingDir for --continue
```

### Benefits of This Approach

✅ **No session ID management** - Claude handles it automatically
✅ **No storage complexity** - Conversations persist in workspace PVC  
✅ **Automatic collision avoidance** - Task-specific directories
✅ **Simple retry logic** - Just add `--continue` flag
✅ **Works with existing infrastructure** - Uses current PVC setup

### Collision Prevention

**Current Protection**:
- Each task gets unique directory: `/workspace/{service}/.task/{task_id}/`
- Claude conversations are directory-specific
- Multiple agents on same service work in different task directories

**Monitoring Strategy**:
- Watch for unexpected conversation mixing in logs
- Verify `--continue` picks up correct previous conversation
- Create issue if collisions occur with multiple concurrent agents

### Testing Plan

1. **Basic continuation**: Start task → kill → retry with `--continue`
2. **Progress preservation**: Verify Claude remembers previous work
3. **Directory isolation**: Run multiple tasks on same service simultaneously
4. **Conversation integrity**: Ensure `--continue` picks up right conversation

### Implementation Priority

**Phase 1** (COMPLETED):
- ✅ Add `--continue` flag for retry attempts (attempts > 1)
- ✅ Remove `/run-{attempt}/` subdirectory structure  
- ✅ Ensure Claude working directory enables conversation persistence
- ✅ Increment attempts counter on job creation

**Phase 2** (Future):
- Monitor for collision issues with multiple concurrent agents
- Optimize directory structure if needed
- Add safeguards if collision problems emerge

## Technical Notes

- **Conversation storage**: Claude stores transcripts locally per directory
- **Retention**: Default 30 days (`cleanupPeriodDays` setting)
- **Persistence**: Conversations survive container restarts via PVC
- **Isolation**: Directory-based separation prevents cross-task interference

This approach dramatically reduces complexity while providing the core functionality needed for TaskRun context preservation.