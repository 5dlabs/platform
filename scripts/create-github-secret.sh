#!/bin/bash
# Script to create GitHub PAT secret for an agent

AGENT_NAME=$1
GITHUB_TOKEN=$2

if [ -z "$AGENT_NAME" ] || [ -z "$GITHUB_TOKEN" ]; then
    echo "Usage: $0 <agent-name> <github-token>"
    echo "Example: $0 swe-1-5dlabs ghp_xxxxxxxxxxxx"
    exit 1
fi

SECRET_NAME="github-pat-${AGENT_NAME}"

kubectl create secret generic "$SECRET_NAME" \
    --from-literal=token="$GITHUB_TOKEN" \
    --namespace=orchestrator \
    --dry-run=client -o yaml | kubectl apply -f -

echo "Created secret: $SECRET_NAME in namespace: orchestrator"