# helm-charts Analysis

**Path:** `infra/charts`
**Type:** HelmChart
**Lines of Code:** 1739
**Description:** helm-charts configuration and files

## Source Files

### orchestrator/crds/platform-crds.yaml (269 lines)

**Full Content:**
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: coderuns.orchestrator.platform
spec:
  group: orchestrator.platform
  scope: Namespaced
  names:
    plural: coderuns
    singular: coderun
    kind: CodeRun
    shortNames:
    - cr
  versions:
  - name: v1
    served: true
    storage: true
    subresources:
      status: {}
    additionalPrinterColumns:
    - name: Task
      type: integer
      jsonPath: .spec.taskId
    - name: Service
      type: string
      jsonPath: .spec.service
    - name: Model
      type: string
      jsonPath: .spec.model
    - name: Phase
      type: string
      jsonPath: .status.phase
    - name: Age
      type: date
      jsonPath: .metadata.creationTimestamp
    schema:
      openAPIV3Schema:
        type: object
        required: ["spec"]
        properties:
          spec:
            type: object
            required: ["taskId", "service", "repositoryUrl", "docsRepositoryUrl", "workingDirectory", "model", "githubUser"]
            properties:
              taskId:
                type: integer
                description: "Task ID to implement"
              service:
                type: string
                description: "Target service name"
              repositoryUrl:
                type: string
                description: "Target project repository URL (where implementation work happens)"
              docsRepositoryUrl:
                type: string
                description: "Documentation repository URL (where Task Master definitions come from)"
              docsProjectDirectory:
                type: string
                description: "Project directory within docs repository (e.g. '_projects/simple-api')"
              docsBranch:
                type: string
                default: "main"
                description: "Docs branch to use (e.g., 'main', 'feature/branch')"
              workingDirectory:
                type: string
                description: "Working directory within target repository (defaults to service name if not specified)"
              model:
                type: string
                description: "Claude model to use (full model name like 'claude-3-5-sonnet-20241022')"
              githubUser:
                type: string
                description: "GitHub username for authentication and commits"
              localTools:
                type: string
                description: "Local MCP tools/servers to enable (comma-separated)"
              remoteTools:
                type: string
                description: "Remote MCP tools/servers to enable (comma-separated)"
              contextVersion:
                type: integer
                default: 1
                description: "Context version for retry attempts (incremented on each retry)"
              promptModification:
                type: string
                description: "Additional context for retry attempts"
              continueSession:
                type: boolean
                default: false
                description: "Whether to continue a previous session"
              overwriteMemory:
                type: boolean
                default: false
                description: "Whether to overwrite memory before starting"
              env:
                type: object
                additionalProperties:
                  type: string
                description: "Environment variables to set in the container"
              envFromSecrets:
                type: array
                items:
                  type: object
                  properties:
                    name:
                      type: string
                      description: "Name of the environment variable"
                    secretName:
                      type: string
                      description: "Name of the secret"
                    secretKey:
                      type: string
                      description: "Key within the secret"
                  required:
                    - name
                    - secretName
                    - secretKey
                description: "Environment variables from secrets"
          status:
            type: object
            properties:
              phase:
                type: string
                description: "Current phase of the code implementation"
              message:
                type: string
                description: "Human-readable message about the current state"
              lastUpdate:
                type: string
                description: "Timestamp when this phase was reached"
              jobName:
                type: string
                description: "Associated Kubernetes Job name"
              pullRequestUrl:
                type: string
                description: "Pull request URL if created"
              retryCount:
                type: integer
                description: "Current retry attempt (if applicable)"
              conditions:
                type: array
                description: "Conditions for the CodeRun"
                items:
                  type: object
                  required: ["type", "status"]
                  properties:
                    type:
                      type: string
                      description: "Type of condition"
                    status:
                      type: string
                      description: "Status of the condition (True, False, or Unknown)"
                    lastTransitionTime:
                      type: string
                      description: "Last time the condition transitioned (RFC3339 format)"
                    reason:
                      type: string
                      description: "Reason for the condition's last transition"
                    message:
                      type: string
                      description: "Human-readable message about the condition"
              configmapName:
                type: string
                description: "Name of the ConfigMap containing the prompt and context"
              contextVersion:
                type: integer
                description: "Version of the context and prompt used"
              promptModification:
                type: string
                description: "Modification to the prompt if any"
              promptMode:
                type: string
                description: "Mode of prompt (e.g., direct, indirect)"
              sessionId:
                type: string
                description: "Session ID for tracking"
