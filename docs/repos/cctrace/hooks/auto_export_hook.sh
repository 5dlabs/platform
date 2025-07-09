#!/bin/bash

# Claude Code Auto Export Hook
# Automatically exports Claude Code sessions when they end
# This hook runs when the main Claude agent finishes (Stop event)

# Get the working directory from Claude's JSON environment
CWD=$(jq -r '.cwd // "."' <<< "$CLAUDE_JSON" 2>/dev/null || pwd)

# Change to the project directory
cd "$CWD" || exit 1

# Log file for debugging
LOG_FILE="/tmp/claude_export_$(date +%Y%m%d_%H%M%S).log"

# Run the export
echo "=== Claude Code Auto Export Hook ===" >> "$LOG_FILE"
echo "Time: $(date)" >> "$LOG_FILE"
echo "Directory: $CWD" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

# Execute the export with a 1-hour window to catch the session
python3 ~/claude_sessions/export_claude_session.py --max-age 3600 >> "$LOG_FILE" 2>&1

# Check if export was successful
if [ $? -eq 0 ]; then
    echo "✅ Export completed successfully" >> "$LOG_FILE"
    
    # Optional: Send desktop notification (if available)
    if command -v notify-send &> /dev/null; then
        notify-send "Claude Session Exported" "Your Claude Code session has been automatically saved" -i dialog-information 2>/dev/null || true
    fi
else
    echo "❌ Export failed" >> "$LOG_FILE"
fi

echo "" >> "$LOG_FILE"
echo "=== End of export ===" >> "$LOG_FILE"

# Always exit successfully to not interfere with Claude Code
exit 0