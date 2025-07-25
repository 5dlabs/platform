#!/bin/bash

# Deploy Organization-wide GitHub Actions Runners
# Consolidated deployment script for 5dlabs org runners

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying 5dlabs Organization-wide GitHub Actions Runners${NC}"
echo -e "${BLUE}============================================================${NC}"

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}❌ kubectl is not installed or not in PATH${NC}"
    exit 1
fi

# Check if we can connect to the cluster
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}❌ Cannot connect to Kubernetes cluster${NC}"
    echo -e "${YELLOW}Make sure you have the correct kubeconfig and cluster access${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Kubernetes cluster connection verified${NC}"

# Function to check if secret exists
check_secret() {
    local secret_name=$1
    local namespace=$2

    if kubectl get secret "$secret_name" -n "$namespace" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Check for required secrets
echo -e "${YELLOW}🔐 Checking required secrets...${NC}"

if ! check_secret "arc-github-token" "arc-systems"; then
    echo -e "${RED}❌ GitHub token secret not found${NC}"
    echo -e "${YELLOW}Please create the GitHub token secret first:${NC}"
    echo -e "${CYAN}kubectl create secret generic arc-github-token -n arc-systems --from-literal=github-token=YOUR_PAT_TOKEN${NC}"
    echo ""
    echo -e "${YELLOW}The PAT token needs these permissions:${NC}"
    echo -e "  • admin:org (to manage org-level runners)"
    echo -e "  • repo (to access repositories)"
    echo -e "  • workflow (to run workflows)"
    exit 1
fi

echo -e "${GREEN}✅ GitHub token secret found${NC}"

if ! check_secret "ghcr-secret" "arc-systems"; then
    echo -e "${YELLOW}⚠️  GHCR image pull secret not found${NC}"
    echo -e "${YELLOW}If your rust-builder image is private, create the secret:${NC}"
    echo -e "${CYAN}kubectl create secret docker-registry ghcr-secret -n arc-systems \\${NC}"
    echo -e "${CYAN}  --docker-server=ghcr.io \\${NC}"
    echo -e "${CYAN}  --docker-username=YOUR_GITHUB_USERNAME \\${NC}"
    echo -e "${CYAN}  --docker-password=YOUR_GITHUB_TOKEN${NC}"
    echo ""
    echo -e "${YELLOW}Continuing with deployment (assuming public image)...${NC}"
else
    echo -e "${GREEN}✅ GHCR image pull secret found${NC}"
fi

# Check if ARC is installed
echo -e "${YELLOW}🔍 Checking Actions Runner Controller installation...${NC}"

if ! kubectl get crd runnerdeployments.actions.summerwind.dev &> /dev/null; then
    echo -e "${RED}❌ Actions Runner Controller (ARC) is not installed${NC}"
    echo -e "${YELLOW}Please install ARC first:${NC}"
    echo -e "${CYAN}helm repo add actions-runner-controller https://actions-runner-controller.github.io/actions-runner-controller${NC}"
    echo -e "${CYAN}helm upgrade --install arc actions-runner-controller/actions-runner-controller \\${NC}"
    echo -e "${CYAN}  --namespace arc-systems --create-namespace${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Actions Runner Controller is installed${NC}"

# Apply the configuration
echo -e "${YELLOW}📦 Applying runner configuration...${NC}"

if kubectl apply -f "$(dirname "$0")/arc-org-runners.yaml"; then
    echo -e "${GREEN}✅ Runner configuration applied successfully${NC}"
else
    echo -e "${RED}❌ Failed to apply runner configuration${NC}"
    exit 1
fi

# Wait for PVC to be bound
echo -e "${YELLOW}⏳ Waiting for PVC to be bound...${NC}"

timeout=60
counter=0
while [ $counter -lt $timeout ]; do
    if kubectl get pvc rust-cache-pvc -n arc-systems -o jsonpath='{.status.phase}' 2>/dev/null | grep -q "Bound"; then
        echo -e "${GREEN}✅ PVC is bound and ready${NC}"
        break
    fi

    if [ $counter -eq 0 ]; then
        echo -e "${YELLOW}Waiting for PVC to bind...${NC}"
    fi

    sleep 2
    counter=$((counter + 2))
done

if [ $counter -ge $timeout ]; then
    echo -e "${YELLOW}⚠️  PVC binding is taking longer than expected${NC}"
    echo -e "${YELLOW}Check PVC status: kubectl get pvc rust-cache-pvc -n arc-systems${NC}"
fi

# Check runner deployment status
echo -e "${YELLOW}🏃 Checking runner deployment status...${NC}"

if kubectl get runnerdeployment org-runners -n arc-systems &> /dev/null; then
    echo -e "${GREEN}✅ Runner deployment created${NC}"

    # Show current status
    echo -e "${CYAN}Current runner status:${NC}"
    kubectl get runnerdeployment org-runners -n arc-systems

    echo ""
    echo -e "${CYAN}Runner pods:${NC}"
    kubectl get pods -n arc-systems -l app=org-runners

else
    echo -e "${RED}❌ Runner deployment not found${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Deployment completed!${NC}"
echo ""
echo -e "${YELLOW}📋 Summary:${NC}"
echo -e "  • Organization: ${CYAN}5dlabs${NC}"
echo -e "  • Runners: ${CYAN}4 replicas${NC}"
echo -e "  • Image: ${CYAN}ghcr.io/5dlabs/agent-platform/rust-builder:1.1.0${NC}"
echo -e "  • Storage: ${CYAN}100Gi PVC for persistent caching${NC}"
echo -e "  • Labels: ${CYAN}self-hosted, linux, x64, k8s-runner, rust-builder, org-runner${NC}"
echo ""
echo -e "${YELLOW}🔧 Useful commands:${NC}"
echo -e "  • Check runners: ${CYAN}kubectl get runnerdeployment -n arc-systems${NC}"
echo -e "  • Check pods: ${CYAN}kubectl get pods -n arc-systems${NC}"
echo -e "  • Check PVC: ${CYAN}kubectl get pvc -n arc-systems${NC}"
echo -e "  • View logs: ${CYAN}kubectl logs -f deployment/org-runners -n arc-systems${NC}"
echo -e "  • Scale runners: ${CYAN}kubectl patch runnerdeployment org-runners -n arc-systems -p '{\"spec\":{\"replicas\":6}}'${NC}"
echo ""
echo -e "${GREEN}✅ Your org-wide runners are ready to use!${NC}"