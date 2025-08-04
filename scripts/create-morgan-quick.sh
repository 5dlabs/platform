#!/bin/bash
# Quick script to create just Morgan for testing
# For org owners - fully automated

set -euo pipefail

ORG="5dlabs"
APP_NAME="5DLabs-Morgan"
DESCRIPTION="Product Management & Documentation Agent"

echo "🚀 Creating Morgan GitHub App (Org Owner Mode)"
echo "============================================="
echo

# Create manifest
cat > /tmp/morgan-manifest.json <<EOF
{
  "name": "${APP_NAME}",
  "description": "${DESCRIPTION}",
  "url": "https://github.com/${ORG}/platform",
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

# Submit manifest
echo "Submitting app manifest..."
RESPONSE=$(gh api \
    --method POST \
    -H "Accept: application/vnd.github+json" \
    "/orgs/${ORG}/app-manifests" \
    -F manifest=@/tmp/morgan-manifest.json)

CODE=$(echo "$RESPONSE" | jq -r '.code')

# Convert manifest to app
echo "Creating GitHub App..."
APP_RESPONSE=$(gh api \
    --method POST \
    -H "Accept: application/vnd.github+json" \
    "/app-manifests/${CODE}/conversions")

# Extract details
APP_ID=$(echo "$APP_RESPONSE" | jq -r '.id')
APP_SLUG=$(echo "$APP_RESPONSE" | jq -r '.slug')
CLIENT_ID=$(echo "$APP_RESPONSE" | jq -r '.client_id')
PEM=$(echo "$APP_RESPONSE" | jq -r '.pem')

echo
echo "✅ Created GitHub App!"
echo "   Name: ${APP_NAME}"
echo "   App ID: ${APP_ID}"
echo "   Slug: ${APP_SLUG}"
echo "   Client ID: ${CLIENT_ID}"
echo

# Save private key
echo "$PEM" > morgan.private-key.pem
echo "✅ Private key saved to: morgan.private-key.pem"

# Store in Kubernetes
echo
echo "Storing credentials in Kubernetes..."
kubectl create secret generic github-app-morgan \
    --namespace=agent-platform \
    --from-literal="app-id=${APP_ID}" \
    --from-literal="private-key=${PEM}" \
    --dry-run=client -o yaml | kubectl apply -f -

echo "✅ Credentials stored in agent-platform namespace"

# Create ExternalSecret
cat <<EOF | kubectl apply -f -
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-morgan
  namespace: agent-platform
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-morgan
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-morgan
      property: app-id
  - secretKey: private-key
    remoteRef:
      key: github-app-morgan
      property: private-key
EOF

echo
echo "🎉 Morgan is ready!"
echo
echo "Next steps:"
echo "1. Update DocsRun workflow template:"
echo "   Change: githubUser: 'pm0-5dlabs'"
echo "   To: githubApp: '5DLabs-Morgan'"
echo
echo "2. Test the docs workflow"
echo
echo "App details saved in:"
echo "- Private key: ./morgan.private-key.pem"
echo "- App ID: ${APP_ID}"

# Cleanup
rm -f /tmp/morgan-manifest.json