# Init Container Improvements

## Changes Made

### 1. Switched to Claude Code Image for Init Container
**Before**: `alpine/git:latest`
**After**: `ghcr.io/5dlabs/platform/claude-code:latest`

**Benefits**:
- Same image for both init and main containers
- Pre-installed tools: git, gh CLI, node, npm, and all Claude Code dependencies
- No need to install additional packages
- Consistent environment between containers

### 2. Removed Incorrect Claude Config Commands
**Issue**: Claude Code doesn't have `config show`, `config get`, etc. commands
**Fix**: Removed these debug commands and focused on verifying .claude.json file

**Correct Configuration Method**:
- Claude Code uses `.claude.json` file for configuration
- Tool permissions are set via the `allowedTools` array in the project settings
- No command-line config commands needed

### 3. Simplified Init Script
**Removed**:
- `apk add` commands (not needed with Claude Code image)
- Package installation attempts

**Added**:
- Verification that git and gh are available
- Clear messages about using pre-installed tools

## Expected Benefits

1. **Faster Init**: No package installation needed
2. **More Reliable**: Using the same tested image
3. **Better Debugging**: Claude Code image has better debugging tools built-in
4. **Consistent Environment**: Same Node.js version, same file permissions, same user setup

## Configuration Format Reminder

Claude Code expects configuration in `.claude.json`:
```json
{
  "projects": {
    "/workspace/test-service": {
      "allowedTools": ["Bash", "Read", "Write", "Edit"]
    }
  }
}
```

This is the ONLY way to configure Claude Code's tool permissions.