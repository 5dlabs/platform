# Actions Runner Controller (ARC) Setup

This directory contains the configuration for GitHub Actions self-hosted runners using ARC.

## Quick Start

1. **Install ARC Controller**:
   ```bash
   # Add Helm repository
   helm repo add actions-runner-controller https://actions-runner-controller.github.io/actions-runner-controller
   helm repo update

   # Install cert-manager (required)
   kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.0/cert-manager.yaml
   kubectl wait --for=condition=ready pod -l app.kubernetes.io/instance=cert-manager -n cert-manager --timeout=300s

   # Install ARC
   helm install arc actions-runner-controller/actions-runner-controller \
     --namespace arc-systems \
     --create-namespace \
     --set syncPeriod=1m
   ```

2. **Create GitHub Token Secret**:
   ```bash
   # Personal Access Token (PAT) with admin:org scope
   kubectl create secret generic arc-github-token \
     --from-literal=github_token=YOUR_PAT_WITH_ADMIN_ORG_SCOPE \
     -n arc-systems
   ```

3. **Deploy Organization Runners**:
   ```bash
   kubectl apply -f arc-org-runners.yaml
   ```

## File Structure

- `arc-org-runners.yaml` - Complete organization runner setup (namespace, RBAC, and runners)
- `setup-org-runners.md` - Detailed guide for organization runners

## Organization Runners

- **Labels**: `self-hosted`, `linux`, `x64`, `k8s-runner`
- **Scope**: Available to all repositories in the 5dlabs organization
- **View**: https://github.com/organizations/5dlabs/settings/actions/runners
- **Usage**: `runs-on: [self-hosted]`
- **Replicas**: 4 runners with 4 CPU/8Gi memory limits

## Monitoring

```bash
# View all runners
kubectl get runners -n arc-systems

# Check runner pods
kubectl get pods -n arc-systems

# View logs
kubectl logs -n arc-systems -l runner-deployment-name=org-runners
kubectl logs -n arc-systems deployment/arc-actions-runner-controller
```

## Troubleshooting

1. **Runners not appearing in GitHub**:
   - Check token permissions (needs `admin:org` for org runners)
   - Verify with: `kubectl logs -n arc-systems deployment/arc-actions-runner-controller`

2. **Jobs queued but not picked up**:
   - Check runner labels match workflow `runs-on`
   - Ensure runners show "Listening for Jobs" in logs

3. **kubectl/helm not found in workflows**:
   - Tools are installed in `/shared/`
   - Workflow steps should copy to `$HOME/bin` or use full path

## Security Notes

- GitHub Secret Scanning is enabled on the repository
- Pre-commit hook provides basic quality checks (no custom secret scanning needed)
- Runners use ephemeral mode - they restart after each job for security
- All runners use Docker-in-Docker for container operations
