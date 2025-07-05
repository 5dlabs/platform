#!/bin/bash
# Script to check GitHub PAT permissions from .env file

# Find the .env file at the root of the project
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$(dirname "$SCRIPT_DIR")/.env"

if [ ! -f "$ENV_FILE" ]; then
    echo "Error: .env file not found at $ENV_FILE"
    exit 1
fi

echo "Checking GitHub PAT permissions from $ENV_FILE"
echo "=============================================="
echo ""

# Counter for tokens
CHECKED=0
SKIPPED=0

# Read the .env file and process GitHub PAT entries
while IFS='=' read -r key value; do
    # Skip empty lines and comments
    [[ -z "$key" || "$key" =~ ^[[:space:]]*# ]] && continue
    
    # Remove leading/trailing whitespace
    key=$(echo "$key" | xargs)
    value=$(echo "$value" | xargs)
    
    # Remove quotes if present
    value="${value%\"}"
    value="${value#\"}"
    value="${value%\'}"
    value="${value#\'}"
    
    # Check if this is a GitHub PAT entry (format: GITHUB_PAT_USERNAME=token)
    if [[ "$key" =~ ^GITHUB_PAT_(.+)$ ]]; then
        USERNAME="${BASH_REMATCH[1]}"
        # Convert USERNAME to lowercase and replace underscores with hyphens for display
        DISPLAY_NAME=$(echo "$USERNAME" | tr '[:upper:]' '[:lower:]' | tr '_' '-')
        
        if [ -z "$value" ] || [ "$value" = "ghp_XXXXXXXXXXXXXXXXXXXXX" ]; then
            echo "⚠️  $DISPLAY_NAME - Skipped (token is empty or placeholder)"
            ((SKIPPED++))
            echo ""
            continue
        fi
        
        echo "🔍 Checking permissions for: $DISPLAY_NAME"
        echo "   Token: ${value:0:10}...${value: -4}"
        
        # Get authenticated user info
        USER_INFO=$(curl -s -H "Authorization: Bearer $value" https://api.github.com/user)
        
        if [ "$(echo "$USER_INFO" | jq -r '.message // empty')" = "Bad credentials" ]; then
            echo "   ❌ Invalid token"
            echo ""
            continue
        fi
        
        LOGIN=$(echo "$USER_INFO" | jq -r '.login // "unknown"')
        echo "   GitHub User: $LOGIN"
        
        # Get token scopes from headers
        SCOPES=$(curl -s -I -H "Authorization: Bearer $value" https://api.github.com/user | grep -i "x-oauth-scopes:" | cut -d' ' -f2- | tr -d '\r\n')
        
        if [ -n "$SCOPES" ]; then
            echo "   Token Scopes: $SCOPES"
        else
            echo "   Token Scopes: (none detected - may be fine-grained PAT)"
        fi
        
        # Check a few common repositories the user might have access to
        echo "   Repository Access:"
        
        # Test repositories - adjust these to your actual repos
        TEST_REPOS=(
            "5dlabs/agent-sandbox"
            "5dlabs/platform"
        )
        
        for REPO in "${TEST_REPOS[@]}"; do
            # Check if user can access the repo
            REPO_CHECK=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $value" "https://api.github.com/repos/$REPO")
            
            if [ "$REPO_CHECK" = "200" ]; then
                # Check push permissions
                REPO_PERMS=$(curl -s -H "Authorization: Bearer $value" "https://api.github.com/repos/$REPO")
                CAN_PUSH=$(echo "$REPO_PERMS" | jq -r '.permissions.push // false')
                CAN_PULL=$(echo "$REPO_PERMS" | jq -r '.permissions.pull // false')
                CAN_ADMIN=$(echo "$REPO_PERMS" | jq -r '.permissions.admin // false')
                
                echo -n "     • $REPO: "
                if [ "$CAN_ADMIN" = "true" ]; then
                    echo "✅ Admin"
                elif [ "$CAN_PUSH" = "true" ]; then
                    echo "✅ Write"
                elif [ "$CAN_PULL" = "true" ]; then
                    echo "⚠️  Read-only"
                else
                    echo "❌ No permissions"
                fi
            elif [ "$REPO_CHECK" = "404" ]; then
                echo "     • $REPO: ❌ Not found or no access"
            else
                echo "     • $REPO: ❌ Error (HTTP $REPO_CHECK)"
            fi
        done
        
        ((CHECKED++))
        echo ""
    fi
done < "$ENV_FILE"

echo "=============================================="
echo "Summary:"
echo "  Checked: $CHECKED tokens"
echo "  Skipped: $SKIPPED entries (empty or placeholder)"

if [ $CHECKED -eq 0 ] && [ $SKIPPED -eq 0 ]; then
    echo ""
    echo "No GitHub PAT entries found in .env file."
fi