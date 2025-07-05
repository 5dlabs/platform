# TaskRun Context Preservation and Session Continuity

## Problem Statement

When a TaskRun fails, times out, or needs follow-up work, Claude should "pick up where it left off" rather than starting fresh. This is critical for:

- **Task retry scenarios**: Network timeouts, temporary failures, resource constraints
- **Multi-phase implementations**: Large tasks that span multiple job executions  
- **Iterative development**: Code review feedback, test failures requiring fixes
- **Progressive refinement**: Tasks that naturally require multiple rounds of work

## Current State vs Desired State

### Current Behavior
- Each TaskRun job starts Claude with a fresh conversation
- No memory of previous attempts or progress made
- Claude re-reads all context and starts from scratch
- Duplicated work and lost progress on retries

### Desired Behavior  
- Subsequent TaskRun executions continue the conversation thread
- Claude remembers what it already accomplished
- Incremental progress across multiple job runs
- Intelligent retry that builds on previous work

## Technical Challenge: Session Continuity

### The Session ID Problem
Claude Code sessions are identified by session IDs. For continuity, we need to:

1. **Capture session ID** from initial TaskRun execution
2. **Store session ID** in TaskRun status for future use
3. **Resume with session ID** on subsequent executions

### Continue vs Resume Decision

**Option 1: Always use `--continue`**
- Pros: Simple, automatic, no session ID management needed
- Cons: Relies on "most recent session" which may not be reliable in distributed environment
- Risk: Multiple concurrent TaskRuns could interfere with each other

**Option 2: Always use `--resume <session-id>`**  
- Pros: Explicit session targeting, reliable in multi-tenant environment
- Cons: Requires session ID storage and management
- Implementation: Store session_id in TaskRun.status.session_id

**Recommendation: Start with `--resume <session-id>`**
This gives us explicit control and avoids cross-TaskRun interference.

## Implementation Design

### Phase 1: Session ID Capture and Storage

1. **Initial execution**: Claude starts normally, we capture session ID from logs/output
2. **Session storage**: Store session_id in TaskRun.status.session_id  
3. **Retry execution**: Use `--resume <session_id>` to continue previous conversation

### Phase 2: Workspace State Preservation

Claude sessions may reference file states that need to be preserved:

1. **File versioning**: Snapshot workspace state after each execution
2. **Incremental changes**: Track what files Claude modified
3. **State restoration**: Ensure workspace matches Claude's memory on resume

### Phase 3: Context Version Management

Integrate with existing TaskRun context versioning:

1. **Context updates**: New markdown files should add to session, not restart it
2. **Version correlation**: Map context_version to session continuity
3. **Update vs retry**: Distinguish between "continue existing work" vs "add new requirements"

## CLI Integration

### New CLI Parameters

```bash
# For retry scenarios - continue existing work
orchestrator task retry <task-id> --continue-session

# For adding new context - resume with updates  
orchestrator task update <task-id> --context "additional requirements"
```

### TaskRun Controller Changes

```rust
// In build_agent_startup_script()
if let Some(session_id) = &tr.status.as_ref().and_then(|s| s.session_id) {
    args.push(format!("--resume={}", session_id));
} else {
    // First run - session ID will be captured from logs
}
```

## Session ID Extraction

### From Claude Output
Claude outputs session information in JSON format:
```json
{"type":"system","subtype":"init","session_id":"abc123-def456-..."}
```

We can parse this from container logs to extract and store the session ID.

### Storage in TaskRun Status
```yaml
status:
  session_id: "abc123-def456-789"  # New field
  attempts: 2
  last_updated: "2025-07-04T20:30:00Z"
```

## Testing Strategy

### Scenario 1: Simple Retry
1. Start TaskRun → capture session ID
2. Kill job mid-execution  
3. Retry TaskRun → should continue from where it left off

### Scenario 2: Timeout Recovery
1. TaskRun times out after partial progress
2. Retry should resume conversation and continue work

### Scenario 3: Context Addition
1. TaskRun completes initial work
2. Add new requirements via context update
3. Should continue existing session with new context

## Open Questions

1. **Session expiration**: How long do Claude sessions remain valid?
2. **Interactive resume**: Can `--resume` work non-interactively with session ID?
3. **Session conflicts**: What happens if session ID is invalid/expired?
4. **Workspace sync**: How do we ensure workspace state matches Claude's memory?

## Implementation Priority

1. ✅ **High**: Basic session ID capture and resume functionality
2. 🔄 **Medium**: CLI integration for retry/update scenarios  
3. ⏳ **Low**: Advanced workspace state management and conflict resolution

This focused approach addresses the core need: TaskRuns that can intelligently continue previous work rather than starting fresh each time.