#!/bin/bash
# Script to update Claude API key in Kubernetes secret

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="orchestrator"
SECRET_NAME="claude-api-key"

echo -e "${YELLOW}Claude API Key Update Script${NC}"
echo "================================="
echo ""

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}Error: kubectl is not installed or not in PATH${NC}"
    exit 1
fi

# Check if we can access the cluster
if ! kubectl get ns $NAMESPACE &> /dev/null; then
    echo -e "${RED}Error: Cannot access namespace $NAMESPACE${NC}"
    exit 1
fi

# Get API key from parameter or prompt
if [ -n "$1" ]; then
    API_KEY="$1"
    echo "Using API key from command line parameter"
else
    echo "Please enter your Claude API key:"
    echo "(It will not be displayed as you type)"
    read -s API_KEY
fi

if [ -z "$API_KEY" ]; then
    echo -e "${RED}Error: API key cannot be empty${NC}"
    echo ""
    echo "Usage: $0 [API_KEY]"
    echo "  You can provide the API key as a parameter or enter it when prompted"
    exit 1
fi

echo ""
echo "Updating secret..."

# Delete existing secret if it exists
if kubectl get secret $SECRET_NAME -n $NAMESPACE &> /dev/null; then
    echo "Deleting existing secret..."
    kubectl delete secret $SECRET_NAME -n $NAMESPACE
fi

# Create new secret
kubectl create secret generic $SECRET_NAME \
    --from-literal=api-key="$API_KEY" \
    -n $NAMESPACE

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Secret updated successfully!${NC}"
    echo ""
    echo "The secret '$SECRET_NAME' has been updated in namespace '$NAMESPACE'"
    echo ""
    echo "To verify the secret was created:"
    echo "  kubectl get secret $SECRET_NAME -n $NAMESPACE"
    echo ""
    echo "To restart existing pods to pick up the new key:"
    echo "  kubectl delete pods -n $NAMESPACE -l app.kubernetes.io/name=claude-agent"
else
    echo -e "${RED}Error: Failed to create secret${NC}"
    exit 1
fi