---
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: docsruns.orchestrator.platform
spec:
  group: orchestrator.platform
  scope: Namespaced
  names:
    plural: docsruns
    singular: docsrun
    kind: DocsRun
    shortNames:
    - dr
  versions:
  - name: v1
    served: true
    storage: true
    subresources:
      status: {}
    additionalPrinterColumns:
    - name: Phase
      type: string
      jsonPath: .status.phase
    - name: Age
      type: date
      jsonPath: .metadata.creationTimestamp
    schema:
      openAPIV3Schema:
        type: object
        required: ["spec"]
        properties:
          spec:
            type: object
            required: ["repositoryUrl", "workingDirectory", "sourceBranch", "model", "githubUser"]
            properties:
              repositoryUrl:
                type: string
                description: "Repository URL for documentation generation"
              workingDirectory:
                type: string
                description: "Working directory within repository"
              sourceBranch:
                type: string
                description: "Source branch to analyze"
              model:
                type: string
                description: "Claude model to use (full model name like 'claude-3-5-sonnet-20241022')"
              githubUser:
                type: string
                description: "GitHub username for authentication and commits"
          status:
            type: object
            properties:
              phase:
                type: string
                description: "Current phase of the documentation generation"
              message:
                type: string
                description: "Human-readable message about the current state"
              lastUpdate:
                type: string
                description: "Timestamp when this phase was reached"
              jobName:
                type: string
                description: "Associated Kubernetes Job name"
              pullRequestUrl:
                type: string
                description: "Pull request URL if created"
              conditions:
                type: array
                description: "Conditions for the DocsRun"
                items:
                  type: object
                  required: ["type", "status"]
                  properties:
                    type:
                      type: string
                      description: "Type of condition"
                    status:
                      type: string
                      description: "Status of the condition (True, False, or Unknown)"
                    lastTransitionTime:
                      type: string
                      description: "Last time the condition transitioned (RFC3339 format)"
                    reason:
                      type: string
                      description: "Reason for the condition's last transition"
                    message:
                      type: string
                      description: "Human-readable message about the condition"
              configmapName:
                type: string
                description: "Name of the ConfigMap containing the prompt and context"
```

### orchestrator/crds/coderun-crd.yaml (175 lines)

**Full Content:**
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: coderuns.orchestrator.platform
spec:
  group: orchestrator.platform
  scope: Namespaced
  names:
    plural: coderuns
    singular: coderun
    kind: CodeRun
    shortNames:
    - cr
  versions:
  - name: v1
    served: true
    storage: true
    subresources:
      status: {}
    additionalPrinterColumns:
    - name: Task
      type: integer
      jsonPath: .spec.taskId
    - name: Service
      type: string
      jsonPath: .spec.service
    - name: Model
      type: string
      jsonPath: .spec.model
    - name: Phase
      type: string
      jsonPath: .status.phase
    - name: Age
      type: date
      jsonPath: .metadata.creationTimestamp
    schema:
      openAPIV3Schema:
        type: object
        required: ["spec"]
        properties:
          spec:
            type: object
            required: ["taskId", "service", "repositoryUrl", "docsRepositoryUrl", "workingDirectory", "model", "githubUser"]
            properties:
              taskId:
                type: integer
                description: "Task ID to implement"
              service:
                type: string
                description: "Target service name"
              repositoryUrl:
                type: string
                description: "Target project repository URL (where implementation work happens)"
              docsRepositoryUrl:
                type: string
                description: "Documentation repository URL (where Task Master definitions come from)"
              docsProjectDirectory:
                type: string
                description: "Project directory within docs repository (e.g. '_projects/simple-api')"
              docsBranch:
                type: string
                default: "main"
                description: "Docs branch to use (e.g., 'main', 'feature/branch')"
              workingDirectory:
                type: string
                description: "Working directory within target repository (defaults to service name if not specified)"
              model:
                type: string
                description: "Claude model to use (full model name like 'claude-3-5-sonnet-20241022')"
              githubUser:
                type: string
                description: "GitHub username for authentication and commits"
              localTools:
                type: string
                description: "Local MCP tools/servers to enable (comma-separated)"
              remoteTools:
                type: string
                description: "Remote MCP tools/servers to enable (comma-separated)"
              contextVersion:
                type: integer
                default: 1
                description: "Context version for retry attempts (incremented on each retry)"
              promptModification:
                type: string
                description: "Additional context for retry attempts"
              continueSession:
                type: boolean
                default: false
                description: "Whether to continue a previous session"
              overwriteMemory:
                type: boolean
                default: false
                description: "Whether to overwrite memory before starting"
              env:
                type: object
                additionalProperties:
                  type: string
                description: "Environment variables to set in the container"
              envFromSecrets:
                type: array
                items:
                  type: object
                  properties:
                    name:
                      type: string
                      description: "Name of the environment variable"
                    secretName:
                      type: string
                      description: "Name of the secret"
                    secretKey:
                      type: string
                      description: "Key within the secret"
                  required:
                    - name
                    - secretName
                    - secretKey
                description: "Environment variables from secrets"
          status:
            type: object
            properties:
              phase:
                type: string
                description: "Current phase of the code implementation"
              message:
                type: string
                description: "Human-readable message about the current state"
              lastUpdate:
                type: string
                description: "Timestamp when this phase was reached"
              jobName:
                type: string
                description: "Associated Kubernetes Job name"
              pullRequestUrl:
                type: string
                description: "Pull request URL if created"
              retryCount:
                type: integer
                description: "Current retry attempt (if applicable)"
              conditions:
                type: array
                description: "Conditions for the CodeRun"
                items:
                  type: object
                  required: ["type", "status"]
                  properties:
                    type:
                      type: string
                      description: "Type of condition"
                    status:
                      type: string
                      description: "Status of the condition (True, False, or Unknown)"
                    lastTransitionTime:
                      type: string
                      description: "Last time the condition transitioned (RFC3339 format)"
                    reason:
                      type: string
                      description: "Reason for the condition's last transition"
                    message:
                      type: string
                      description: "Human-readable message about the condition"
              configmapName:
                type: string
                description: "Name of the ConfigMap containing the prompt and context"
              contextVersion:
                type: integer
                description: "Version of the context and prompt used"
              promptModification:
                type: string
                description: "Modification to the prompt if any"
              promptMode:
                type: string
                description: "Mode of prompt (e.g., direct, indirect)"
              sessionId:
                type: string
                description: "Session ID for tracking"
```

