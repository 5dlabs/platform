# Orchestrator Helm Chart

A Helm chart for deploying the Platform Orchestrator that manages Claude Code agents and task execution across Kubernetes clusters.

## Overview

The Orchestrator is a Rust-based service that:
- Processes PM task submissions via REST API
- Deploys Claude Code jobs to Kubernetes clusters
- Manages shared workspace and agent coordination
- Handles webhook events from GitHub
- Orchestrates multi-agent collaboration

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- Persistent Volume provisioner (for shared workspace)
- Container registry access for pulling images

## Installation

### Quick Start

```bash
# Add the chart repository (if using a helm repo)
# helm repo add platform https://charts.5dlabs.com
# helm repo update

# Install with default values
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --create-namespace \
  --set secrets.anthropicApiKey="your-anthropic-api-key" \
  --set secrets.githubToken="your-github-token"
```

### Production Installation

```bash
# Create a values file for production
cat > orchestrator-prod-values.yaml << EOF
image:
  tag: "v1.0.0"  # Use specific version tag

secrets:
  anthropicApiKey: "your-anthropic-api-key"
  githubToken: "your-github-token"

ingress:
  enabled: true
  hosts:
    - host: orchestrator.yourdomain.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: orchestrator-tls
      hosts:
        - orchestrator.yourdomain.com

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 200m
    memory: 256Mi

# Enable shared workspace PVC creation
sharedWorkspace:
  enabled: true
  size: "100Gi"
  storageClass: "fast-ssd"
EOF

# Install with production values
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --create-namespace \
  --values orchestrator-prod-values.yaml
```

## Configuration

### Required Configuration

| Parameter | Description | Required |
|-----------|-------------|----------|
| `secrets.anthropicApiKey` | Anthropic API key for Claude agents | Yes |
| `secrets.githubToken` | GitHub token for repository access | Yes |

### Common Configuration Options

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of orchestrator replicas | `1` |
| `image.repository` | Container image repository | `ghcr.io/5dlabs/platform/orchestrator` |
| `image.tag` | Container image tag | `"latest"` |
| `image.pullPolicy` | Image pull policy | `Always` |

### Service Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `service.type` | Kubernetes service type | `ClusterIP` |
| `service.port` | Service port | `80` |
| `service.targetPort` | Container port | `8080` |

### Ingress Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class name | `"nginx"` |
| `ingress.hosts[0].host` | Hostname | `orchestrator.local` |

### RBAC Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `rbac.create` | Create RBAC resources | `true` |
| `rbac.namespaced` | Use Role/RoleBinding instead of ClusterRole | `true` |

### Claude Code Chart Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `claudeCodeChart.enabled` | Include Claude Code Helm chart | `true` |
| `claudeCodeChart.mountPath` | Mount path for chart files | `"/infra"` |

### Shared Workspace Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `sharedWorkspace.enabled` | Create shared workspace PVC | `false` |
| `sharedWorkspace.name` | PVC name | `"shared-workspace-pvc"` |
| `sharedWorkspace.storageClass` | Storage class | `"local-path"` |
| `sharedWorkspace.size` | Storage size | `"50Gi"` |

## Upgrading

```bash
# Upgrade to a new version
helm upgrade orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values your-values.yaml

# Upgrade with new configuration
helm upgrade orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --set image.tag="v1.1.0" \
  --reuse-values
```

## Uninstalling

```bash
# Uninstall the release
helm uninstall orchestrator --namespace orchestrator

# Optionally delete the namespace
kubectl delete namespace orchestrator
```

## Monitoring and Troubleshooting

### Health Checks

The orchestrator exposes a health check endpoint at `/health`:

```bash
# Check health via port-forward
kubectl port-forward -n orchestrator svc/orchestrator 8080:80
curl http://localhost:8080/health

# Check health via ingress (if enabled)
curl http://orchestrator.local/health
```

### Logs

```bash
# View orchestrator logs
kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator -f

# View logs from specific pod
kubectl logs -n orchestrator deployment/orchestrator -f
```

### Common Issues

1. **Orchestrator pod not starting**
   - Check if API keys are properly set
   - Verify image pull secrets are configured
   - Check resource limits and node capacity

2. **Claude Code deployments failing**
   - Verify RBAC permissions
   - Check if shared workspace PVC exists
   - Ensure Claude Code Helm chart is properly mounted

3. **Ingress not working**
   - Verify ingress controller is installed
   - Check DNS resolution for ingress host
   - Verify TLS certificates (if using HTTPS)

## Development

### Local Development

```bash
# Install with development values
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --create-namespace \
  --set image.pullPolicy=IfNotPresent \
  --set secrets.anthropicApiKey="test-key" \
  --set secrets.githubToken="test-token"
```

### Testing

```bash
# Lint the chart
helm lint ./infra/orchestrator-chart

# Render templates locally
helm template orchestrator ./infra/orchestrator-chart \
  --values test-values.yaml

# Test installation
helm install --dry-run --debug orchestrator ./infra/orchestrator-chart
```

## Architecture

The orchestrator consists of:

1. **Deployment**: Main orchestrator service
2. **Service**: ClusterIP service for internal communication
3. **Ingress**: External access (optional)
4. **ConfigMap**: Configuration settings
5. **Secret**: API keys and sensitive data
6. **ServiceAccount**: Kubernetes service account
7. **RBAC**: Role and RoleBinding for permissions
8. **Claude Code Chart ConfigMap**: Embedded Helm chart for agents

## Contributing

1. Make changes to the chart templates
2. Update the Chart.yaml version
3. Test the changes locally
4. Submit a pull request

## License

This chart is part of the Platform project and follows the same license terms.