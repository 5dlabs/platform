#!/bin/bash

# Script to update GitHub PAT for ARC authentication
# Usage: ./scripts/update-github-pat.sh <github_pat>

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if PAT is provided
if [ $# -eq 0 ]; then
    print_error "GitHub PAT is required"
    echo "Usage: $0 <github_pat>"
    echo ""
    echo "Required GitHub PAT scopes:"
    echo "  - repo (Full control of private repositories)"
    echo "  - admin:org (Full control of orgs and teams)"
    echo "  - admin:public_key (Full control of user public keys)"
    echo "  - admin:repo_hook (Full control of repository hooks)"
    echo "  - admin:org_hook (Full control of organization hooks)"
    exit 1
fi

PAT="$1"

# Validate PAT format (GitHub PATs start with ghp_, gho_, ghu_, ghs_, or ghr_)
if [[ ! "$PAT" =~ ^gh[psuor]_[A-Za-z0-9_]{36,255}$ ]]; then
    print_error "Invalid GitHub PAT format"
    echo "GitHub PATs should start with ghp_, gho_, ghu_, ghs_, or ghr_ followed by alphanumeric characters"
    exit 1
fi

print_status "Updating GitHub PAT secret..."

# Update the secret in secret-store namespace
kubectl create secret generic github-pat -n secret-store \
  --from-literal=token="$PAT" \
  --dry-run=client -o yaml | kubectl apply -f -

if [ $? -ne 0 ]; then
    print_error "Failed to update secret"
    exit 1
fi

print_status "Adding ArgoCD ignore annotations..."

# Add ArgoCD ignore annotations to prevent ArgoCD from managing this secret
kubectl annotate secret github-pat -n secret-store \
  argocd.argoproj.io/compare=false \
  argocd.argoproj.io/sync=false \
  --overwrite

if [ $? -ne 0 ]; then
    print_warning "Failed to add ArgoCD annotations, but secret was updated"
fi

print_status "Waiting for External Secrets to sync..."

# Wait for External Secrets to sync (they refresh every 30s)
sleep 5

# Check if External Secrets are working
print_status "Checking External Secrets status..."
EXTERNAL_SECRETS_STATUS=$(kubectl get externalsecrets -A -o jsonpath='{range .items[?(@.metadata.name=="github-pat")]}{.metadata.namespace}{" "}{.status.conditions[0].status}{"\n"}{end}')

if echo "$EXTERNAL_SECRETS_STATUS" | grep -q "False"; then
    print_warning "Some External Secrets are not syncing properly"
    kubectl get externalsecrets -A -o wide | grep github-pat
else
    print_status "External Secrets are syncing properly"
fi

# Verify the secret was updated in ARC namespaces
print_status "Verifying secrets in ARC namespaces..."

for namespace in arc-systems arc-runners; do
    if kubectl get secret github-pat -n "$namespace" >/dev/null 2>&1; then
        TOKEN_LENGTH=$(kubectl get secret github-pat -n "$namespace" -o jsonpath='{.data.github_token}' | base64 -d | wc -c | tr -d ' ')
        if [ "$TOKEN_LENGTH" -gt 30 ]; then
            print_status "✓ Secret updated in $namespace namespace (${TOKEN_LENGTH} chars)"
        else
            print_warning "✗ Secret in $namespace namespace seems too short (${TOKEN_LENGTH} chars)"
        fi
    else
        print_warning "✗ Secret not found in $namespace namespace"
    fi
done

print_status "Checking ARC controller logs for errors..."

# Check recent ARC controller logs for authentication errors
sleep 10
RECENT_ERRORS=$(kubectl logs -n arc-systems deployment/arc-gha-rs-controller --tail=5 --since=1m 2>/dev/null | grep -i "error\|failed" || true)

if [ -n "$RECENT_ERRORS" ]; then
    print_warning "Recent errors found in ARC controller logs:"
    echo "$RECENT_ERRORS"
    echo ""
    print_warning "If you see 'Bad credentials' errors, the PAT may need additional permissions"
else
    print_status "No recent errors in ARC controller logs"
fi

print_status "GitHub PAT update complete!"
print_status "External Secrets will sync the new PAT within 30 seconds"
print_status "Monitor ARC controller logs: kubectl logs -n arc-systems deployment/arc-gha-rs-controller -f"

echo ""
print_status "To check if runners are working:"
echo "  kubectl get autoscalingrunnersets -n arc-runners -o wide"
echo "  kubectl get pods -n arc-runners"