### orchestrator/crds/docsrun-crd.yaml (93 lines)

**Full Content:**
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: docsruns.orchestrator.platform
spec:
  group: orchestrator.platform
  scope: Namespaced
  names:
    plural: docsruns
    singular: docsrun
    kind: DocsRun
    shortNames:
    - dr
  versions:
  - name: v1
    served: true
    storage: true
    subresources:
      status: {}
    additionalPrinterColumns:
    - name: Phase
      type: string
      jsonPath: .status.phase
    - name: Age
      type: date
      jsonPath: .metadata.creationTimestamp
    schema:
      openAPIV3Schema:
        type: object
        required: ["spec"]
        properties:
          spec:
            type: object
            required: ["repositoryUrl", "workingDirectory", "sourceBranch", "model", "githubUser"]
            properties:
              repositoryUrl:
                type: string
                description: "Repository URL for documentation generation"
              workingDirectory:
                type: string
                description: "Working directory within repository"
              sourceBranch:
                type: string
                description: "Source branch to analyze"
              model:
                type: string
                description: "Claude model to use (full model name like 'claude-3-5-sonnet-20241022')"
              githubUser:
                type: string
                description: "GitHub username for authentication and commits"
          status:
            type: object
            properties:
              phase:
                type: string
                description: "Current phase of the documentation generation"
              message:
                type: string
                description: "Human-readable message about the current state"
              lastUpdate:
                type: string
                description: "Timestamp when this phase was reached"
              jobName:
                type: string
                description: "Associated Kubernetes Job name"
              pullRequestUrl:
                type: string
                description: "Pull request URL if created"
              conditions:
                type: array
                description: "Conditions for the DocsRun"
                items:
                  type: object
                  required: ["type", "status"]
                  properties:
                    type:
                      type: string
                      description: "Type of condition"
                    status:
                      type: string
                      description: "Status of the condition (True, False, or Unknown)"
                    lastTransitionTime:
                      type: string
                      description: "Last time the condition transitioned (RFC3339 format)"
                    reason:
                      type: string
                      description: "Reason for the condition's last transition"
                    message:
                      type: string
                      description: "Human-readable message about the condition"
              configmapName:
                type: string
                description: "Name of the ConfigMap containing the prompt and context"
```

### orchestrator/Chart.yaml (23 lines)

**Full Content:**
```yaml
apiVersion: v2
name: orchestrator
description: A Helm chart for the Platform Orchestrator - manages Claude Code agents via TaskRun CRDs
type: application
version: 0.1.1
appVersion: "latest"

keywords:
  - orchestrator
  - claude-code
  - task-management
  - kubernetes
  - automation

home: https://github.com/5dlabs/platform
maintainers:
  - name: Platform Team
    email: platform@5dlabs.com

sources:
  - https://github.com/5dlabs/platform

dependencies: []
```

### orchestrator/README.md (310 lines)

**Full Content:**
```md
# Orchestrator Helm Chart

A Helm chart for deploying the Platform Orchestrator that manages Claude Code agents and task execution across Kubernetes clusters.

## Overview

The Orchestrator is a Rust-based service that:
- Processes PM task submissions via REST API
- Deploys Claude Code jobs to Kubernetes clusters
- Manages per-service workspaces and agent coordination
- Handles webhook events from GitHub
- Orchestrates multi-agent collaboration

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- Persistent Volume provisioner (for per-service workspaces)
- Container registry access for pulling images

## Installation

### Using the Helm Repository (Recommended)

Add the 5dlabs Helm repository to install the orchestrator chart:

```bash
# Add the Helm repository
helm repo add 5dlabs https://5dlabs.github.io/platform
helm repo update

# Install the orchestrator chart
helm install orchestrator 5dlabs/orchestrator --namespace orchestrator --create-namespace
```

**Note**: CRDs are not included in the Helm chart and must be installed separately (see Step 1 below).

### Manual Installation from Source

If you prefer to install from the source repository:

### Step 1: Install Custom Resource Definitions (CRDs)

**⚠️ Important**: CRDs must be installed before the Helm chart.

```bash
# Install CRDs from GitHub (recommended)
kubectl apply -f https://raw.githubusercontent.com/5dlabs/platform/main/infra/charts/orchestrator/crds/platform-crds.yaml

# Or install from local files
kubectl apply -f crds/
```

### Step 2: Setup GitHub Agent Secrets

**⚠️ Important**: Each agent needs SSH keys and GitHub tokens configured externally.

```bash
# Setup secrets for your GitHub user
./infra/scripts/setup-agent-secrets.sh \
  --user your-github-username \
  --ssh-key ~/.ssh/your_github_key \
  --token ghp_your_personal_access_token

# Setup additional agents (repeat as needed)
./infra/scripts/setup-agent-secrets.sh \
  --user another-user \
  --ssh-key ~/.ssh/another_key \
  --token ghp_another_token
```

**Requirements:**
- SSH key pair for each GitHub user (private key + `.pub` file)
- GitHub Personal Access Token with `repo` permissions
- SSH key must be added to the GitHub account

