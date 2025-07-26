# Autonomous Agent Prompt: Analyze and Optimize Toolman Deployment

## Context
You are tasked with analyzing the existing Toolman Kubernetes deployment manifest located at `toolman/charts/toolman/templates/deployment.yaml`. This deployment is part of the Helm chart that hasn't been deployed yet. Your goal is to ensure the deployment configuration is production-ready and optimized for our orchestrator environment.

## Current Situation
- **Deployment Status**: Exists as a template but NOT deployed
- **Location**: `toolman/charts/toolman/templates/deployment.yaml`
- **Features**: Complete configuration including containers, volumes, security contexts, probes
- **Dependency**: Will be deployed as part of Task 1 (Helm chart deployment)

## Your Mission
Thoroughly analyze the deployment manifest, validate its production readiness, and document any necessary value overrides to ensure optimal performance and security when deployed.

## Detailed Instructions

### 1. Initial Deployment Analysis
```bash
# Navigate to the deployment template
cd toolman/charts/toolman/templates/

# Read the entire deployment manifest
cat deployment.yaml

# Analyze template variables
grep -n "{{" deployment.yaml | head -20

# Check default values that affect deployment
cd ../
grep -A 5 -B 5 "replicaCount\|resources\|image\|security" values.yaml
```

### 2. Container Configuration Review
Examine these critical areas:

**Main Container Analysis:**
```bash
# Extract container configuration
yq eval '.spec.template.spec.containers' deployment.yaml

# Review:
# - Image and tag configuration
# - Port specifications
# - Environment variables
# - Volume mounts
# - Security context
# - Resource limits
```

**Init Container Review:**
```bash
# Check init containers
yq eval '.spec.template.spec.initContainers' deployment.yaml

# Understand:
# - Purpose of each init container
# - Permission setup requirements
# - Dependencies created
```

**Sidecar Containers:**
```bash
# Look for Docker-in-Docker or other sidecars
grep -A 20 "dind\|sidecar" deployment.yaml

# Assess:
# - When sidecars are enabled
# - Security implications
# - Resource requirements
```

### 3. Volume Configuration Assessment
```yaml
# Identify all volume types:
# 1. ConfigMap volumes (MCP server definitions)
# 2. PersistentVolumeClaim (data storage)
# 3. EmptyDir (temporary storage)
# 4. HostPath/Docker socket (if DinD enabled)

# For each volume, verify:
# - Mount paths are correct
# - Permissions are appropriate
# - No sensitive data exposure
```

### 4. Security Context Evaluation
```bash
# Extract security contexts
yq eval '.spec.template.spec.securityContext' deployment.yaml
yq eval '.spec.template.spec.containers[0].securityContext' deployment.yaml

# Verify:
# - Non-root user (UID 1001)
# - Appropriate group (GID 2375)
# - Dropped capabilities
# - Read-only root filesystem where possible
# - No unnecessary privileges
```

### 5. Production Readiness Assessment
Create a comprehensive analysis:

**High Availability:**
- Current replica count configuration
- Update strategy settings
- Pod disruption budget support
- Anti-affinity rule recommendations

**Resource Management:**
- Default resource requests/limits
- Recommendations for production
- Horizontal Pod Autoscaling readiness

**Health Checks:**
- Liveness probe configuration
- Readiness probe configuration
- Probe timing appropriateness
- Failure threshold settings

### 6. Create Production Values Override
Based on your analysis, create `toolman-production-values.yaml`:

```yaml
# Production overrides for toolman deployment
replicaCount: 3  # HA requirement

image:
  repository: ghcr.io/5dlabs/toolman
  tag: "v1.0.0"  # Specific version, not latest
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

# Health check tuning
livenessProbe:
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3

# HA configuration
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

# PDB for production
podDisruptionBudget:
  enabled: true
  minAvailable: 2
```

### 7. Template Rendering Validation
```bash
# Test template rendering with production values
helm template toolman ./toolman/charts/toolman/ \
  --namespace orchestrator \
  -f toolman-production-values.yaml > rendered.yaml

# Extract and review the deployment
yq eval 'select(.kind == "Deployment")' rendered.yaml > deployment-rendered.yaml

# Validate the rendered deployment
kubectl --dry-run=client apply -f deployment-rendered.yaml
```

### 8. Security Scanning
```bash
# If available, run security scanners
# Example with kubesec
kubesec scan deployment-rendered.yaml

# Check for:
# - Privileged containers
# - Host network/PID/IPC usage
# - Unsafe capabilities
# - Writable root filesystem
```

### 9. Documentation Creation
Create a comprehensive analysis document including:

1. **Current Configuration Summary**
   - Container specifications
   - Volume mounts
   - Security settings
   - Resource allocations

2. **Production Recommendations**
   - Required value overrides
   - Security enhancements
   - Performance optimizations
   - HA configurations

3. **Risk Assessment**
   - Identified security concerns
   - Resource constraint risks
   - Availability considerations

4. **Deployment Checklist**
   - Pre-deployment validations
   - Required secrets/configs
   - Monitoring setup
   - Rollback procedures

## Testing Requirements

### 1. Template Validation
```bash
# Lint with production values
helm lint ./toolman/charts/toolman/ -f toolman-production-values.yaml

# Dry run deployment
helm install toolman ./toolman/charts/toolman/ \
  --namespace orchestrator \
  --dry-run --debug \
  -f toolman-production-values.yaml
```

### 2. Configuration Validation
- Verify all template variables resolve correctly
- Ensure no hardcoded values that should be configurable
- Validate label and annotation consistency
- Check resource naming conventions

### 3. Security Validation
- Confirm non-root execution
- Verify minimal required privileges
- Validate network policies compatibility
- Check secret handling

## Deliverables

1. **Deployment Analysis Report**
   - Complete review of existing deployment.yaml
   - Security findings and recommendations
   - Performance optimization suggestions
   - HA readiness assessment

2. **Production Values File**
   - toolman-production-values.yaml
   - Documented rationale for each override
   - Environment-specific configurations

3. **Validation Results**
   - Template rendering output
   - Security scan results
   - Dry-run test outcomes

4. **Operational Guide**
   - How deployment integrates with Helm chart
   - Monitoring recommendations
   - Troubleshooting common issues
   - Update procedures

## Success Metrics
- Zero security violations identified
- All production requirements addressed
- Template renders without errors
- Resource allocations optimized
- HA configurations validated
- Documentation comprehensive

## Important Notes
- The deployment is part of the Helm chart - don't modify it directly
- Focus on value overrides rather than template changes
- Consider this analysis will inform Task 1 deployment
- Document any concerns for the platform team

Proceed with the analysis and provide detailed findings for each section.