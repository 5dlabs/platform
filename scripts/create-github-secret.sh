#!/bin/bash
# Script to create GitHub PAT secrets from .env file

# Find the .env file at the root of the project
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$(dirname "$SCRIPT_DIR")/.env"

if [ ! -f "$ENV_FILE" ]; then
    echo "Error: .env file not found at $ENV_FILE"
    echo "Please create a .env file at the root of the project with GitHub PAT entries"
    exit 1
fi

echo "Reading GitHub PATs from $ENV_FILE"
echo "=================================="

# Counter for created secrets
CREATED=0
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
        # Convert USERNAME to lowercase and replace underscores with hyphens
        USERNAME=$(echo "$USERNAME" | tr '[:upper:]' '[:lower:]' | tr '_' '-')
        
        if [ -z "$value" ] || [ "$value" = "ghp_XXXXXXXXXXXXXXXXXXXXX" ]; then
            echo "⚠️  Skipping $USERNAME - token is empty or placeholder"
            ((SKIPPED++))
            continue
        fi
        
        SECRET_NAME="github-pat-${USERNAME}"
        
        echo "Creating secret: $SECRET_NAME"
        if kubectl create secret generic "$SECRET_NAME" \
            --from-literal=token="$value" \
            --namespace=orchestrator \
            --dry-run=client -o yaml | kubectl apply -f -; then
            echo "✅ Created secret: $SECRET_NAME"
            ((CREATED++))
        else
            echo "❌ Failed to create secret: $SECRET_NAME"
        fi
        echo ""
    fi
done < "$ENV_FILE"

echo "=================================="
echo "Summary:"
echo "  Created: $CREATED secrets"
echo "  Skipped: $SKIPPED entries (empty or placeholder)"

if [ $CREATED -eq 0 ] && [ $SKIPPED -eq 0 ]; then
    echo ""
    echo "No GitHub PAT entries found in .env file."
    echo "Add entries in the format: GITHUB_PAT_USERNAME=ghp_xxxxxxxxxxxx"
fi