### Step 3: Install the Helm Chart

#### Quick Start

```bash
# Add the chart repository (if using a helm repo)
# helm repo add platform https://charts.5dlabs.com
# helm repo update

# Install with default values
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --create-namespace \
  --set secrets.anthropicApiKey="your-anthropic-api-key"
```

#### Production Installation

```bash
# 1. First install CRDs and setup agent secrets (if not already done)
kubectl apply -f https://raw.githubusercontent.com/5dlabs/platform/main/infra/charts/orchestrator/crds/platform-crds.yaml
./infra/scripts/setup-agent-secrets.sh --user your-user --ssh-key ~/.ssh/key --token ghp_xxx

# 2. Create a values file for production
cat > orchestrator-prod-values.yaml << EOF
image:
  tag: "v1.0.0"  # Use specific version tag

secrets:
  anthropicApiKey: "your-anthropic-api-key"

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

# Per-service workspace configuration
storage:
  storageClassName: "fast-ssd"
  workspaceSize: "100Gi"
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
| `storage.storageClassName` | Storage class for workspace PVCs | `"local-path"` |
| `storage.workspaceSize` | Size for workspace PVCs | `"10Gi"` |

### Workspace Management

Workspaces are automatically created per-service as PVCs named `workspace-{service}`. Each CodeRun gets its own isolated workspace, while DocsRuns use ephemeral storage.

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
   - Check if service workspace PVCs exist (`kubectl get pvc | grep workspace-`)
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
```

### orchestrator/DEPLOYMENT.md (299 lines)

**Full Content:**
```md
# Orchestrator Deployment Guide

This guide walks you through deploying the orchestrator using the Helm chart and migrating from the existing manifest-based deployment.

## Current State Analysis

Based on the current deployment in the `orchestrator` namespace, the following resources are currently deployed via manifests:

- **Deployment**: `orchestrator` (1 replica)
- **Service**: `orchestrator` (ClusterIP)
- **ConfigMap**: `orchestrator-config`
- **Secret**: `orchestrator-secrets`
- **ConfigMap**: `claude-code-helm-chart` (contains Claude Code chart files)
- **ServiceAccount**: `orchestrator`
- **RBAC**: Role and RoleBinding for orchestrator permissions

## Migration Strategy

### Option 1: In-Place Migration (Recommended)

This approach updates the existing deployment with minimal downtime.

#### Step 1: Backup Current Configuration

```bash
# Export current configuration
kubectl get configmap orchestrator-config -n orchestrator -o yaml > backup-config.yaml
kubectl get secret orchestrator-secrets -n orchestrator -o yaml > backup-secrets.yaml

# Extract current API keys (base64 decode them)
kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.ANTHROPIC_API_KEY}' | base64 -d
kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.GITHUB_TOKEN}' | base64 -d
```

#### Step 2: Create Values File

```bash
# Create values file with current configuration
cat > orchestrator-values.yaml << EOF
secrets:
  anthropicApiKey: "$(kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.ANTHROPIC_API_KEY}' | base64 -d)"
  githubToken: "$(kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.GITHUB_TOKEN}' | base64 -d)"

config:
  kubernetesNamespace: "orchestrator"
  helmChartPath: "/infra/claude-code"
  helmNamespace: "orchestrator"
  helmTimeout: "600s"
  serverHost: "0.0.0.0"
  serverPort: "8080"
  rustLog: "orchestrator=debug,tower_http=debug,axum=debug"

ingress:
  enabled: true
  hosts:
    - host: orchestrator.local
      paths:
        - path: /
          pathType: Prefix

tolerations:
  - key: node-role.kubernetes.io/control-plane
    operator: Exists
    effect: NoSchedule
EOF
```

#### Step 3: Deploy with Helm

```bash
# Install the Helm chart (this will take over management of existing resources)
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values orchestrator-values.yaml

# Verify deployment
kubectl get pods -n orchestrator
kubectl logs -n orchestrator deployment/orchestrator
```

#### Step 4: Cleanup Old Resources (if needed)

```bash
# Check for any orphaned resources
kubectl get all -n orchestrator

# Remove any old manually created resources that aren't managed by Helm
# (The Helm chart should adopt most existing resources)
```

### Option 2: Fresh Deployment

This approach creates a new deployment alongside the old one, then switches traffic.

#### Step 1: Deploy to New Namespace

```bash
# Create a custom values file with your configuration
cat > orchestrator-prod-values.yaml << EOF
secrets:
  anthropicApiKey: "your-anthropic-api-key"
  githubToken: "your-github-token"
ingress:
  enabled: true
  hosts:
    - host: orchestrator.yourdomain.com
EOF
# Edit the file with additional configuration as needed

# Deploy to new namespace
helm install orchestrator-new ./infra/orchestrator-chart \
  --namespace orchestrator-new \
  --create-namespace \
  --values orchestrator-prod-values.yaml
```

#### Step 2: Test New Deployment

```bash
# Test the new deployment
kubectl port-forward -n orchestrator-new svc/orchestrator 8080:80
curl http://localhost:8080/health

# Test Claude Code deployment functionality
# (Submit a test task to verify the orchestrator can deploy agents)
```

#### Step 3: Switch Traffic

```bash
# Update ingress or load balancer to point to new service
# Or scale down old deployment and rename new one

