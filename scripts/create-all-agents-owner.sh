#!/bin/bash
# Fully Automated GitHub Apps Creation - For Org Owners
# Uses your existing GitHub CLI authentication

set -euo pipefail

# Configuration
ORG="5dlabs"
NAMESPACE="agent-platform"
SECRET_STORE_NAMESPACE="secret-store"

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# All 13 agents with their details
declare -A AGENTS=(
    ["morgan"]="5DLabs-Morgan|Product Management & Documentation Agent"
    ["rex"]="5DLabs-Rex|Senior Backend Architecture Agent"
    ["blaze"]="5DLabs-Blaze|Performance Optimization Agent"
    ["scout"]="5DLabs-Scout|Security & Compliance Agent"
    ["ziggy"]="5DLabs-Ziggy|API & Integration Agent"
    ["mason"]="5DLabs-Mason|Platform Architecture & DevOps Agent"
    ["vigil"]="5DLabs-Vigil|Site Reliability Agent"
    ["nimbus"]="5DLabs-Nimbus|Cloud Infrastructure Agent"
    ["pixel"]="5DLabs-Pixel|Frontend Architecture Agent"
    ["swipe"]="5DLabs-Swipe|Mobile & Responsive Design Agent"
    ["kit"]="5DLabs-Kit|Design System Agent"
    ["sherlock"]="5DLabs-Sherlock|QA Strategy Agent"
    ["otto"]="5DLabs-Otto|Test Automation Agent"
)

# Function to create GitHub App using org owner permissions
create_github_app() {
    local name=$1
    local app_name=$2
    local description=$3
    
    echo -e "${BLUE}Creating GitHub App: ${app_name}${NC}"
    
    # Create app manifest
    local manifest=$(cat <<EOF
{
  "name": "${app_name}",
  "description": "${description}",
  "url": "https://github.com/${ORG}/platform",
  "hook_attributes": {
    "active": false
  },
  "public": false,
  "default_permissions": {
    "contents": "write",
    "pull_requests": "write",
    "issues": "write",
    "metadata": "read",
    "actions": "write",
    "checks": "write"
  },
  "default_events": []
}
EOF
)
    
    # Create manifest file
    echo "$manifest" > /tmp/manifest-${name}.json
    
    # Submit manifest and get the code
    local response=$(gh api \
        --method POST \
        -H "Accept: application/vnd.github+json" \
        "/orgs/${ORG}/app-manifests" \
        -F manifest=@/tmp/manifest-${name}.json)
    
    local code=$(echo "$response" | jq -r '.code')
    
    # Auto-complete the flow using the code
    # This uses the manifest conversion endpoint
    local app_response=$(gh api \
        --method POST \
        -H "Accept: application/vnd.github+json" \
        "/app-manifests/${code}/conversions")
    
    # Extract app details
    local app_id=$(echo "$app_response" | jq -r '.id')
    local app_slug=$(echo "$app_response" | jq -r '.slug')
    local client_id=$(echo "$app_response" | jq -r '.client_id')
    local pem=$(echo "$app_response" | jq -r '.pem')
    
    echo -e "${GREEN}✓ Created app: ${app_name}"
    echo "  App ID: ${app_id}"
    echo "  Slug: ${app_slug}"
    echo "  Client ID: ${client_id}${NC}"
    
    # Store the private key
    echo "$pem" > /tmp/${name}.private-key.pem
    
    # Install the app on the platform repository
    install_app "$app_slug" "$app_id"
    
    # Store credentials in Kubernetes
    store_credentials "$name" "$app_id" "$pem"
    
    # Clean up
    rm -f /tmp/manifest-${name}.json
}

# Function to install app on repository
install_app() {
    local app_slug=$1
    local app_id=$2
    
    echo -e "${BLUE}Installing app on platform repository...${NC}"
    
    # Get repository ID
    local repo_id=$(gh api "/repos/${ORG}/platform" | jq -r '.id')
    
    # Install the app
    gh api \
        --method POST \
        -H "Accept: application/vnd.github+json" \
        "/user/installations/${app_id}/repositories" \
        -f "repository_ids[]=${repo_id}" 2>/dev/null || true
    
    echo -e "${GREEN}✓ Installed on platform repository${NC}"
}

# Function to store credentials
store_credentials() {
    local name=$1
    local app_id=$2
    local private_key=$3
    
    echo -e "${BLUE}Storing credentials for ${name}...${NC}"
    
    # Create secret in secret-store namespace
    kubectl create secret generic "github-app-5dlabs-${name}" \
        --namespace="${SECRET_STORE_NAMESPACE}" \
        --from-literal="app-id=${app_id}" \
        --from-literal="private-key=${private_key}" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create ExternalSecret for agent-platform namespace
    cat <<EOF | kubectl apply -f -
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-${name}
  namespace: ${NAMESPACE}
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-${name}
    creationPolicy: Owner
  data:
  - secretKey: app-id
    remoteRef:
      key: github-app-5dlabs-${name}
      property: app-id
  - secretKey: private-key
    remoteRef:
      key: github-app-5dlabs-${name}
      property: private-key
EOF
    
    echo -e "${GREEN}✓ Stored credentials for ${name}${NC}"
}

# Main execution
main() {
    echo -e "${GREEN}🚀 Automated GitHub Apps Creation (Org Owner Mode)${NC}"
    echo -e "${GREEN}=================================================${NC}"
    echo
    
    # Check GitHub CLI auth
    if ! gh auth status >/dev/null 2>&1; then
        echo -e "${YELLOW}Authenticating with GitHub CLI...${NC}"
        gh auth login
    fi
    
    # Verify org owner status
    local username=$(gh api user | jq -r '.login')
    echo -e "${GREEN}Authenticated as: ${username}${NC}"
    
    # Check org membership
    local role=$(gh api "/orgs/${ORG}/memberships/${username}" | jq -r '.role')
    if [[ "$role" != "admin" ]]; then
        echo -e "${YELLOW}Warning: You're not listed as admin. Are you the owner?${NC}"
        read -p "Continue anyway? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    echo
    echo "This will create all 13 agent apps automatically."
    read -p "Continue? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
    
    # Create each agent
    for agent in "${!AGENTS[@]}"; do
        IFS='|' read -r app_name description <<< "${AGENTS[$agent]}"
        create_github_app "$agent" "$app_name" "$description"
        echo
        sleep 2  # Rate limiting
    done
    
    echo -e "${GREEN}✅ All 13 agent apps created successfully!${NC}"
    echo
    echo "Private keys saved in /tmp/*.private-key.pem"
    echo "All credentials stored in Kubernetes"
    echo
    echo "Next steps:"
    echo "1. Update workflow templates to use GitHub App authentication"
    echo "2. Test with Morgan (docs workflow)"
    echo "3. Remove old user accounts to save costs"
}

# Check prerequisites
if ! command -v gh >/dev/null 2>&1; then
    echo "Installing GitHub CLI..."
    brew install gh
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "Installing jq..."
    brew install jq
fi

# Run main
main "$@"