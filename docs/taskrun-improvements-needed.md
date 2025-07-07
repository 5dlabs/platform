# TaskRun Improvements Needed

## Issues Found During Testing

### 1. Branch Issue
- Claude pushed to 'master' instead of 'main'
- Need to ensure we checkout the correct default branch after cloning

### 2. Pull Request vs Direct Push
- Current: Claude pushes directly to the branch
- Needed: Claude should create a feature branch and open a PR
- Updated test instructions to include PR creation

### 3. Smart Repository Management
- Current: Always tries to clone/backup on every task
- Needed: Only clone on first task for a service
- Subsequent tasks should just update/pull
- Logic already partially exists (checks for .git directory)

### 4. Excessive Debug Output
Current debug output includes:
- Multiple Claude version checks
- Settings file discovery in many locations  
- Environment variable dumps
- Directory listings at multiple levels
- DEVCONTAINER variables that aren't relevant

Should reduce to:
- Git repository status
- Working directory confirmation
- Task file availability
- Simple "Claude starting" message

### 5. Commit Scope
- Current: Only specific files are committed
- Needed: Option to commit all workspace changes (`git add -A`)
- This helps see the full state of what Claude created

## Implementation Priority

1. **Reduce debug output** (High) - Makes logs readable
2. **Fix repository management** (High) - Prevents data loss on subsequent tasks
3. **Update default instructions** (Medium) - Ensure PRs are created, not direct pushes
4. **Add workspace state tracking** (Low) - Know if this is first task for service

## Code Locations to Update

1. **Debug output**: `taskrun.rs` - `build_agent_startup_script()` function
2. **Repository logic**: `taskrun.rs` - `build_init_script()` function around line 715
3. **Task instructions**: Update CLAUDE.md generation to include PR instructions
4. **CRD Status**: Could add `workspaceInitialized` boolean to track first run