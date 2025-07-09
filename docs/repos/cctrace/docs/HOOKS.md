# Claude Code Hooks Configuration Guide

This guide explains how to configure Claude Code hooks to automatically export your sessions.

## What are Claude Code Hooks?

Claude Code hooks are user-defined shell commands that execute at specific points in Claude Code's lifecycle. They allow you to:
- Automatically export sessions when they end
- Log commands as they're executed
- Send notifications
- Implement custom workflows

## Hook Events

| Event | Description | Use Case |
|-------|-------------|----------|
| `PreToolUse` | Before any tool is used | Validate or log commands |
| `PostToolUse` | After a tool completes | Process results |
| `Stop` | When main agent finishes | Export complete sessions |
| `SubagentStop` | When a subagent completes | Export task-specific work |
| `Notification` | During notifications | Custom alerts |

## Basic Configuration

### 1. Edit Settings File

Open `~/.claude/settings.json` and add a hooks section:

```json
{
  "model": "opus",
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash ~/claude_sessions/hooks/auto_export_hook.sh"
          }
        ]
      }
    ]
  }
}
```

### 2. Available Hook Scripts

#### auto_export_hook.sh
- Exports every session automatically
- Creates logs in `/tmp/claude_export_*.log`
- Sends desktop notifications (if available)

#### smart_export_hook.sh
- Only exports substantial sessions (10+ messages)
- Removes exports for very short sessions
- Creates a latest export summary file

## Advanced Configurations

### Export on Specific Conditions

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": {
          "cwd": "/home/user/important-project"
        },
        "hooks": [
          {
            "type": "command",
            "command": "bash ~/claude_sessions/hooks/auto_export_hook.sh"
          }
        ]
      }
    ]
  }
}
```

### Log All Bash Commands

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "echo \"$(date): $CLAUDE_JSON\" | jq -r '.tool_input.command' >> ~/.claude/bash_history.log"
          }
        ]
      }
    ]
  }
}
```

### Multiple Hooks

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash ~/claude_sessions/hooks/auto_export_hook.sh"
          },
          {
            "type": "command",
            "command": "echo 'Session ended' | wall"
          }
        ]
      }
    ]
  }
}
```

## Environment Variables

Hooks receive context through the `$CLAUDE_JSON` environment variable:

```bash
# Extract working directory
CWD=$(jq -r '.cwd' <<< "$CLAUDE_JSON")

# Extract session ID
SESSION_ID=$(jq -r '.sessionId' <<< "$CLAUDE_JSON")

# Extract tool name (in PreToolUse/PostToolUse)
TOOL_NAME=$(jq -r '.tool_name' <<< "$CLAUDE_JSON")
```

## Creating Custom Hooks

### Template for Custom Export Hook

```bash
#!/bin/bash

# Custom Claude Code Export Hook

# Parse context
CWD=$(jq -r '.cwd // "."' <<< "$CLAUDE_JSON")
SESSION_ID=$(jq -r '.sessionId // ""' <<< "$CLAUDE_JSON")
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Your custom logic here
if [[ "$CWD" == *"important"* ]]; then
    # Only export sessions in directories containing "important"
    python3 ~/claude_sessions/export_claude_session.py \
        --output-dir ~/important_sessions \
        --max-age 7200
fi

# Always exit 0 to not interfere with Claude Code
exit 0
```

### Hook Best Practices

1. **Always exit 0**: Non-zero exit codes can interfere with Claude Code
2. **Log to files**: Use log files for debugging (`/tmp/` is good)
3. **Be fast**: Hooks should complete quickly to avoid slowing down Claude Code
4. **Handle errors**: Use `|| true` for non-critical commands
5. **Test first**: Test hooks manually before adding to settings.json

## Troubleshooting

### Hook Not Running

1. Check JSON syntax:
   ```bash
   jq . ~/.claude/settings.json
   ```

2. Check hook script permissions:
   ```bash
   chmod +x ~/claude_sessions/hooks/*.sh
   ```

3. Check logs:
   ```bash
   ls -la /tmp/claude_export_*.log
   tail -f /tmp/claude_export_*.log
   ```

### Common Issues

- **jq not found**: Install with `sudo apt-get install jq`
- **Permission denied**: Ensure scripts are executable
- **Silent failures**: Check log files for errors
- **Settings not loading**: Restart Claude Code after editing settings.json

## Security Considerations

- Hooks run with your full user permissions
- Be careful with untrusted hook scripts
- Validate inputs from `$CLAUDE_JSON`
- Use absolute paths in hook commands
- Consider using read-only operations in hooks

## Examples Repository

Find more hook examples in the `hooks/` directory:
- `auto_export_hook.sh` - Basic automatic export
- `smart_export_hook.sh` - Conditional export with filtering
- More examples coming soon!

---

For more information about Claude Code hooks, see the [official documentation](https://docs.anthropic.com/en/docs/claude-code/hooks).