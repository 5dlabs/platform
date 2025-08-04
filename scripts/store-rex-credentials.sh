#!/bin/bash
# Quick script to store Rex's credentials after manual creation

echo "🦖 Store Rex GitHub App Credentials"
echo "===================================="
echo

# Get App ID
read -p "Enter the App ID (from GitHub page): " APP_ID

# Get private key path
read -p "Enter path to the downloaded private key (.pem file): " KEY_PATH

# Expand path
KEY_PATH=$(eval echo "$KEY_PATH")

if [[ ! -f "$KEY_PATH" ]]; then
    echo "❌ Error: Private key file not found at: $KEY_PATH"
    exit 1
fi

# Read private key
PRIVATE_KEY=$(cat "$KEY_PATH")

echo
echo "Storing credentials in Kubernetes..."

# Store in secret-store namespace first
kubectl create secret generic "github-app-5dlabs-rex" \
    --namespace="secret-store" \
    --from-literal="app-id=${APP_ID}" \
    --from-literal="private-key=${PRIVATE_KEY}" \
    --dry-run=client -o yaml | kubectl apply -f -

# Create ExternalSecret to sync to agent-platform
cat <<EOF | kubectl apply -f -
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-rex
  namespace: agent-platform
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-rex
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-5dlabs-rex
      property: app-id
  - secretKey: private-key
    remoteRef:
      key: github-app-5dlabs-rex
      property: private-key
EOF

echo
echo "✅ Rex's credentials stored successfully!"
echo
echo "🦖 Rex says: 'Excellent work! Now let's build something robust.'"