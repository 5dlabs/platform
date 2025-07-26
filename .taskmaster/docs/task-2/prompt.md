# Autonomous Agent Prompt: Configure Toolman Kubernetes Deployment

## Context
You are tasked with reviewing and customizing the Toolman Kubernetes deployment manifest located at `toolman/charts/toolman/templates/deployment.yaml`. This deployment is critical for running the Toolman HTTP proxy that enables Claude agents to access MCP tools. You must ensure it's production-ready for our orchestrator environment.

## Your Mission
Analyze the existing deployment manifest, validate all configurations, and prepare it for production use with appropriate customizations for resource management, security, and high availability.

## Detailed Instructions

### 1. Initial Deployment Review
```bash
# Navigate to the deployment template
cd toolman/charts/toolman/templates/

# Read the deployment manifest
cat deployment.yaml

# Analyze the structure and note:
# - Container definitions
# - Volume configurations  
# - Security contexts
# - Environment variables
# - Health check configurations
```

### 2. Container Configuration Analysis
Focus on these critical areas:

**Main Container:**
- Image specification and tag strategy
- Port configuration (should be 3000)
- Command and args (if any)
- Working directory settings

**Init Container:**
- Purpose and necessity
- Permission setup commands
- Volume initialization

**Sidecar Containers:**
- Docker-in-Docker sidecar purpose
- When it's needed (Docker-based MCP servers)
- Security implications

### 3. Volume Mount Verification
Ensure proper volume configuration:
```yaml
# Critical volumes to verify:
# 1. ConfigMap volume for MCP server definitions
- name: config
  configMap:
    name: toolman-servers-config
    
# 2. Persistent storage (if enabled)
- name: data
  persistentVolumeClaim:
    claimName: {{ include "toolman.fullname" . }}
    
# 3. Temp directory
- name: tmp
  emptyDir: {}
  
# 4. Docker socket (if DinD enabled)
- name: docker-socket
  emptyDir: {}
```

### 4. Environment Variable Configuration
Validate and document all environment variables:
- `PORT`: Should be set to "3000"
- `PROJECT_DIR`: Data directory path
- `RUST_LOG`: Logging level configuration
- Any secret references
- Additional platform-specific variables

### 5. Security Context Review
```yaml
# Verify security settings:
securityContext:
  runAsUser: 1001      # Non-root user
  runAsGroup: 2375     # Specific group
  fsGroup: 2375        # File system group
  runAsNonRoot: true   # Enforce non-root
  capabilities:
    drop:
    - ALL              # Drop all capabilities
```

### 6. Resource Requirements Testing
```bash
# Create test deployment with different resource configs
# Test 1: Minimal resources
resources:
  requests:
    cpu: 100m
    memory: 256Mi
  limits:
    cpu: 500m
    memory: 512Mi

# Test 2: Production resources
resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

# Monitor performance under load
# Document optimal settings
```

### 7. High Availability Configuration
Implement production-ready HA settings:

```yaml
# 1. Replica count (minimum 2-3)
replicas: {{ .Values.replicaCount | default 3 }}

# 2. Pod disruption budget
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: toolman-pdb
spec:
  minAvailable: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: toolman

# 3. Anti-affinity rules
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        topologyKey: kubernetes.io/hostname
```

### 8. Health Check Optimization
```yaml
# Validate and tune health checks:
livenessProbe:
  httpGet:
    path: /health
    port: http
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
  
readinessProbe:
  httpGet:
    path: /ready
    port: http
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  successThreshold: 1
  failureThreshold: 3
```

### 9. ConfigMap Integration Testing
```bash
# Deploy test instance
helm install toolman-test ./toolman/charts/toolman/ -n test

# Verify ConfigMap mounting
kubectl exec -n test deployment/toolman-test -- ls -la /config/
kubectl exec -n test deployment/toolman-test -- cat /config/servers-config.json

# Test MCP server accessibility
curl http://toolman-test:3000/mcp/servers
```

### 10. Production Customization Document
Create a comprehensive customization guide including:

**values-production.yaml:**
```yaml
# Toolman Production Values
namespace: orchestrator
replicaCount: 3

image:
  repository: ghcr.io/5dlabs/toolman
  tag: "v1.0.0"  # Use specific version
  pullPolicy: IfNotPresent

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

service:
  type: ClusterIP
  port: 3000

persistence:
  enabled: true
  size: 20Gi
  storageClass: "fast-ssd"

# Monitoring
metrics:
  enabled: true
  path: /metrics
  port: 9090

# Logging
logLevel: "info"

# Node affinity for dedicated nodes
nodeSelector:
  node-role: "tools"
```

## Testing Requirements

### 1. Functional Tests
- [ ] Deployment creates all pods successfully
- [ ] Pods reach ready state within 60 seconds
- [ ] ConfigMap is properly mounted
- [ ] Environment variables are set correctly
- [ ] Service endpoints are populated

### 2. Load Tests
- [ ] Deploy with production resources
- [ ] Simulate concurrent MCP requests
- [ ] Monitor CPU and memory usage
- [ ] Verify no OOM kills or restarts
- [ ] Check response times remain acceptable

### 3. Resilience Tests
- [ ] Kill a pod and verify recovery
- [ ] Update ConfigMap and verify reload
- [ ] Simulate node failure
- [ ] Test rolling update process

### 4. Security Tests
- [ ] Verify non-root execution
- [ ] Check no privileged access (except DinD if needed)
- [ ] Validate network policies
- [ ] Ensure secrets are not exposed

## Deliverables

1. **Deployment Analysis Report**
   - Current configuration assessment
   - Security findings
   - Performance considerations
   - HA readiness evaluation

2. **Customization Guide**
   - Production values.yaml
   - Environment-specific overrides
   - Scaling recommendations
   - Monitoring integration

3. **Test Results**
   - Functional test outcomes
   - Load test metrics
   - Resilience test results
   - Security scan report

4. **Operational Runbook**
   - Deployment procedures
   - Upgrade processes
   - Troubleshooting guide
   - Monitoring setup

## Success Metrics
- Zero deployment failures
- Pod startup time < 30 seconds
- Zero security violations
- Resource usage within limits
- 99.9% uptime target
- Successful ConfigMap integration
- All health checks passing

Proceed with the deployment review and provide detailed findings for each section. Focus on production readiness while maintaining operational simplicity.