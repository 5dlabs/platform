# TaskRun Context Preservation and Session Continuity

## Overview

The core challenge: When a TaskRun fails, times out, or needs follow-up work, Claude should "pick up where it left off" rather than starting fresh. This requires preserving conversation context across multiple job executions for the same logical task.

## Command Definitions

### `--continue` (`-c`)
- **Purpose**: Continue the most recent conversation
- **Behavior**: Automatically resumes the last active Claude session without user selection
- **Session Selection**: Automatic (most recent)
- **Use Case**: When you know you want to continue exactly where you left off

### `--resume` (`-r`)
- **Purpose**: Resume a specific conversation by session ID or interactive selection
- **Behavior**: Either accepts a specific session ID or presents an interactive menu to choose from available sessions
- **Session Selection**: Manual (specific session ID or user choice)
- **Use Case**: When you need to resume a specific previous session or choose from multiple options

## Use Case Scenarios

### When to Use `--continue`

1. **Simple Task Continuation**
   - Task was interrupted (network issue, timeout, manual stop)
   - You want to pick up immediately where Claude left off
   - There's only one recent session and you want to continue it
   - No ambiguity about which session to resume

2. **Development Workflow**
   - Working on a single task/feature continuously
   - Brief interruptions that don't require context switching
   - Iterative development where each session builds on the previous

3. **Error Recovery**
   - Claude encountered a temporary error
   - System restart or container restart
   - Quick reconnection after infrastructure issues

### When to Use `--resume`

1. **Multiple Concurrent Tasks**
   - Working on multiple different tasks/features
   - Need to switch between different contexts
   - Multiple TaskRuns for the same service with different sessions

2. **Selective Session Recovery**
   - Multiple previous sessions exist
   - Need to resume a specific session (not the most recent)
   - Want to review session history before choosing

3. **Long-term Task Management**
   - Returning to a task after working on other tasks
   - Need to resume from a specific checkpoint/milestone
   - Task spans multiple days/sessions with different contexts

4. **Collaborative Workflows**
   - Multiple team members working on same service
   - Need to resume a session started by someone else
   - Shared task management with specific session tracking

## Orchestrator Integration Implications

### TaskRun Context Preservation

**Continue Scenario:**
- TaskRun should maintain session continuity
- File changes should be preserved in workspace
- Context version increments should be minimal
- Retry logic should use `--continue` for simple restarts

**Resume Scenario:**
- TaskRun may need to track multiple session IDs
- Context preservation becomes more complex
- May require backup/merge strategies for file conflicts
- Update vs retry distinction becomes critical

### CLI Implementation Requirements

**For `--continue`:**
- Simple flag addition to Claude execution
- Should work with existing retry mechanisms
- Minimal additional state management needed

**For `--resume`:**
- Requires session ID tracking in TaskRun status
- May need interactive session selection (if no ID provided)
- Complex state management for multiple sessions
- Integration with TaskRun context versioning

## Recommended Implementation Strategy

### Phase 1: Continue Support
1. Add `--continue` flag to TaskRun controller
2. Implement for retry scenarios
3. Test with simple task interruption/continuation

### Phase 2: Resume Support  
1. Add session ID tracking to TaskRun status
2. Implement session ID persistence across attempts
3. Add CLI parameter for specific session resume
4. Handle interactive session selection

### Phase 3: Advanced Context Management
1. Implement backup/merge strategies for file conflicts
2. Add distinction between retry (continue) vs update (resume)
3. Multi-session support for complex workflows

## Decision Matrix

| Scenario | Use Continue | Use Resume | Rationale |
|----------|-------------|------------|-----------|
| Task timeout/restart | ✅ | ❌ | Simple continuation |
| Manual task stop | ✅ | ❌ | Resume recent work |
| Switch between tasks | ❌ | ✅ | Need specific session |
| Multiple concurrent tasks | ❌ | ✅ | Avoid session confusion |
| Error recovery | ✅ | ❌ | Quick reconnection |
| Collaborative work | ❌ | ✅ | Specific session needed |
| Long-term task management | ❌ | ✅ | Checkpoint selection |
| Context switching | ❌ | ✅ | Multiple contexts |

## Technical Implementation Notes

- Continue: Simpler to implement, should be prioritized
- Resume: More complex, requires session management infrastructure
- Both should integrate with existing TaskRun retry/update mechanisms
- Session ID persistence will be key for resume functionality
- Interactive session selection may require special handling in containerized environment