# Claude Code Configuration Investigation

## Summary
After investigating the Claude Code configuration system using official documentation and analyzing the orchestrator implementation, I've discovered that **the empty `allowedTools` array does not prevent Claude from using tools**.

## Key Findings

### 1. Configuration Loading Hierarchy (from Anthropic docs)
Claude Code loads configuration in this order (highest to lowest precedence):
1. Enterprise policies
2. **Command line arguments**
3. Local project settings (`.claude/settings.local.json`)
4. Shared project settings (`.claude/settings.json` or `.claude.json`)
5. User settings (`~/.claude/settings.json`)

### 2. Current Implementation Analysis

#### Command Execution
From `main-container.sh.hbs` line 158:
```bash
CLAUDE_CMD="{{command}} -p --output-format stream-json --verbose"
```

The orchestrator is NOT using the `--allowedTools` flag when starting Claude.

#### Default Configuration
From `default_config.yaml`:
- Command: `["claude"]` 
- Args: `["-p", "Read the task context..."]`
- claudeSettings has tool permissions configured with all tools allowed (`"*"`)

### 3. Why Claude Still Has Tool Access

Despite the empty `allowedTools` array in `.claude.json`, Claude can still use tools because:

1. **No CLI Restriction**: The orchestrator doesn't pass `--disallowedTools` flags
2. **Default Behavior**: Without explicit CLI restrictions, Claude likely defaults to allowing tools
3. **Configuration Override**: The `.claude.json` being created by Claude on startup (with empty allowedTools) might not be the active configuration

### 4. The Real Issue

The configuration file shows two different states:
- **Init container**: Creates config with tools enabled
- **Main container**: Shows config with empty allowedTools

This happens because:
1. Init container creates a proper `.claude.json`
2. Claude Code starts and creates its own default `.claude.json`, overwriting the template
3. However, Claude still operates with tools enabled (possibly using internal defaults)

## Recommendations

### Option 1: Use CLI Flags (Explicit Control)
```bash
CLAUDE_CMD="claude -p --output-format stream-json --verbose --allowedTools 'Bash' 'Edit' 'Read' 'Write' 'Glob' 'Grep' 'TodoWrite' 'TodoRead'"
```

### Option 2: Use settings.local.json
According to docs, `.claude/settings.local.json` takes precedence over `.claude.json`. Create this file instead:
```json
{
  "projects": {
    "/workspace": {
      "allowedTools": ["Bash", "Edit", "Read", "Write", "Glob", "Grep", "TodoWrite", "TodoRead"]
    }
  }
}
```

### Option 3: Prevent Claude from Creating Default Config
Set up the configuration before Claude starts to prevent it from creating defaults:
```bash
# Create .claude directory with proper permissions
mkdir -p /workspace/.claude
# Create settings files before Claude runs
# This might prevent Claude from creating its own
```

## Conclusion

The empty `allowedTools` array is **not blocking Claude's functionality**. Claude is successfully using tools despite this configuration anomaly. However, for clarity and explicit control, we should:

1. **Short term**: Add `--allowedTools` flags to the Claude command
2. **Long term**: Understand why Claude overwrites the config and prevent it
3. **Best practice**: Use explicit CLI flags for production deployments rather than relying on configuration files that might be overwritten