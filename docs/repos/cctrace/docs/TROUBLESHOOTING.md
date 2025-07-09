# Troubleshooting Guide

This guide helps resolve common issues with Claude Code Session Export.

## Common Issues

### 1. "No Claude Code sessions found"

**Symptoms:**
- Export script shows no sessions for your project
- `~/.claude/projects/` doesn't contain your project

**Solutions:**

1. **Verify you're in the right directory:**
   ```bash
   pwd  # Should show your project path
   ```

2. **Check if sessions exist:**
   ```bash
   ls -la ~/.claude/projects/
   ```

3. **Ensure Claude Code has written to the project:**
   - Send at least one message in Claude Code
   - Wait a moment for the JSONL file to be created

4. **Check project name mapping:**
   ```bash
   # Your directory: /home/user/my-project
   # Claude stores as: ~/.claude/projects/-home-user-my-project/
   ```

### 2. "Multiple active sessions" / Wrong session exported

**Symptoms:**
- Multiple Claude Code instances running
- Export picks the wrong session

**Solutions:**

1. **Use session ID explicitly:**
   ```bash
   # List all sessions first
   ls -la ~/.claude/projects/-your-project/
   
   # Export specific session
   python3 ~/claude_sessions/export_claude_session.py --session-id <uuid>
   ```

2. **Increase detection window:**
   ```bash
   # For older sessions (up to 1 hour)
   python3 ~/claude_sessions/export_claude_session.py --max-age 3600
   ```

3. **Run export from within Claude Code:**
   ```
   /user:export
   ```
   This uses PID detection for better accuracy.

### 3. Hook not triggering

**Symptoms:**
- Sessions end but no automatic export
- No log files created

**Solutions:**

1. **Validate settings.json syntax:**
   ```bash
   jq . ~/.claude/settings.json
   ```
   
   If error appears, fix JSON syntax.

2. **Check hook script permissions:**
   ```bash
   chmod +x ~/claude_sessions/hooks/*.sh
   ls -la ~/claude_sessions/hooks/
   ```

3. **Test hook manually:**
   ```bash
   export CLAUDE_JSON='{"cwd":"/home/user/project","sessionId":"test"}'
   bash ~/claude_sessions/hooks/auto_export_hook.sh
   ```

4. **Check for log files:**
   ```bash
   ls -la /tmp/claude_export_*.log
   cat /tmp/claude_export_*.log
   ```

### 4. "jq: command not found"

**Symptoms:**
- Hook scripts fail with jq errors
- Can't parse CLAUDE_JSON

**Solutions:**

Install jq:
```bash
# Ubuntu/Debian
sudo apt-get update && sudo apt-get install jq

# macOS
brew install jq

# Other systems
# Download from https://stedolan.github.io/jq/download/
```

### 5. Permission denied errors

**Symptoms:**
- Can't execute export script
- Can't write to export directory

**Solutions:**

1. **Fix script permissions:**
   ```bash
   chmod +x ~/claude_sessions/export_claude_session.py
   ```

2. **Fix directory permissions:**
   ```bash
   mkdir -p ~/claude_sessions/exports
   chmod 755 ~/claude_sessions
   chmod 755 ~/claude_sessions/exports
   ```

3. **Check file ownership:**
   ```bash
   ls -la ~/.claude/projects/
   # Should be owned by your user
   ```

### 6. Export is empty or incomplete

**Symptoms:**
- Exported files exist but have no content
- Missing messages or thinking blocks

**Solutions:**

1. **Ensure session is complete:**
   - Wait for Claude Code to finish writing
   - Try increasing --max-age parameter

2. **Check JSONL file directly:**
   ```bash
   # Find your session file
   ls -la ~/.claude/projects/-your-project/*.jsonl
   
   # Check if it has content
   wc -l ~/.claude/projects/-your-project/*.jsonl
   
   # View last few lines
   tail ~/.claude/projects/-your-project/*.jsonl
   ```

3. **Look for JSON parsing errors:**
   ```bash
   # Test if JSONL is valid
   cat ~/.claude/projects/-your-project/*.jsonl | while read line; do
     echo "$line" | jq . > /dev/null || echo "Invalid JSON: $line"
   done
   ```

### 7. Slash command not working

**Symptoms:**
- `/user:export` not recognized
- Command does nothing

**Solutions:**

1. **Verify installation:**
   ```bash
   cat ~/.claude/commands/export.md
   ```

2. **Restart Claude Code:**
   - Close and reopen Claude Code
   - Slash commands are loaded on startup

3. **Check command syntax:**
   - Use `/user:export` (not `/export`)
   - No spaces in command name

## Debugging Steps

### 1. Enable Verbose Logging

Add debug output to export script:
```bash
# Edit the script
nano ~/claude_sessions/export_claude_session.py

# Add at the beginning of main():
print(f"DEBUG: Python version: {sys.version}")
print(f"DEBUG: Script location: {__file__}")
print(f"DEBUG: Home directory: {Path.home()}")
```

### 2. Check Claude Code Process

```bash
# Find Claude processes
ps aux | grep claude

# Check if running inside Claude Code
echo $PPID
ps -p $PPID -o comm=
```

### 3. Manual Session Check

```python
# Python script to check sessions
import json
from pathlib import Path

sessions_dir = Path.home() / '.claude' / 'projects'
for project_dir in sessions_dir.iterdir():
    print(f"\nProject: {project_dir.name}")
    for jsonl_file in project_dir.glob('*.jsonl'):
        print(f"  Session: {jsonl_file.name}")
        with open(jsonl_file) as f:
            lines = f.readlines()
            print(f"  Messages: {len(lines)}")
            if lines:
                first = json.loads(lines[0])
                last = json.loads(lines[-1])
                print(f"  Start: {first.get('timestamp', 'unknown')}")
                print(f"  End: {last.get('timestamp', 'unknown')}")
```

## Getting Help

If these solutions don't resolve your issue:

1. **Check existing issues:**
   - GitHub Issues page
   - Search for your error message

2. **Create detailed bug report:**
   - Claude Code version
   - Operating system
   - Exact error messages
   - Steps to reproduce

3. **Include diagnostic info:**
   ```bash
   # System info
   uname -a
   python3 --version
   
   # Claude sessions
   ls -la ~/.claude/projects/
   
   # Export attempt
   python3 ~/claude_sessions/export_claude_session.py 2>&1
   ```

## Prevention Tips

1. **Regular testing:**
   - Test export after installation
   - Verify hooks work before relying on them

2. **Backup important sessions:**
   - Export manually for critical work
   - Don't rely solely on automatic exports

3. **Monitor disk space:**
   - Exports can accumulate over time
   - Set up cleanup for old exports

4. **Keep tools updated:**
   - Update export script periodically
   - Check for new features and fixes