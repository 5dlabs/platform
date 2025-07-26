# Acceptance Criteria: Task 1 - Deploy Toolman Helm Chart

## Overview
This document defines the acceptance criteria for successfully deploying the existing Toolman Helm chart to the orchestrator namespace in our Kubernetes cluster.

## Current State
- **Chart Location**: `toolman/charts/toolman/` (fully developed and ready)
- **Deployment Status**: NOT deployed
- **Target Namespace**: orchestrator (exists, contains only orchestrator deployment)
- **Image**: ghcr.io/5dlabs/toolman

## Deployment Requirements

### 1. Pre-Deployment Validation
- [ ] **Image Accessibility**: Verify ghcr.io/5dlabs/toolman image can be pulled
- [ ] **Authentication Setup**: ghcr-secret configured if required
- [ ] **Namespace Ready**: orchestrator namespace exists and is accessible
- [ ] **Helm Lint Success**: Chart passes `helm lint` without errors
- [ ] **Dry Run Success**: `helm install --dry-run` completes without errors

### 2. Successful Deployment
- [ ] **Helm Install**: Chart deploys without errors using `helm install`
- [ ] **All Pods Running**: Deployment creates pods that reach Running state
- [ ] **No Restarts**: Pods remain stable without restart loops
- [ ] **Resources Created**: All expected Kubernetes resources are created
- [ ] **PVC Bound**: If persistence enabled, PVC is successfully bound

### 3. Service Availability
- [ ] **Service Created**: toolman service exists in orchestrator namespace
- [ ] **Endpoints Populated**: Service has active endpoints
- [ ] **Internal Access**: Service accessible at toolman.orchestrator.svc.cluster.local:3000
- [ ] **Port 3000 Open**: Service correctly exposes port 3000

### 4. Health Verification
- [ ] **Health Endpoint**: GET /health returns successful response
- [ ] **Ready Endpoint**: GET /ready returns successful response
- [ ] **No Error Logs**: Pod logs show no critical errors
- [ ] **MCP Servers Loaded**: Logs confirm MCP server initialization

## Technical Validation

### 1. Resource Verification
```bash
# Expected resources after deployment:
kubectl get all -n orchestrator -l app.kubernetes.io/name=toolman

# Should show:
- deployment.apps/toolman (READY)
- service/toolman (ClusterIP on port 3000)
- pods with status Running
- replicaset managed by deployment
```

### 2. ConfigMap Validation
```bash
# Verify ConfigMap exists
kubectl get configmap -n orchestrator | grep toolman

# Expected ConfigMaps:
- toolman-config or toolman-servers-config

# Verify MCP servers configured
kubectl get configmap toolman-config -n orchestrator -o yaml

# Should contain all 7 pre-configured servers:
- brave-search
- memory
- terraform
- kubernetes
- solana
- rustdocs
- reddit
```

### 3. Network Connectivity Tests
```bash
# Internal cluster test
kubectl run test-pod --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman:3000/health

# Expected: HTTP 200 OK

# Full FQDN test
kubectl run test-pod --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman.orchestrator.svc.cluster.local:3000/ready

# Expected: HTTP 200 OK
```

### 4. MCP Server Endpoint Test
```bash
# List available MCP servers
kubectl run test-pod --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman:3000/mcp/servers

# Expected: JSON response listing available MCP servers
```

## Test Scenarios

### Scenario 1: Basic Deployment
```bash
# Deploy with minimal configuration
helm install toolman ./toolman/charts/toolman/ -n orchestrator

# Verify deployment
kubectl rollout status deployment/toolman -n orchestrator

# Expected: deployment "toolman" successfully rolled out
```

### Scenario 2: Deployment with Image Pull Secret
```bash
# Deploy with ghcr-secret
helm install toolman ./toolman/charts/toolman/ \
  -n orchestrator \
  --set imagePullSecrets[0].name=ghcr-secret

# Verify secret is used
kubectl get deployment toolman -n orchestrator -o yaml | grep imagePullSecrets -A 2
```

### Scenario 3: Service Access Verification
```bash
# Port forward for local testing
kubectl port-forward -n orchestrator svc/toolman 3000:3000

# Test endpoints locally
curl http://localhost:3000/health
curl http://localhost:3000/ready
curl http://localhost:3000/mcp/servers
```

### Scenario 4: Pod Stability Check
```bash
# Monitor pod stability for 5 minutes
kubectl get pods -n orchestrator -l app.kubernetes.io/name=toolman -w

# Expected: No restarts, status remains Running
```

## Documentation Deliverables

### 1. Deployment Record
- [ ] **Exact Command**: Document the helm install command used
- [ ] **Values Used**: Any custom values or overrides applied
- [ ] **Secrets Created**: List of any secrets configured
- [ ] **Timestamp**: When deployment was completed

### 2. Verification Results
- [ ] **Resource List**: Output of kubectl get all for toolman resources
- [ ] **Health Check Results**: Successful health/ready endpoint responses
- [ ] **MCP Server List**: Available servers from /mcp/servers endpoint
- [ ] **Log Excerpts**: Key log entries showing successful startup

### 3. Access Information
- [ ] **Service URL**: Internal cluster URL for toolman service
- [ ] **Port Information**: Confirmed port 3000 accessibility
- [ ] **DNS Names**: All valid DNS names for accessing the service
- [ ] **Integration Points**: How other services can connect to toolman

## Definition of Done

✅ **Deployment Complete**
- Helm chart successfully installed to orchestrator namespace
- All pods running and healthy
- No errors in deployment process

✅ **Service Operational**
- Service endpoints active and responding
- Health checks passing
- MCP server endpoint functional

✅ **Verification Passed**
- All test scenarios completed successfully
- No pod restarts or errors in logs
- Service accessible from within cluster

✅ **Documentation Complete**
- Deployment commands recorded
- Verification results documented
- Access information provided

## Sign-off Criteria

- [ ] **Deployment Success**: Chart deployed without manual intervention
- [ ] **Service Health**: All health endpoints responding correctly
- [ ] **Stability Confirmed**: Pods running stable for at least 30 minutes
- [ ] **Access Verified**: Service accessible from test pods
- [ ] **MCP Servers Ready**: /mcp/servers endpoint returns expected servers

## Notes
- Image authentication may be required for ghcr.io
- Focus is on deployment, not chart modification
- Document any issues encountered for future reference
- This deployment enables subsequent MCP tool integration tasks