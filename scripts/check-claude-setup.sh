#!/bin/bash
# Script to check Claude agent setup and troubleshoot issues

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="orchestrator"
SECRET_NAME="claude-api-key"

echo -e "${BLUE}Claude Agent Setup Checker${NC}"
echo "================================="
echo ""

# Check secret exists
echo -e "${YELLOW}1. Checking API Key Secret...${NC}"
if kubectl get secret $SECRET_NAME -n $NAMESPACE &> /dev/null; then
    echo -e "${GREEN}✓ Secret '$SECRET_NAME' exists${NC}"
    
    # Check if secret has the api-key field
    if kubectl get secret $SECRET_NAME -n $NAMESPACE -o jsonpath='{.data.api-key}' | base64 -d &> /dev/null; then
        KEY_LENGTH=$(kubectl get secret $SECRET_NAME -n $NAMESPACE -o jsonpath='{.data.api-key}' | base64 -d | wc -c)
        echo -e "${GREEN}✓ API key is present (length: $KEY_LENGTH characters)${NC}"
    else
        echo -e "${RED}✗ API key field is missing or invalid${NC}"
    fi
else
    echo -e "${RED}✗ Secret '$SECRET_NAME' not found in namespace '$NAMESPACE'${NC}"
    echo ""
    echo "To create the secret, run:"
    echo "  ./scripts/update-claude-api-key.sh"
    exit 1
fi

echo ""

# Check recent Claude agent pods
echo -e "${YELLOW}2. Recent Claude Agent Pods...${NC}"
PODS=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=claude-agent --no-headers 2>/dev/null || echo "")
if [ -z "$PODS" ]; then
    echo "No Claude agent pods found. Checking all recent job pods..."
    kubectl get pods -n $NAMESPACE | grep -E "(task-|claude-)" | tail -5 || echo "No task pods found"
else
    echo "$PODS"
fi

echo ""

# Check failed pods
echo -e "${YELLOW}3. Failed Pod Logs (last 3)...${NC}"
FAILED_PODS=$(kubectl get pods -n $NAMESPACE --field-selector=status.phase=Failed --no-headers 2>/dev/null | tail -3 | awk '{print $1}')
if [ -n "$FAILED_PODS" ]; then
    for POD in $FAILED_PODS; do
        echo -e "${RED}Pod: $POD${NC}"
        kubectl logs -n $NAMESPACE $POD --tail=5 2>/dev/null || echo "  Could not get logs"
        echo ""
    done
else
    echo -e "${GREEN}No failed pods found${NC}"
fi

# Check error pods
ERROR_PODS=$(kubectl get pods -n $NAMESPACE | grep Error | tail -3 | awk '{print $1}')
if [ -n "$ERROR_PODS" ]; then
    echo -e "${YELLOW}4. Error Pod Logs (last 3)...${NC}"
    for POD in $ERROR_PODS; do
        echo -e "${RED}Pod: $POD${NC}"
        kubectl logs -n $NAMESPACE $POD --tail=5 2>/dev/null || echo "  Could not get logs"
        echo ""
    done
fi

# Check TaskRuns
echo -e "${YELLOW}5. Recent TaskRuns...${NC}"
kubectl get taskruns -n $NAMESPACE --no-headers | tail -5 | while read -r line; do
    NAME=$(echo "$line" | awk '{print $1}')
    PHASE=$(kubectl get taskrun $NAME -n $NAMESPACE -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
    MESSAGE=$(kubectl get taskrun $NAME -n $NAMESPACE -o jsonpath='{.status.message}' 2>/dev/null || echo "")
    
    if [ "$PHASE" = "Running" ]; then
        echo -e "${GREEN}$NAME: $PHASE${NC}"
    elif [ "$PHASE" = "Failed" ]; then
        echo -e "${RED}$NAME: $PHASE - $MESSAGE${NC}"
    else
        echo "$NAME: $PHASE"
    fi
done

echo ""
echo -e "${BLUE}Common Issues:${NC}"
echo "1. Invalid API key: Update with ./scripts/update-claude-api-key.sh"
echo "2. Pod crashes: Check logs with: kubectl logs -n $NAMESPACE <pod-name>"
echo "3. Image pull errors: Check imagePullSecrets configuration"
echo "4. PVC issues: Verify shared-workspace PVC exists and is bound"