# Scale down old deployment
kubectl scale deployment orchestrator --replicas=0 -n orchestrator

# Verify new deployment is working
kubectl get pods -n orchestrator-new
```

#### Step 4: Cleanup Old Deployment

```bash
# Remove old resources
kubectl delete namespace orchestrator

# Rename new namespace (optional)
# This requires recreating resources, so consider keeping the new namespace name
```

## Deployment Verification

### Health Checks

```bash
# Check pod status
kubectl get pods -n orchestrator -l app.kubernetes.io/name=orchestrator

# Check health endpoint
kubectl port-forward -n orchestrator svc/orchestrator 8080:80
curl http://localhost:8080/health

# Check via ingress (if configured)
curl http://orchestrator.local/health
```

### Configuration Verification

```bash
# Verify ConfigMap
kubectl get configmap -n orchestrator -l app.kubernetes.io/name=orchestrator

# Verify Secret (without exposing values)
kubectl get secret -n orchestrator -l app.kubernetes.io/name=orchestrator

# Verify RBAC
kubectl auth can-i create jobs --as=system:serviceaccount:orchestrator:orchestrator -n orchestrator
kubectl auth can-i create configmaps --as=system:serviceaccount:orchestrator:orchestrator -n orchestrator
```

### Claude Code Chart Verification

```bash
# Verify the Claude Code chart is mounted
kubectl exec -n orchestrator deployment/orchestrator -- ls -la /infra/claude-code/

# Check chart files
kubectl exec -n orchestrator deployment/orchestrator -- cat /infra/claude-code/Chart.yaml
```

## Monitoring and Logging

### Logs

```bash
# View orchestrator logs
kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator -f

# View previous logs (if pod restarted)
kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator --previous
```

### Metrics

```bash
# If metrics are enabled, check metrics endpoint
kubectl port-forward -n orchestrator svc/orchestrator 8080:80
curl http://localhost:8080/metrics
```

## Troubleshooting

### Common Issues

1. **Pod not starting**
   ```bash
   kubectl describe pod -n orchestrator -l app.kubernetes.io/name=orchestrator
   kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator
   ```

2. **RBAC permissions**
   ```bash
   kubectl auth can-i create jobs --as=system:serviceaccount:orchestrator:orchestrator -n orchestrator
   ```

3. **ConfigMap/Secret issues**
   ```bash
   kubectl get configmap,secret -n orchestrator
   kubectl describe configmap orchestrator-config -n orchestrator
   ```

4. **Ingress not working**
   ```bash
   kubectl get ingress -n orchestrator
   kubectl describe ingress orchestrator -n orchestrator
   ```

### Recovery Procedures

If something goes wrong during migration:

```bash
# Rollback Helm release
helm rollback orchestrator -n orchestrator

# Or restore from backup
kubectl apply -f backup-config.yaml
kubectl apply -f backup-secrets.yaml

# Scale up original deployment if it was scaled down
kubectl scale deployment orchestrator --replicas=1 -n orchestrator
```

## Maintenance

### Updating the Orchestrator

```bash
# Update image tag
helm upgrade orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --set image.tag="v1.1.0" \
  --reuse-values

# Update configuration
helm upgrade orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values updated-values.yaml
```

### Backup and Restore

```bash
# Backup Helm values
helm get values orchestrator -n orchestrator > orchestrator-backup-values.yaml

# Backup all resources
kubectl get all,configmap,secret,ingress -n orchestrator -o yaml > orchestrator-backup.yaml

# Restore if needed
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values orchestrator-backup-values.yaml
```

## Security Considerations

1. **API Keys**: Store in Kubernetes secrets, never in values files
2. **RBAC**: Use namespaced roles when possible
3. **Network Policies**: Consider implementing network policies for pod-to-pod communication
4. **Image Security**: Use specific image tags, not `latest` in production
5. **TLS**: Enable TLS for ingress in production environments

## Performance Tuning

1. **Resources**: Adjust CPU/memory limits based on workload
2. **Replicas**: Consider running multiple replicas for high availability
3. **Storage**: Use fast storage classes for per-service workspace PVCs
4. **Node Affinity**: Pin orchestrator to specific nodes if needed
```

### orchestrator/templates/deployment.yaml (120 lines)

