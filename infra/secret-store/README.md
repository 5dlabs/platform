# Secret Store Setup

This directory contains the setup for External Secrets with a Kubernetes secret store backend.

## Manual Setup Required

After deploying the secret-store application, you need to manually create the GitHub PAT secret:

```bash
# Create the GitHub PAT secret with your actual token
kubectl create secret generic github-pat \
  -n secret-store \
  --from-literal=token=ghp_YOUR_ACTUAL_GITHUB_PAT_HERE

# Verify the secret was created
kubectl get secret github-pat -n secret-store -o yaml
```

## GitHub PAT Requirements

The PAT needs the following permissions for ARC:
- `actions:read` - To read workflow runs
- `administration:read` - To read runner groups (if using)
- `metadata:read` - Basic repository metadata
- `Self-hosted runners:write` - To register and manage runners

## How it Works

1. **secret-store namespace** - Contains the source GitHub PAT secret
2. **External Secrets** - Pulls the PAT from secret-store and creates secrets in arc-system and arc-runners namespaces
3. **ARC Controller** - Uses the secret from arc-system namespace
4. **ARC Runners** - Use the secret from arc-runners namespace

## Files

- `github-secrets.yaml` - Creates namespace, RBAC, and placeholder secret
- `arc-external-secrets.yaml` - External Secrets configuration to sync PAT to ARC namespaces
- `rust-cache-pvc.yaml` - PVC for Rust build caching in runners
- `README.md` - This file

## Verification

Check that External Secrets are working:

```bash
# Check SecretStores
kubectl get secretstore -n arc-system
kubectl get secretstore -n arc-runners

# Check ExternalSecrets
kubectl get externalsecret -n arc-system
kubectl get externalsecret -n arc-runners

# Check that secrets were created
kubectl get secret github-pat -n arc-system
kubectl get secret github-pat -n arc-runners
```