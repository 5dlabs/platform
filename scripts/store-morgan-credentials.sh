#!/bin/bash
set -e

echo "=== Storing Morgan GitHub App Credentials ==="

# Morgan's App ID (you'll need to get this from the app settings page)
read -p "Enter Morgan's App ID: " APP_ID

# Check if private key exists
if [ ! -f "5DLabs-Morgan.private-key.pem" ]; then
    echo "Error: 5DLabs-Morgan.private-key.pem not found in current directory"
    exit 1
fi

# Create the secret
kubectl create secret generic morgan-github-app \
  --from-literal=app-id="${APP_ID}" \
  --from-file=private-key=5DLabs-Morgan.private-key.pem \
  --namespace=external-secrets \
  --dry-run=client -o yaml | kubectl apply -f -

echo "✅ Morgan credentials stored in external-secrets namespace"

# Create ExternalSecret for agent-platform namespace
cat << YAML | kubectl apply -f -
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: morgan-github-app
  namespace: agent-platform
spec:
  secretStoreRef:
    name: cluster-secret-store
    kind: ClusterSecretStore
  target:
    name: morgan-github-app
  dataFrom:
  - extract:
      key: morgan-github-app
      conversionStrategy: Default
YAML

echo "✅ ExternalSecret created to sync Morgan credentials to agent-platform namespace"
