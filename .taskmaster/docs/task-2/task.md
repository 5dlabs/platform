# Task 2: Implement Toolman Kubernetes Deployment

## Overview
Verify and optimize the Kubernetes deployment manifest for Toolman that exists at `toolman/charts/toolman/templates/deployment.yaml`. Since the Helm chart hasn't been deployed yet (Task 1), this task focuses on understanding the deployment configuration and ensuring it will meet our production requirements when deployed.

## Current State
The deployment manifest **already exists** at `toolman/charts/toolman/templates/deployment.yaml` with:
- Complete container configuration for ghcr.io/5dlabs/toolman
- Volume mounts for config, tmp, docker socket, and persistent storage
- Security context with non-root user (UID 1001, GID 2375)
- Environment variables (PORT, PROJECT_DIR, RUST_LOG)
- Docker-in-Docker sidecar container for MCP servers requiring Docker
- Init container for setting up directories with proper permissions
- Support for liveness and readiness probes

**Status**: Deployment manifest exists but has **NOT been applied** to the cluster as the Helm chart hasn't been deployed yet.

## Context
This task ensures the deployment manifest within the Helm chart is production-ready before we deploy it in Task 1. We need to verify that all deployment configurations align with our orchestrator requirements and security standards.

## Objectives
1. Thoroughly analyze the existing deployment.yaml template
2. Verify production-readiness of all configurations
3. Identify any required value overrides for production
4. Document optimal resource allocations
5. Ensure high availability configurations are appropriate
6. Validate security contexts meet our standards

## Technical Deep Dive

### Current Deployment Features
The existing deployment includes sophisticated configurations:

1. **Container Configuration**
   - Main toolman container with ghcr.io/5dlabs/toolman image
   - Optional Docker-in-Docker (DinD) sidecar for Docker-based MCP servers
   - Init container for directory setup and permissions

2. **Volume Management**
   - Config volume for MCP server definitions
   - Temp directory for runtime operations
   - Docker socket mount (when DinD enabled)
   - Persistent storage for stateful data

3. **Security Configuration**
   - Non-root user execution (UID 1001, GID 2375)
   - Security contexts properly configured
   - Read-only root filesystem where applicable

4. **Environment Configuration**
   - Direct environment variables (PORT, PROJECT_DIR, RUST_LOG)
   - SecretRef for sensitive configurations
   - Proper configuration injection patterns

5. **Health Management**
   - Liveness probe configuration
   - Readiness probe configuration
   - Proper startup delays and intervals

### Key Integration Points

1. **ConfigMap Integration**
   ```yaml
   volumes:
   - name: config
     configMap:
       name: toolman-servers-config  # Critical: contains MCP server definitions
   ```

2. **Service Discovery**
   - Deployment labels must match Service selector
   - Proper pod naming for DNS resolution
   - Network policy compatibility

3. **Resource Management**
   - CPU/Memory requests for guaranteed QoS
   - Limits to prevent resource exhaustion
   - Horizontal Pod Autoscaling readiness

## Implementation Guide

### Step 1: Deployment Template Analysis
```bash
# Navigate to the deployment template
cd toolman/charts/toolman/templates/

# Review the full deployment manifest
cat deployment.yaml

# Check how values are used in the template
grep -n "\.Values\." deployment.yaml

# Understand the template structure
head -n 50 deployment.yaml
```

### Step 2: Validate Against Production Requirements
Review these critical sections:

1. **Container Specifications**
   - Image pull policy and secrets
   - Resource requests and limits
   - Security contexts
   - Environment variables

2. **Volume Configuration**
   - ConfigMap mounting for MCP servers
   - Persistent storage if enabled
   - Temp directory setup
   - Docker socket for DinD

3. **High Availability**
   - Replica count from values
   - Update strategy
   - Pod disruption budget support
   - Anti-affinity rules

4. **Health Checks**
   - Liveness probe configuration
   - Readiness probe configuration
   - Startup probe if needed

### Step 3: Create Production Values
Based on the deployment analysis, create production-ready values:

```yaml
# production-values.yaml
replicaCount: 3

image:
  repository: ghcr.io/5dlabs/toolman
  tag: "v1.0.0"  # Use specific version
  pullPolicy: IfNotPresent

imagePullSecrets:
  - name: ghcr-secret

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

# Add pod disruption budget
podDisruptionBudget:
  enabled: true
  minAvailable: 2

# Enable anti-affinity
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchExpressions:
          - key: app.kubernetes.io/name
            operator: In
            values:
            - toolman
        topologyKey: kubernetes.io/hostname
```

### Step 4: Test Template Rendering
```bash
# Test how the deployment will render with production values
helm template toolman ./toolman/charts/toolman/ \
  -f production-values.yaml \
  --namespace orchestrator > rendered-deployment.yaml

# Extract just the deployment
yq eval 'select(.kind == "Deployment")' rendered-deployment.yaml > deployment-only.yaml

# Review the rendered deployment
cat deployment-only.yaml
```

### Step 5: Security Validation
Ensure the deployment meets security requirements:
- Non-root user execution
- Dropped capabilities
- Read-only root filesystem where possible
- No privileged containers (except DinD if required)

## Testing Strategy

Since the deployment hasn't been applied yet, testing focuses on validation:

### 1. Template Validation
```bash
# Ensure deployment template is valid
helm lint ./toolman/charts/toolman/

# Check with production values
helm lint ./toolman/charts/toolman/ -f production-values.yaml
```

### 2. Dry Run Testing
```bash
# Test deployment with production values
helm install toolman ./toolman/charts/toolman/ \
  --namespace orchestrator \
  --dry-run --debug \
  -f production-values.yaml
```

### 3. Resource Validation
Review the rendered deployment for:
- Appropriate resource requests/limits
- Correct volume mounts
- Proper environment variables
- Security context settings

## Success Criteria
1. ✅ Deployment template thoroughly analyzed
2. ✅ Production values documented
3. ✅ Security requirements validated
4. ✅ Resource allocations optimized
5. ✅ High availability features configured
6. ✅ Template renders without errors

## Dependencies
- Task 1: Helm chart must be deployed for this deployment to be applied
- Deployment manifest is part of the Helm chart structure

## Related Tasks
- Task 1: Deploy the Helm chart containing this deployment
- Task 3: Service and ConfigMap templates that work with this deployment
- Task 13: Final testing of the deployed service