#!/bin/bash
# Setup GitHub agent secrets for 5D Labs Platform using External Secrets
# This script creates secrets in the secret-store namespace and ExternalSecrets to sync them

set -euo pipefail

# Default values
SECRET_STORE_NAMESPACE="secret-store"
TARGET_NAMESPACE="agent-platform"
DRY_RUN=""
VERBOSE=""

# Function to show usage
usage() {
    cat << EOF
Setup GitHub agent secrets for 5D Labs Platform using External Secrets

USAGE:
    $0 --user <github-user> --ssh-key <path> --token <token> [OPTIONS]

REQUIRED:
    --user <username>       GitHub username (e.g., 'johnsmith')
    --ssh-key <path>        Path to SSH private key (e.g., '~/.ssh/github_key')
    --token <token>         GitHub personal access token (ghp_xxxx)

OPTIONS:
    --target-namespace <name>    Target namespace for secrets (default: agent-platform)
    --secret-store <name>        Secret store namespace (default: secret-store)
    --dry-run                    Show commands without executing
    --verbose                    Show detailed output
    --help                       Show this help message

EXAMPLES:
    # Setup secrets for user 'johnsmith'
    $0 --user johnsmith --ssh-key ~/.ssh/github_johnsmith --token ghp_abc123

    # Dry run to see what would be created
    $0 --user alice --ssh-key ~/.ssh/alice_github --token ghp_xyz789 --dry-run

    # Setup in different target namespace
    $0 --user bob --ssh-key ~/.ssh/bob --token ghp_def456 --target-namespace my-agents

NOTES:
    - SSH key path should point to the PRIVATE key (public key will be derived)
    - GitHub token needs 'repo' permissions for PR creation
    - Creates secret in secret-store namespace: agent-secrets-<user>
    - Creates ExternalSecret in target namespace to sync the secret
    - Existing secrets will be replaced without warning

EOF
}

# Function for verbose logging
log() {
    if [[ -n "$VERBOSE" ]]; then
        echo "🔧 $1" >&2
    fi
}

# Function for dry run output
dry_run() {
    if [[ -n "$DRY_RUN" ]]; then
        echo "DRY RUN: $1"
        return 0
    fi
    return 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --user)
            GITHUB_USER="$2"
            shift 2
            ;;
        --ssh-key)
            SSH_KEY_PATH="$2"
            shift 2
            ;;
        --token)
            GITHUB_TOKEN="$2"
            shift 2
            ;;
        --target-namespace)
            TARGET_NAMESPACE="$2"
            shift 2
            ;;
        --secret-store)
            SECRET_STORE_NAMESPACE="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN="1"
            shift
            ;;
        --verbose)
            VERBOSE="1"
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo "Error: Unknown option $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "${GITHUB_USER:-}" ]]; then
    echo "Error: --user is required" >&2
    usage >&2
    exit 1
fi

if [[ -z "${SSH_KEY_PATH:-}" ]]; then
    echo "Error: --ssh-key is required" >&2
    usage >&2
    exit 1
fi

if [[ -z "${GITHUB_TOKEN:-}" ]]; then
    echo "Error: --token is required" >&2
    usage >&2
    exit 1
fi

# Expand SSH key path
SSH_KEY_PATH=$(eval echo "$SSH_KEY_PATH")

# Validate SSH key exists
if [[ ! -f "$SSH_KEY_PATH" ]]; then
    echo "Error: SSH private key not found at: $SSH_KEY_PATH" >&2
    exit 1
fi

# Generate public key from private key
SSH_PUBLIC_KEY_PATH="${SSH_KEY_PATH}.pub"
if [[ ! -f "$SSH_PUBLIC_KEY_PATH" ]]; then
    log "Generating public key from private key"
    if ! dry_run "ssh-keygen -y -f '$SSH_KEY_PATH' > '$SSH_PUBLIC_KEY_PATH'"; then
        ssh-keygen -y -f "$SSH_KEY_PATH" > "$SSH_PUBLIC_KEY_PATH"
    fi
fi

# Read keys
SSH_PRIVATE_KEY=$(cat "$SSH_KEY_PATH")
SSH_PUBLIC_KEY=$(cat "$SSH_PUBLIC_KEY_PATH")

# Secret names
SECRET_NAME="agent-secrets-$GITHUB_USER"
EXTERNAL_SECRET_NAME="agent-secrets-$GITHUB_USER"

