#!/bin/bash
# Automated GitHub Apps Agent Creation Script
# Creates all 13 agent apps with proper permissions and branding

set -euo pipefail

# Configuration
ORG="5dlabs"
NAMESPACE="agent-platform"
SECRET_STORE_NAMESPACE="secret-store"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Agent definitions
declare -A AGENTS=(
    ["morgan"]="5DLabs-Morgan|Product Management & Documentation Agent|📋"
    ["rex"]="5DLabs-Rex|Senior Backend Architecture Agent|🦖"
    ["blaze"]="5DLabs-Blaze|Performance Optimization Agent|⚡"
    ["scout"]="5DLabs-Scout|Security & Compliance Agent|🔐"
    ["ziggy"]="5DLabs-Ziggy|API & Integration Agent|🔌"
    ["mason"]="5DLabs-Mason|Platform Architecture & DevOps Agent|🏗️"
    ["vigil"]="5DLabs-Vigil|Site Reliability Agent|🛡️"
    ["nimbus"]="5DLabs-Nimbus|Cloud Infrastructure Agent|☁️"
    ["pixel"]="5DLabs-Pixel|Frontend Architecture Agent|🎨"
    ["swipe"]="5DLabs-Swipe|Mobile & Responsive Design Agent|📱"
    ["kit"]="5DLabs-Kit|Design System Agent|🧩"
    ["sherlock"]="5DLabs-Sherlock|QA Strategy Agent|🔍"
    ["otto"]="5DLabs-Otto|Test Automation Agent|🤖"
)

# Function to create GitHub App via API
create_github_app() {
    local name=$1
    local app_name=$2
    local description=$3
    local emoji=$4
    
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
    
    # Create the app using GitHub API
    local response=$(gh api \
        --method POST \
        -H "Accept: application/vnd.github+json" \
        "/orgs/${ORG}/app-manifests" \
        -f manifest="${manifest}")
    
    # Extract app creation URL
    local code=$(echo "$response" | jq -r '.code')
    
    echo -e "${YELLOW}Complete app creation at: https://github.com/organizations/${ORG}/settings/apps/new?code=${code}${NC}"
    echo "This will open in your browser to complete setup..."
    
    # Open browser for final step (GitHub requires this for security)
    if command -v open >/dev/null 2>&1; then
        open "https://github.com/organizations/${ORG}/settings/apps/new?code=${code}"
    elif command -v xdg-open >/dev/null 2>&1; then
        xdg-open "https://github.com/organizations/${ORG}/settings/apps/new?code=${code}"
    fi
    
    echo "Press ENTER after completing the app creation in browser..."
    read -r
    
    # Get the app info
    local app_slug=$(gh api "/orgs/${ORG}/apps" | jq -r ".[] | select(.name == \"${app_name}\") | .slug")
    local app_id=$(gh api "/apps/${app_slug}" | jq -r '.id')
    
    echo -e "${GREEN}✓ Created app: ${app_name} (ID: ${app_id})${NC}"
    
    # Download private key
    echo "Generating private key..."
    local key_response=$(gh api \
        --method POST \
        -H "Accept: application/vnd.github+json" \
        "/apps/${app_slug}/private-keys")
    
    local private_key=$(echo "$key_response" | jq -r '.key')
    
    # Store credentials in Kubernetes secret
    store_app_credentials "$name" "$app_id" "$private_key"
    
    # Install app on repositories
    install_app_on_repos "$app_slug"
}

# Function to store app credentials
store_app_credentials() {
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

# Function to install app on repositories
install_app_on_repos() {
    local app_slug=$1
    
    echo -e "${BLUE}Installing app on platform repository...${NC}"
    
    # Get installation access token
    local installation_id=$(gh api "/orgs/${ORG}/installations" | \
        jq -r ".installations[] | select(.app_slug == \"${app_slug}\") | .id")
    
    if [[ -z "$installation_id" ]]; then
        # Install the app
        gh api \
            --method POST \
            -H "Accept: application/vnd.github+json" \
            "/apps/${app_slug}/installations" \
            -f "repository_ids[]=platform"
    fi
    
    echo -e "${GREEN}✓ Installed on platform repository${NC}"
}

# Function to create system prompt ConfigMap
create_system_prompts() {
    echo -e "${BLUE}Creating system prompts ConfigMap...${NC}"
    
    # This would include all the system prompts from the migration plan
    kubectl create configmap agent-system-prompts \
        --namespace="${NAMESPACE}" \
        --from-file=system-prompts/ \
        --dry-run=client -o yaml | kubectl apply -f -
    
    echo -e "${GREEN}✓ Created system prompts ConfigMap${NC}"
}

# Main execution
main() {
    echo -e "${GREEN}🚀 GitHub Apps Agent Creation Script${NC}"
    echo -e "${GREEN}====================================${NC}"
    echo
    
    # Check prerequisites
    if ! command -v gh >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: GitHub CLI (gh) is not installed${NC}"
        echo "Install with: brew install gh"
        exit 1
    fi
    
    if ! command -v kubectl >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: kubectl is not installed${NC}"
        exit 1
    fi
    
    if ! command -v jq >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: jq is not installed${NC}"
        echo "Install with: brew install jq"
        exit 1
    fi
    
    # Check GitHub CLI auth
    if ! gh auth status >/dev/null 2>&1; then
        echo -e "${YELLOW}GitHub CLI not authenticated. Running 'gh auth login'...${NC}"
        gh auth login
    fi
    
    # Create each agent app
    for agent in "${!AGENTS[@]}"; do
        IFS='|' read -r app_name description emoji <<< "${AGENTS[$agent]}"
        create_github_app "$agent" "$app_name" "$description" "$emoji"
        echo
    done
    
    # Create system prompts
    create_system_prompts
    
    echo -e "${GREEN}✅ All agent apps created successfully!${NC}"
    echo
    echo "Next steps:"
    echo "1. Update workflow templates to use GitHub App authentication"
    echo "2. Test with Morgan (PM agent) first"
    echo "3. Roll out to all agents"
}

# Run main function
main "$@"