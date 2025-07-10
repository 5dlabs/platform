#!/bin/bash
# Diagnostic script to check tasks.json versions

echo "=== Tasks.json Diagnostic ==="
echo

cd /Users/jonathonfritz/platform/example || exit 1

echo "1. LOCAL VERSION CHECK:"
echo "----------------------"
if [ -f .taskmaster/tasks/tasks.json ]; then
    FIRST_TASK=$(cat .taskmaster/tasks/tasks.json | jq -r '.master.tasks[0] | "ID: \(.id), Title: \(.title)"')
    echo "✓ Local tasks.json exists"
    echo "  First task: $FIRST_TASK"
    
    if grep -q "Express TypeScript" .taskmaster/tasks/tasks.json; then
        echo "  ❌ WARNING: Contains Node.js/Express content!"
    else
        echo "  ✓ No Node.js/Express content detected"
    fi
else
    echo "❌ Local tasks.json NOT FOUND"
fi

echo
echo "2. GIT STATUS CHECK:"
echo "-------------------"
git status --porcelain .taskmaster/ | head -10
if [ -z "$(git status --porcelain .taskmaster/)" ]; then
    echo "✓ No uncommitted changes in .taskmaster/"
    echo "  (Auto-commit won't trigger)"
else
    echo "⚠️  Uncommitted changes exist in .taskmaster/"
fi

echo
echo "3. CURRENT BRANCH:"
echo "-----------------"
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
echo "Branch: $CURRENT_BRANCH"

echo
echo "4. REMOTE VERSION CHECK:"
echo "-----------------------"
echo "Fetching latest from remote..."
git fetch origin >/dev/null 2>&1

if git show origin/$CURRENT_BRANCH:.taskmaster/tasks/tasks.json >/dev/null 2>&1; then
    REMOTE_FIRST_TASK=$(git show origin/$CURRENT_BRANCH:.taskmaster/tasks/tasks.json | jq -r '.master.tasks[0] | "ID: \(.id), Title: \(.title)"' 2>/dev/null || echo "Failed to parse")
    echo "✓ Remote tasks.json exists on origin/$CURRENT_BRANCH"
    echo "  First task: $REMOTE_FIRST_TASK"
    
    if git show origin/$CURRENT_BRANCH:.taskmaster/tasks/tasks.json | grep -q "Express TypeScript"; then
        echo "  ❌ WARNING: Remote contains Node.js/Express content!"
        echo "  This is what Claude will see!"
    fi
else
    echo "❌ Remote tasks.json NOT FOUND on origin/$CURRENT_BRANCH"
fi

echo
echo "5. LAST COMMITS TOUCHING tasks.json:"
echo "-----------------------------------"
git log --oneline -n 5 -- .taskmaster/tasks/tasks.json

echo
echo "=== DIAGNOSIS ==="
echo
if git show origin/$CURRENT_BRANCH:.taskmaster/tasks/tasks.json 2>/dev/null | grep -q '"id": 11'; then
    echo "❌ PROBLEM FOUND: Remote repository has old Node.js tasks (ID 11+)"
    echo "   This is why Claude gets the wrong tasks!"
    echo
    echo "TO FIX:"
    echo "1. Ensure local has correct tasks (ID 1: Rust tasks)"
    echo "2. Force commit and push:"
    echo "   git add .taskmaster/tasks/tasks.json"
    echo "   git commit -m 'fix: update to correct Rust tasks.json'"
    echo "   git push origin $CURRENT_BRANCH"
else
    echo "✓ Remote appears to have correct tasks"
fi