log "Setting up secrets for user: $GITHUB_USER"
log "Secret store namespace: $SECRET_STORE_NAMESPACE"
log "Target namespace: $TARGET_NAMESPACE"
log "SSH private key: $SSH_KEY_PATH"
log "SSH public key: $SSH_PUBLIC_KEY_PATH"

# Create or update secret in secret-store namespace
log "Creating secret in secret-store namespace: $SECRET_NAME"

if dry_run "kubectl create secret generic '$SECRET_NAME' in namespace '$SECRET_STORE_NAMESPACE'"; then
    cat << EOF
kubectl create secret generic "$SECRET_NAME" \\
    --namespace="$SECRET_STORE_NAMESPACE" \\
    --from-literal="SSH_PRIVATE_KEY=<SSH_PRIVATE_KEY>" \\
    --from-literal="SSH_PUBLIC_KEY=<SSH_PUBLIC_KEY>" \\
    --from-literal="GITHUB_TOKEN=$GITHUB_TOKEN" \\
    --dry-run=client -o yaml | kubectl apply -f -
EOF
else
    kubectl create secret generic "$SECRET_NAME" \
        --namespace="$SECRET_STORE_NAMESPACE" \
        --from-literal="SSH_PRIVATE_KEY=$SSH_PRIVATE_KEY" \
        --from-literal="SSH_PUBLIC_KEY=$SSH_PUBLIC_KEY" \
        --from-literal="GITHUB_TOKEN=$GITHUB_TOKEN" \
        --dry-run=client -o yaml | kubectl apply -f -
fi

# Create ExternalSecret in target namespace
log "Creating ExternalSecret in target namespace: $EXTERNAL_SECRET_NAME"

EXTERNAL_SECRET_YAML=$(cat << EOF
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: $EXTERNAL_SECRET_NAME
  namespace: $TARGET_NAMESPACE
  labels:
    managed-by: "agent-secrets-script"
    github-user: "$GITHUB_USER"
spec:
  refreshInterval: 30s
  secretStoreRef:
    name: secret-store
    kind: ClusterSecretStore
  target:
    name: $EXTERNAL_SECRET_NAME
    creationPolicy: Owner
  data:
  - secretKey: SSH_PRIVATE_KEY
    remoteRef:
      key: $SECRET_NAME
      property: SSH_PRIVATE_KEY
  - secretKey: SSH_PUBLIC_KEY
    remoteRef:
      key: $SECRET_NAME
      property: SSH_PUBLIC_KEY
  - secretKey: GITHUB_TOKEN
    remoteRef:
      key: $SECRET_NAME
      property: GITHUB_TOKEN
EOF
)

if dry_run "kubectl apply ExternalSecret '$EXTERNAL_SECRET_NAME'"; then
    echo "DRY RUN: Would create ExternalSecret:"
    echo "$EXTERNAL_SECRET_YAML"
else
    echo "$EXTERNAL_SECRET_YAML" | kubectl apply -f -
fi

# Wait for ExternalSecret to sync
if [[ -z "$DRY_RUN" ]]; then
    log "Waiting for ExternalSecret to sync..."
    kubectl wait --for=condition=Ready externalsecret "$EXTERNAL_SECRET_NAME" \
        --namespace="$TARGET_NAMESPACE" \
        --timeout=60s || {
        echo "Warning: ExternalSecret did not become ready within 60s"
        echo "Check the status with: kubectl describe externalsecret $EXTERNAL_SECRET_NAME -n $TARGET_NAMESPACE"
    }
fi

echo "✅ Successfully configured agent secrets for user: $GITHUB_USER"
echo "📍 Secret in secret-store: $SECRET_STORE_NAMESPACE/$SECRET_NAME"
echo "📍 ExternalSecret in target: $TARGET_NAMESPACE/$EXTERNAL_SECRET_NAME"
echo "📍 Synced secret in target: $TARGET_NAMESPACE/$EXTERNAL_SECRET_NAME"

if [[ -z "$DRY_RUN" ]]; then
    echo ""
    echo "🔍 Verification:"
    echo "kubectl get secret $EXTERNAL_SECRET_NAME -n $TARGET_NAMESPACE"
    echo "kubectl describe externalsecret $EXTERNAL_SECRET_NAME -n $TARGET_NAMESPACE"
fi