**Full Content:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "orchestrator.fullname" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "orchestrator.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      annotations:
        # Force pod restart when claude-templates ConfigMap changes
        claude-templates/checksum: {{ include (print $.Template.BasePath "/claude-templates-configmap.yaml") . | sha256sum }}
        # Force pod restart when controller config changes
        controller-config/checksum: {{ include (print $.Template.BasePath "/task-controller-config.yaml") . | sha256sum }}
        {{- with .Values.podAnnotations }}
        {{- toYaml . | nindent 8 }}
      {{- end }}
      labels:
        {{- include "orchestrator.selectorLabels" . | nindent 8 }}
    spec:
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ include "orchestrator.serviceAccountName" . }}
      securityContext:
        {{- toYaml .Values.podSecurityContext | nindent 8 }}
      containers:
        - name: {{ .Chart.Name }}
          securityContext:
            {{- toYaml .Values.securityContext | nindent 12 }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          command: ["/app/orchestrator"]
          ports:
            - name: {{ .Values.service.name }}
              containerPort: {{ .Values.service.targetPort }}
              protocol: TCP
          env:
            # Kubernetes configuration
            - name: KUBERNETES_NAMESPACE
              valueFrom:
                configMapKeyRef:
                  name: {{ include "orchestrator.fullname" . }}-config
                  key: KUBERNETES_NAMESPACE
            - name: RUST_LOG
              valueFrom:
                configMapKeyRef:
                  name: {{ include "orchestrator.fullname" . }}-config
                  key: RUST_LOG
            # Secrets for agents
            {{- if .Values.secrets.anthropicApiKey }}
            - name: ANTHROPIC_API_KEY
              valueFrom:
                secretKeyRef:
                  {{- if eq .Values.secrets.anthropicApiKey "use-existing" }}
                  name: orchestrator-secrets
                  {{- else }}
                  name: {{ include "orchestrator.fullname" . }}-secrets
                  {{- end }}
                  key: ANTHROPIC_API_KEY
            {{- end }}
          volumeMounts:
            # Mount claude templates ConfigMap
            - name: claude-templates
              mountPath: /claude-templates
              readOnly: true
            # Mount controller configuration ConfigMap
            - name: controller-config
              mountPath: /config
              readOnly: true
          {{- if .Values.healthCheck.enabled }}
          livenessProbe:
            httpGet:
              path: {{ .Values.healthCheck.path }}
              port: {{ .Values.service.name }}
            initialDelaySeconds: {{ .Values.healthCheck.livenessProbe.initialDelaySeconds }}
            periodSeconds: {{ .Values.healthCheck.livenessProbe.periodSeconds }}
            timeoutSeconds: {{ .Values.healthCheck.livenessProbe.timeoutSeconds }}
            successThreshold: {{ .Values.healthCheck.livenessProbe.successThreshold }}
            failureThreshold: {{ .Values.healthCheck.livenessProbe.failureThreshold }}
          readinessProbe:
            httpGet:
              path: {{ .Values.healthCheck.path }}
              port: {{ .Values.service.name }}
            initialDelaySeconds: {{ .Values.healthCheck.readinessProbe.initialDelaySeconds }}
            periodSeconds: {{ .Values.healthCheck.readinessProbe.periodSeconds }}
            timeoutSeconds: {{ .Values.healthCheck.readinessProbe.timeoutSeconds }}
            successThreshold: {{ .Values.healthCheck.readinessProbe.successThreshold }}
            failureThreshold: {{ .Values.healthCheck.readinessProbe.failureThreshold }}
          {{- end }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
      volumes:
        # Mount claude templates ConfigMap
        - name: claude-templates
          configMap:
            name: {{ include "orchestrator.fullname" . }}-claude-templates
        # Mount controller configuration ConfigMap
        - name: controller-config
          configMap:
            name: {{ include "orchestrator.fullname" . }}-task-controller-config
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
```

### orchestrator/templates/ingress.yaml (59 lines)

**Full Content:**
```yaml
{{- if .Values.ingress.enabled -}}
{{- $fullName := include "orchestrator.fullname" . -}}
{{- $svcPort := .Values.service.port -}}
{{- if and .Values.ingress.className (not (hasKey .Values.ingress.annotations "kubernetes.io/ingress.class")) }}
  {{- $_ := set .Values.ingress.annotations "kubernetes.io/ingress.class" .Values.ingress.className}}
{{- end }}
{{- if semverCompare ">=1.19-0" .Capabilities.KubeVersion.GitVersion -}}
apiVersion: networking.k8s.io/v1
{{- else if semverCompare ">=1.14-0" .Capabilities.KubeVersion.GitVersion -}}
apiVersion: networking.k8s.io/v1beta1
{{- else -}}
apiVersion: extensions/v1beta1
{{- end }}
kind: Ingress
metadata:
  name: {{ $fullName }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  {{- if and .Values.ingress.className (semverCompare ">=1.18-0" .Capabilities.KubeVersion.GitVersion) }}
  ingressClassName: {{ .Values.ingress.className }}
  {{- end }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            {{- if and .pathType (semverCompare ">=1.18-0" $.Capabilities.KubeVersion.GitVersion) }}
            pathType: {{ .pathType }}
            {{- end }}
            backend:
              {{- if semverCompare ">=1.19-0" $.Capabilities.KubeVersion.GitVersion }}
              service:
                name: {{ $fullName }}
                port:
                  number: {{ $svcPort }}
              {{- else }}
              serviceName: {{ $fullName }}
              servicePort: {{ $svcPort }}
              {{- end }}
          {{- end }}
    {{- end }}
{{- end }}
```

### orchestrator/templates/service.yaml (15 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "orchestrator.fullname" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: {{ .Values.service.name }}
      protocol: TCP
      name: {{ .Values.service.name }}
  selector:
    {{- include "orchestrator.selectorLabels" . | nindent 4 }}
```

### orchestrator/templates/task-controller-config.yaml (76 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "orchestrator.fullname" . }}-task-controller-config
  namespace: {{ .Release.Namespace }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
data:
  config.yaml: |
    # Task Controller Configuration
    # Simplified configuration for CodeRun and DocsRun controllers

    # Job configuration
    job:
      activeDeadlineSeconds: 3600  # 1 hour timeout

    # Claude agent configuration
    agent:
      image:
        repository: {{ .Values.agent.image.repository | quote }}
        tag: {{ .Values.agent.image.tag | quote }}
      imagePullSecrets:
        {{- range .Values.imagePullSecrets }}
        - {{ .name | quote }}
        {{- end }}

    # Secrets configuration (references Kubernetes secrets)
    secrets:
      apiKeySecretName: "{{ include "orchestrator.fullname" . }}-secrets"
      apiKeySecretKey: "ANTHROPIC_API_KEY"

    # Tool permissions configuration (only used when agentToolsOverride=true)
    # When false: uses hardcoded list in settings.json.hbs template
    # When true: uses this configuration
    permissions:
      agentToolsOverride: false
      allow:
        - "Bash"
        - "Edit"
        - "Read"
        - "Write"
        - "MultiEdit"
        - "Glob"
        - "Grep"
        - "LS"
        - "Task"
        - "ExitPlanMode"
        - "NotebookRead"
        - "NotebookEdit"
        - "WebFetch"
        - "WebSearch"
        - "TodoRead"
        - "TodoWrite"
      deny: []

    # Telemetry configuration (used in templates)
    telemetry:
      enabled: true
      otlpEndpoint: "otel-collector-opentelemetry-collector.telemetry.svc.cluster.local:4317"
      otlpProtocol: "grpc"
      logsEndpoint: "otel-collector-opentelemetry-collector.telemetry.svc.cluster.local:4317"
      logsProtocol: "grpc"

    # Storage configuration
    storage:
      {{- if .Values.storage.storageClassName }}
      storageClassName: {{ .Values.storage.storageClassName | quote }}
      {{- end }}
      workspaceSize: {{ .Values.storage.workspaceSize | default "10Gi" | quote }}

    # Cleanup configuration (event-driven cleanup by controller)
    cleanup:
      enabled: {{ .Values.cleanup.enabled | default true }}
      completedJobDelayMinutes: {{ .Values.cleanup.completedJobDelayMinutes | default 5 }}
      failedJobDelayMinutes: {{ .Values.cleanup.failedJobDelayMinutes | default 60 }}
      deleteConfigMap: {{ .Values.cleanup.deleteConfigMap | default true }}
```

### orchestrator/templates/rbac.yaml (51 lines)

**Full Content:**
```yaml
{{- if .Values.rbac.create -}}
{{- if .Values.rbac.namespaced }}
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: {{ include "orchestrator.roleName" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
rules:
{{- toYaml .Values.rbac.rules | nindent 2 }}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: {{ include "orchestrator.roleName" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: {{ include "orchestrator.roleName" . }}
subjects:
- kind: ServiceAccount
  name: {{ include "orchestrator.serviceAccountName" . }}
  namespace: {{ .Release.Namespace }}
{{- else }}
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: {{ include "orchestrator.roleName" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
rules:
{{- toYaml .Values.rbac.rules | nindent 2 }}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: {{ include "orchestrator.roleName" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: {{ include "orchestrator.roleName" . }}
subjects:
- kind: ServiceAccount
  name: {{ include "orchestrator.serviceAccountName" . }}
  namespace: {{ .Release.Namespace }}
{{- end }}
{{- end }}
```

### orchestrator/templates/claude-templates-configmap.yaml (12 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "orchestrator.fullname" . }}-claude-templates
  namespace: {{ .Release.Namespace }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
data:
{{- range $path, $content := .Files.Glob "claude-templates/**/*.hbs" }}
  {{ $path | trimPrefix "claude-templates/" | replace "/" "_" }}: |
{{ $.Files.Get $path | nindent 4 }}
{{- end }}
```

### orchestrator/templates/serviceaccount.yaml (12 lines)

**Full Content:**
```yaml
{{- if .Values.serviceAccount.create -}}
apiVersion: v1
kind: ServiceAccount
metadata:
  name: {{ include "orchestrator.serviceAccountName" . }}
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
  {{- with .Values.serviceAccount.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
{{- end }}
```

### orchestrator/templates/configmap.yaml (13 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "orchestrator.fullname" . }}-config
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
data:
  KUBERNETES_NAMESPACE: {{ .Values.config.kubernetesNamespace | quote }}
  SERVER_HOST: {{ .Values.config.serverHost | quote }}
  SERVER_PORT: {{ .Values.config.serverPort | quote }}
  RUST_LOG: {{ .Values.config.rustLog | quote }}
  DEFAULT_DOCS_MODEL: {{ .Values.models.defaultDocsModel | quote }}
  DEFAULT_CODE_MODEL: {{ .Values.models.defaultCodeModel | quote }}
```

### orchestrator/templates/secret.yaml (13 lines)

**Full Content:**
```yaml
{{- if and .Values.secrets.anthropicApiKey (ne .Values.secrets.anthropicApiKey "use-existing") }}
apiVersion: v1
kind: Secret
metadata:
  name: {{ include "orchestrator.fullname" . }}-secrets
  labels:
    {{- include "orchestrator.labels" . | nindent 4 }}
type: Opaque
stringData:
  {{- if .Values.secrets.anthropicApiKey }}
  ANTHROPIC_API_KEY: {{ .Values.secrets.anthropicApiKey | quote }}
  {{- end }}
{{- end }}
```

### orchestrator/values.yaml (199 lines)

**Full Content:**
```yaml
# Default values for orchestrator.
# This is a YAML-formatted file.
# Declare variables to be passed into your templates.

replicaCount: 1

image:
  repository: ghcr.io/5dlabs/platform/orchestrator
  pullPolicy: Always
  # Overrides the image tag whose default is the chart appVersion.
  tag: "latest"

# Agent/Task Runner image configuration (used by controller to create Jobs)
agent:
  image:
    repository: ghcr.io/5dlabs/platform/claude-code
    tag: "1.0.56"
    pullPolicy: Always

# Storage configuration for workspace PVCs
storage:
  # Storage class name (e.g., "local-path" for local development, leave empty for default)
  storageClassName: "local-path"
  # Size of workspace PVCs
  workspaceSize: "10Gi"

# Cleanup configuration (controller-based event-driven cleanup)
cleanup:
  # Whether to enable automatic cleanup of completed jobs
  enabled: true
  # Minutes to wait before cleaning up successful jobs (default: 5 minutes)
  completedJobDelayMinutes: 5
  # Minutes to wait before cleaning up failed jobs (default: 60 minutes)
  failedJobDelayMinutes: 60
  # Whether to delete associated ConfigMaps when cleaning up jobs
  deleteConfigMap: true

imagePullSecrets:
  - name: ghcr-secret

nameOverride: ""
fullnameOverride: ""

serviceAccount:
  # Specifies whether a service account should be created
  create: true
  # Annotations to add to the service account
  annotations: {}
  # The name of the service account to use.
  # If not set and create is true, a name is generated using the fullname template
  name: "orchestrator"

podAnnotations:
  kubectl.kubernetes.io/restartedAt: ""

podSecurityContext:
  fsGroup: 2000
  runAsNonRoot: true
  runAsUser: 1000

securityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: false
  runAsNonRoot: true
  runAsUser: 1000
  capabilities:
    drop:
    - ALL
  seccompProfile:
    type: RuntimeDefault

service:
  type: ClusterIP
  port: 80
  targetPort: 8080
  name: http

ingress:
  enabled: false
  className: "nginx"
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "false"
  hosts:
    - host: orchestrator.local
      paths:
        - path: /
          pathType: Prefix
  tls: []

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi

autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 100
  targetCPUUtilizationPercentage: 80
  # targetMemoryUtilizationPercentage: 80

nodeSelector: {}

tolerations:
  - key: node-role.kubernetes.io/control-plane
    operator: Exists
    effect: NoSchedule

affinity: {}

# Configuration for the orchestrator service
config:
  # Kubernetes namespace (auto-populated in most cases)
  kubernetesNamespace: "orchestrator"

  # Server configuration
  serverHost: "0.0.0.0"
  serverPort: "8080"

  # Logging
  rustLog: "orchestrator=debug,tower_http=debug,axum=debug,kube=info"

# Default model configurations
models:
  # Default model for documentation generation
  defaultDocsModel: "claude-opus-4-20250514"
  # Default model for code tasks
  defaultCodeModel: "claude-sonnet-4-20250514"

# Secret configuration for API keys
secrets:
  # REQUIRED: Set your Anthropic API key
  anthropicApiKey: ""
  # Note: GitHub secrets (SSH keys + tokens) are managed externally per agent
  # See infra/scripts/setup-agent-secrets.sh for setup instructions

# RBAC configuration
rbac:
  # Create RBAC resources
  create: true
  # Use Role/RoleBinding (true) or ClusterRole/ClusterRoleBinding (false)
  namespaced: true
  rules:
    # CodeRun and DocsRun CRD management
    - apiGroups: ["orchestrator.platform"]
      resources: ["coderuns", "docsruns"]
      verbs: ["create", "get", "list", "watch", "update", "patch", "delete"]
    - apiGroups: ["orchestrator.platform"]
      resources: ["coderuns/status", "docsruns/status"]
      verbs: ["get", "update", "patch"]
    # Job management in orchestrator namespace
    - apiGroups: ["batch"]
      resources: ["jobs"]
      verbs: ["create", "get", "list", "watch", "delete", "patch", "update"]
    # ConfigMap and Secret access (for agent configuration and task files)
    - apiGroups: [""]
      resources: ["configmaps", "secrets"]
      verbs: ["get", "list", "create", "update", "delete", "watch", "patch"]
    # ServiceAccount management (required for Helm operations)
    - apiGroups: [""]
      resources: ["serviceaccounts"]
      verbs: ["get", "list", "create", "update", "delete", "patch"]
    # Service management (required for Helm operations)
    - apiGroups: [""]
      resources: ["services"]
      verbs: ["get", "list", "create", "update", "delete", "patch"]
    # Pod monitoring
    - apiGroups: [""]
      resources: ["pods", "pods/log"]
      verbs: ["get", "list", "watch"]
    # PVC management for agent workspaces
    - apiGroups: [""]
      resources: ["persistentvolumeclaims"]
      verbs: ["create", "get", "list", "delete"]
    # Events for debugging
    - apiGroups: [""]
      resources: ["events"]
      verbs: ["get", "list", "watch"]

# Health checks
healthCheck:
  enabled: true
  path: "/health"
  port: 8080
  livenessProbe:
    initialDelaySeconds: 30
    periodSeconds: 60
    timeoutSeconds: 1
    successThreshold: 1
    failureThreshold: 3
  readinessProbe:
    initialDelaySeconds: 10
    periodSeconds: 30
    timeoutSeconds: 1
    successThreshold: 1
    failureThreshold: 3

```

