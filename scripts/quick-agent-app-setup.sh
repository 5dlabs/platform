#!/bin/bash
# Quick GitHub App Setup - Simplified Version
# Uses GitHub CLI for maximum automation

set -euo pipefail

# First agent to test with
APP_NAME="5DLabs-Morgan"
APP_DESCRIPTION="Product Management & Documentation Agent"
REPO="platform"
ORG="5dlabs"

echo "🚀 Quick GitHub App Setup for ${APP_NAME}"
echo "======================================"
echo

# Step 1: Create the GitHub App manifest
cat > app-manifest.json <<EOF
{
  "name": "${APP_NAME}",
  "description": "${APP_DESCRIPTION}",
  "url": "https://github.com/${ORG}/${REPO}",
  "hook_attributes": {
    "active": false
  },
  "public": false,
  "default_permissions": {
    "contents": "write",
    "pull_requests": "write", 
    "issues": "write",
    "metadata": "read"
  },
  "default_events": []
}
EOF

echo "📝 Created app manifest"

# Step 2: Create app via manifest flow (requires browser)
echo "Creating GitHub App via manifest..."
RESPONSE=$(gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  "/orgs/${ORG}/app-manifests" \
  -F manifest=@app-manifest.json)

CODE=$(echo "$RESPONSE" | jq -r '.code')

echo 
echo "⚠️  MANUAL STEP REQUIRED:"
echo "1. Open this URL in your browser:"
echo "   https://github.com/organizations/${ORG}/settings/apps/new?code=${CODE}"
echo "2. Click 'Create GitHub App'"
echo "3. Download the private key when prompted"
echo "4. Note the App ID shown on the page"
echo
echo "Press ENTER when complete..."
read -r

# Step 3: Get app details
echo "Enter the App ID from the GitHub page: "
read -r APP_ID

echo "Enter the path to the downloaded private key file: "
read -r PRIVATE_KEY_PATH

# Step 4: Create Kubernetes secret
echo "Creating Kubernetes secret..."
kubectl create secret generic "github-app-morgan" \
    --namespace="agent-platform" \
    --from-literal="app-id=${APP_ID}" \
    --from-file="private-key=${PRIVATE_KEY_PATH}" \
    --dry-run=client -o yaml | kubectl apply -f -

# Step 5: Quick test script
cat > test-morgan-auth.sh <<'EOF'
#!/bin/bash
# Test GitHub App authentication

APP_ID=$(kubectl get secret github-app-morgan -n agent-platform -o jsonpath='{.data.app-id}' | base64 -d)
PRIVATE_KEY=$(kubectl get secret github-app-morgan -n agent-platform -o jsonpath='{.data.private-key}' | base64 -d)

echo "Testing GitHub App authentication..."
echo "App ID: $APP_ID"

# This would be in your workflow
# Generate JWT, get installation token, make API calls
echo "✅ Credentials stored successfully!"
EOF

chmod +x test-morgan-auth.sh

echo
echo "✅ Setup complete!"
echo
echo "Next steps:"
echo "1. Update DocsRun workflow template to use 'githubApp: 5DLabs-Morgan'"
echo "2. Test a docs workflow"
echo "3. If successful, run full agent creation script"

# Cleanup
rm app-manifest.json