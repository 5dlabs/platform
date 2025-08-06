#!/bin/bash
# Auto-save hook for docs generation - prevents work loss by pushing completed tasks incrementally

# Only process .taskmaster/docs files
if [[ "$1" != *"/.taskmaster/docs/"* ]] || [[ "$1" != *.md ]]; then
    exit 0
fi

echo "[Auto-save] Processing file: $1"

# Extract task number
TASK_NUM=$(echo "$1" | grep -oE 'task-[0-9]+' | head -1)
if [ -z "$TASK_NUM" ]; then
    exit 0
fi

TASK_DIR=$(dirname "$1")

# Check if all 3 required files exist for this task
if [ ! -f "$TASK_DIR/task.md" ] || [ ! -f "$TASK_DIR/prompt.md" ] || [ ! -f "$TASK_DIR/acceptance-criteria.md" ]; then
    echo "[Auto-save] Task $TASK_NUM not complete yet, waiting for all files"
    exit 0
fi

# Lock file to prevent concurrent git operations
LOCK_FILE="/tmp/docs-auto-save.lock"
LOCK_TIMEOUT=30

# Try to acquire lock with timeout
WAITED=0
while [ -f "$LOCK_FILE" ] && [ $WAITED -lt $LOCK_TIMEOUT ]; do
    echo "[Auto-save] Waiting for previous git operation to complete..."
    sleep 1
    WAITED=$((WAITED + 1))
done

if [ $WAITED -ge $LOCK_TIMEOUT ]; then
    echo "[Auto-save] Warning: Lock timeout reached, proceeding anyway"
    rm -f "$LOCK_FILE"
fi

# Create lock
echo $$ > "$LOCK_FILE"

# Ensure we're on the auto-save branch
if [ -z "$DOCS_AUTO_SAVE_BRANCH" ]; then
    # Create branch name similar to container script pattern
    TIMESTAMP=$(date +%Y%m%d-%H%M%S)
    export DOCS_AUTO_SAVE_BRANCH="docs/auto-save-${TIMESTAMP}"
    
    # Create or switch to branch
    if ! git checkout -b "$DOCS_AUTO_SAVE_BRANCH" 2>/dev/null; then
        git checkout "$DOCS_AUTO_SAVE_BRANCH"
    fi
    echo "[Auto-save] Created/switched to branch: $DOCS_AUTO_SAVE_BRANCH"
fi

# Stage the completed task
git add "$TASK_DIR" 2>/dev/null

# Check if there are actual changes to commit
if git diff --cached --quiet; then
    echo "[Auto-save] No changes to commit for $TASK_NUM"
    rm -f "$LOCK_FILE"
    exit 0
fi

# Commit the task
if ! git commit -m "docs: auto-save $TASK_NUM documentation" -m "Incremental backup to prevent work loss" 2>/dev/null; then
    echo "[Auto-save] Failed to commit $TASK_NUM"
    rm -f "$LOCK_FILE"
    exit 1
fi

echo "[Auto-save] Committed $TASK_NUM documentation"

# Check if token needs refresh (if been > 30 minutes)
TOKEN_AGE_FILE="/tmp/github-token-age"
CURRENT_TIME=$(date +%s)

if [ -f "$TOKEN_AGE_FILE" ]; then
    LAST_REFRESH=$(cat "$TOKEN_AGE_FILE")
    AGE=$((CURRENT_TIME - LAST_REFRESH))
    
    if [ $AGE -gt 1800 ]; then
        echo "[Auto-save] Token is ${AGE}s old, refreshing..."
        
        # Only refresh if we have the required env vars
        if [ -n "$GITHUB_APP_PRIVATE_KEY" ] && [ -n "$GITHUB_APP_ID" ] && [ -n "$INSTALLATION_ID" ]; then
            # Generate new JWT
            TEMP_KEY="/tmp/github-key-$$"
            echo "$GITHUB_APP_PRIVATE_KEY" > "$TEMP_KEY"
            chmod 600 "$TEMP_KEY"
            
            JWT_HEADER=$(printf '{"alg":"RS256","typ":"JWT"}' | base64 -w 0 | tr '+/' '-_' | tr -d '=')
            NOW=$(date +%s)
            EXP=$((NOW + 600))
            JWT_PAYLOAD=$(printf '{"iat":%d,"exp":%d,"iss":"%s"}' "$NOW" "$EXP" "$GITHUB_APP_ID" | base64 -w 0 | tr '+/' '-_' | tr -d '=')
            JWT_SIGNATURE=$(printf '%s.%s' "$JWT_HEADER" "$JWT_PAYLOAD" | openssl dgst -sha256 -sign "$TEMP_KEY" -binary | base64 -w 0 | tr '+/' '-_' | tr -d '=')
            JWT_TOKEN="$JWT_HEADER.$JWT_PAYLOAD.$JWT_SIGNATURE"
            
            # Get new access token
            NEW_TOKEN=$(curl -s -X POST \
                -H "Authorization: Bearer $JWT_TOKEN" \
                -H "Accept: application/vnd.github+json" \
                "https://api.github.com/app/installations/$INSTALLATION_ID/access_tokens" | jq -r '.token')
            
            if [ -n "$NEW_TOKEN" ] && [ "$NEW_TOKEN" != "null" ]; then
                export GITHUB_TOKEN="$NEW_TOKEN"
                echo "https://x-access-token:${GITHUB_TOKEN}@github.com" > ~/.git-credentials
                echo "$CURRENT_TIME" > "$TOKEN_AGE_FILE"
                echo "[Auto-save] Token refreshed successfully"
            fi
            
            rm -f "$TEMP_KEY"
        fi
    fi
else
    # First run, record current time
    echo "$CURRENT_TIME" > "$TOKEN_AGE_FILE"
fi

# Push with retries (run in background to not block Claude)
(
    PUSHED=false
    for attempt in 1 2 3; do
        if git push -u origin "$DOCS_AUTO_SAVE_BRANCH" 2>/dev/null; then
            echo "[Auto-save] ✓ Successfully pushed $TASK_NUM (attempt $attempt)"
            PUSHED=true
            break
        else
            echo "[Auto-save] Push attempt $attempt failed for $TASK_NUM"
            if [ $attempt -lt 3 ]; then
                sleep 2
            fi
        fi
    done
    
    if [ "$PUSHED" = false ]; then
        echo "[Auto-save] ⚠️ Failed to push $TASK_NUM after 3 attempts - changes saved locally"
    fi
    
    # Release lock
    rm -f "$LOCK_FILE"
) &

# Return immediately so Claude isn't blocked
exit 0
