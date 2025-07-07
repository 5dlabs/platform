# Per-Attempt Subdirectories Implementation

## Summary
Implemented subdirectories for each task attempt to preserve history across retries. Each attempt now gets its own directory at `.task/{task_id}/attempt-{n}/`.

## Changes Made

### 1. Updated init-container.sh.hbs
- Changed from: `mkdir -p /workspace/.task/{{task_id}}`
- Changed to: `mkdir -p /workspace/.task/{{task_id}}/attempt-{{attempts}}`

- Task files are now copied to the attempt-specific directory
- Backward compatibility maintained by also copying markdown files to task root
- Updated .gitignore patterns to allow session exports from attempt subdirectories

### 2. Updated taskrun.rs Controller
- Added `attempts` field to export script data context
- Export script now receives both `task_id` and `attempts` parameters

### 3. Updated export-session.sh Template
- Changed output directory from `/workspace/.task/${TASK_ID}/`
- Changed to: `/workspace/.task/${TASK_ID}/attempt-${ATTEMPT}/`
- Session exports now go to attempt-specific directories

## Directory Structure Example
```
/workspace/.task/
└── 9999/
    ├── task.md              # Backward compatibility
    ├── design-spec.md       # Backward compatibility
    ├── prompt.md            # Backward compatibility
    ├── attempt-1/           # First attempt
    │   ├── task.md
    │   ├── design-spec.md
    │   ├── prompt.md
    │   ├── claude-session.md     # Session export
    │   ├── claude-session.xml
    │   └── claude-session-raw.jsonl
    └── attempt-2/           # Retry attempt
        ├── task.md
        ├── design-spec.md
        ├── prompt.md
        ├── claude-session.md
        ├── claude-session.xml
        └── claude-session-raw.jsonl
```

## Benefits
1. **History Preservation**: Each attempt's files and exports are preserved
2. **Debugging**: Can compare outputs across different attempts
3. **Clean Isolation**: Each attempt starts with a clean subdirectory
4. **Backward Compatibility**: Files still copied to task root for tools expecting them there

## Testing
To test this implementation:
1. Submit a task that will fail
2. Retry the task
3. Verify that `.task/{task_id}/attempt-1/` and `.task/{task_id}/attempt-2/` exist
4. Check that session exports are in the correct attempt subdirectory