# Acceptance Criteria: Task 2 - Implement Toolman Kubernetes Deployment

## Overview
This document defines the acceptance criteria for successfully implementing and customizing the Toolman Kubernetes deployment for production use in our orchestrator environment.

## Core Requirements

### 1. Deployment Configuration
- [ ] **Manifest Review Complete**: All aspects of deployment.yaml analyzed
- [ ] **Container Specs Validated**: Image, ports, commands configured correctly
- [ ] **Volume Mounts Verified**: All required volumes properly mounted
- [ ] **Environment Variables Set**: All required env vars configured
- [ ] **Security Context Applied**: Non-root user, proper permissions
- [ ] **Resource Limits Defined**: CPU and memory requests/limits set

### 2. High Availability Setup
- [ ] **Replica Count**: Minimum 2 replicas configured
- [ ] **Anti-Affinity Rules**: Pods distributed across nodes
- [ ] **Pod Disruption Budget**: PDB configured for maintenance
- [ ] **Rolling Updates**: Update strategy properly configured
- [ ] **Graceful Shutdown**: PreStop hooks implemented

### 3. Integration Points
- [ ] **ConfigMap Integration**: toolman-servers-config properly mounted
- [ ] **Service Matching**: Labels match service selectors
- [ ] **Namespace Alignment**: Deploys to orchestrator namespace
- [ ] **DNS Resolution**: Pod DNS names resolvable

## Technical Specifications

### 1. Container Configuration
```yaml
# Required container settings:
containers:
- name: toolman
  image: ghcr.io/5dlabs/toolman:v1.0.0  # Specific version, not latest
  ports:
  - containerPort: 3000
    name: http
    protocol: TCP
  env:
  - name: PORT
    value: "3000"
  - name: PROJECT_DIR
    value: "/data/projects"
  - name: RUST_LOG
    value: "info"
```

### 2. Volume Configuration
```yaml
# Required volumes:
volumes:
- name: config
  configMap:
    name: toolman-servers-config
    items:
    - key: servers-config.json
      path: servers-config.json
- name: data
  persistentVolumeClaim:
    claimName: toolman-data
- name: tmp
  emptyDir: {}
```

### 3. Security Configuration
```yaml
# Required security context:
securityContext:
  runAsUser: 1001
  runAsGroup: 2375
  fsGroup: 2375
  runAsNonRoot: true
  capabilities:
    drop:
    - ALL
```

### 4. Resource Configuration
```yaml
# Minimum production resources:
resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi
```

### 5. Health Checks
```yaml
# Required probes:
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
  failureThreshold: 3
```

## Test Cases

### Test Case 1: Basic Deployment
```bash
# Deploy toolman
helm install toolman ./toolman/charts/toolman/ \
  -n orchestrator \
  -f production-values.yaml

# Verify deployment
kubectl get deployment toolman -n orchestrator

# Expected:
# - Deployment shows READY 3/3
# - No restart counts
# - All pods running
```

### Test Case 2: ConfigMap Mount Verification
```bash
# Check ConfigMap mounting
kubectl exec -n orchestrator deployment/toolman -- \
  cat /config/servers-config.json | jq .

# Expected:
# - Valid JSON output
# - Contains MCP server definitions
# - File permissions allow reading
```

### Test Case 3: Resource Usage Validation
```bash
# Monitor resource usage
kubectl top pod -n orchestrator -l app.kubernetes.io/name=toolman

# Expected:
# - CPU usage < 80% of limit
# - Memory usage < 80% of limit
# - No OOMKilled events
```

### Test Case 4: High Availability Testing
```bash
# Delete a pod
kubectl delete pod -n orchestrator -l app.kubernetes.io/name=toolman | head -1

# Watch recovery
kubectl get pods -n orchestrator -l app.kubernetes.io/name=toolman -w

# Expected:
# - New pod created within 30 seconds
# - Service remains available
# - No downtime for other pods
```

### Test Case 5: Health Check Validation
```bash
# Test liveness endpoint
kubectl exec -n orchestrator deployment/toolman -- \
  curl -s localhost:3000/health

# Test readiness endpoint  
kubectl exec -n orchestrator deployment/toolman -- \
  curl -s localhost:3000/ready

# Expected:
# - HTTP 200 responses
# - Fast response times (<1s)
```

### Test Case 6: Docker Sidecar (if enabled)
```bash
# If Docker-in-Docker is enabled
kubectl exec -n orchestrator deployment/toolman -c dind -- \
  docker version

# Expected:
# - Docker daemon running
# - Can execute docker commands
```

## Performance Criteria

### 1. Startup Performance
- [ ] **Pod Startup**: < 30 seconds to ready state
- [ ] **Init Container**: Completes in < 10 seconds
- [ ] **Health Checks**: Pass within 2 attempts

### 2. Runtime Performance
- [ ] **CPU Usage**: Average < 50% of request
- [ ] **Memory Usage**: Stable, no leaks
- [ ] **Response Time**: Health checks < 100ms

### 3. Reliability
- [ ] **Pod Stability**: No restarts in 24 hours
- [ ] **Recovery Time**: < 60 seconds after failure
- [ ] **Update Time**: Rolling update < 5 minutes

## Security Checklist

- [ ] **Non-Root User**: Containers run as UID 1001
- [ ] **No Privileged Mode**: Except DinD if required
- [ ] **Capabilities Dropped**: All capabilities removed
- [ ] **Read-Only Root**: Where applicable
- [ ] **Network Policies**: Compatible with deployment
- [ ] **Secret Management**: No hardcoded secrets

## Documentation Requirements

### 1. Deployment Guide
- [ ] **Prerequisites**: Listed and verified
- [ ] **Step-by-Step**: Clear deployment instructions
- [ ] **Customization**: How to modify for environments
- [ ] **Troubleshooting**: Common issues and solutions

### 2. Configuration Reference
- [ ] **Environment Variables**: All vars documented
- [ ] **Volume Mounts**: Purpose of each volume
- [ ] **Resource Sizing**: Recommendations provided
- [ ] **Security Context**: Rationale explained

### 3. Operational Procedures
- [ ] **Health Monitoring**: How to check health
- [ ] **Log Analysis**: Where to find logs
- [ ] **Scaling Guide**: When and how to scale
- [ ] **Update Process**: Safe update procedures

## Definition of Done

✅ **Deployment Validated**
- Manifest thoroughly reviewed and understood
- All features tested and working
- Security requirements met
- Resource limits appropriate

✅ **Integration Verified**
- ConfigMap properly mounted
- Service discovery working
- Network connectivity confirmed
- DNS resolution functional

✅ **High Availability Confirmed**
- Multiple replicas running
- Anti-affinity rules active
- Pod disruptions handled gracefully
- Zero-downtime updates possible

✅ **Testing Complete**
- All test cases passing
- Load testing performed
- Failure scenarios tested
- Security scans clean

✅ **Documentation Delivered**
- Deployment guide complete
- Configuration reference ready
- Operational runbook created
- Customization examples provided

## Sign-off Requirements

- [ ] **Technical Review**: Deployment configuration approved
- [ ] **Security Review**: Security settings validated
- [ ] **Performance Review**: Resource usage acceptable
- [ ] **Integration Test**: End-to-end flow working
- [ ] **Documentation Review**: All guides complete and accurate

## Notes
- This deployment is critical for the entire MCP tool integration
- Ensure backward compatibility with existing configs
- Consider future scaling requirements
- Document any deviations from standard patterns