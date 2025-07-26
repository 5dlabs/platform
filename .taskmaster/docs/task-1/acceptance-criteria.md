# Acceptance Criteria: Task 1 - Create Toolman Helm Chart Structure

## Overview
This document defines the acceptance criteria for successfully completing the review and customization of the Toolman Helm chart for our orchestrator environment.

## Functional Requirements

### 1. Chart Validation
- [ ] **Helm Lint Success**: Chart must pass `helm lint` without any errors
- [ ] **Dry Run Success**: `helm install --dry-run` completes without errors
- [ ] **Template Generation**: All Kubernetes resources generate correctly
- [ ] **API Version Compatibility**: All resources use Kubernetes APIs compatible with our cluster version

### 2. Configuration Review
- [ ] **Complete Values Documentation**: All configurable values in values.yaml are documented
- [ ] **Default Values Verified**: Default values are appropriate for our use case
- [ ] **Required Overrides Identified**: Platform-specific overrides are clearly identified
- [ ] **MCP Server Configuration**: Pre-configured MCP servers are reviewed and understood

### 3. Template Analysis
- [ ] **Deployment Template**: Container specs, volumes, and security contexts reviewed
- [ ] **Service Template**: Port 3000 exposure confirmed
- [ ] **ConfigMap Template**: MCP server definition structure documented
- [ ] **PVC Template**: Persistence requirements understood
- [ ] **Security Contexts**: Non-root user execution verified

### 4. Custom Configuration
- [ ] **Custom Values File**: Created with orchestrator-specific settings
- [ ] **Namespace Configuration**: Set to 'orchestrator'
- [ ] **Resource Limits**: Appropriate CPU and memory limits defined
- [ ] **Replica Count**: Set to 2 or more for high availability
- [ ] **Image Configuration**: Correct repository and tag specified

## Technical Requirements

### 1. Kubernetes Resources
```yaml
# Expected resources after deployment:
- Deployment: toolman (2+ replicas)
- Service: toolman-service (ClusterIP, port 3000)
- ConfigMap: toolman-config (MCP server definitions)
- PVC: toolman-storage (if persistence enabled)
```

### 2. Network Configuration
- [ ] **Service Type**: ClusterIP for internal access
- [ ] **Port Exposure**: Port 3000 accessible within cluster
- [ ] **Service Name**: Accessible as 'toolman-service'
- [ ] **DNS Resolution**: Full FQDN works: toolman-service.orchestrator.svc.cluster.local

### 3. Security Configuration
- [ ] **Non-Root User**: Container runs as non-root (UID 1001)
- [ ] **Read-Only Root**: Root filesystem is read-only where possible
- [ ] **Security Context**: Proper security contexts applied
- [ ] **RBAC**: Required RBAC resources identified (if any)

## Test Scenarios

### Scenario 1: Basic Deployment Test
```bash
# Test deployment to isolated namespace
kubectl create namespace toolman-test
helm install toolman ./toolman/charts/toolman/ -n toolman-test

# Verify all resources created
kubectl get all -n toolman-test

# Expected output:
# - Deployment running with 2/2 replicas
# - Service created and endpoints populated
# - ConfigMap present with server definitions
```

### Scenario 2: Custom Values Test
```bash
# Deploy with custom values
helm install toolman ./toolman/charts/toolman/ \
  -n toolman-test \
  -f custom-values.yaml

# Verify customizations applied
kubectl get deployment toolman -n toolman-test -o yaml | grep -E "(replicas|image:|cpu:|memory:)"
```

### Scenario 3: Service Connectivity Test
```bash
# Test internal service connectivity
kubectl run test-pod --rm -it --image=curlimages/curl -n toolman-test -- \
  curl -v http://toolman-service:3000/health

# Expected: HTTP 200 response
```

### Scenario 4: MCP Server Accessibility Test
```bash
# Verify MCP servers are configured
kubectl get configmap toolman-config -n toolman-test -o yaml

# Expected: servers-config.json contains all pre-configured servers
# - brave-search
# - kubernetes
# - memory
# - terraform
# - etc.
```

## Documentation Deliverables

### 1. Chart Analysis Document
- [ ] **Structure Overview**: Complete file listing and purpose
- [ ] **Configuration Options**: All values.yaml parameters explained
- [ ] **Template Details**: Key templates and their functions
- [ ] **Customization Guide**: How to modify for different environments

### 2. Platform Integration Guide
- [ ] **Custom Values File**: Complete custom-values.yaml for orchestrator
- [ ] **Deployment Commands**: Step-by-step deployment instructions
- [ ] **Verification Steps**: How to confirm successful deployment
- [ ] **Troubleshooting**: Common issues and solutions

### 3. MCP Server Configuration Guide
- [ ] **Server Format**: How to add new MCP servers
- [ ] **Transport Types**: Examples for stdio, SSE, and HTTP
- [ ] **Environment Variables**: Required secrets and configs
- [ ] **Testing Servers**: How to verify MCP server connectivity

## Definition of Done

✅ **Chart Review Complete**
- All files in toolman/charts/toolman/ have been reviewed
- values.yaml fully understood and documented
- All templates analyzed for compatibility

✅ **Validation Passed**
- helm lint passes without errors
- helm install --dry-run succeeds
- Test deployment to isolated namespace successful

✅ **Customization Ready**
- custom-values.yaml created for orchestrator environment
- All platform-specific requirements addressed
- Resource limits and security contexts configured

✅ **Documentation Complete**
- Chart analysis document created
- Platform integration guide written
- MCP server configuration guide prepared

✅ **Testing Verified**
- All test scenarios pass successfully
- Service connectivity confirmed
- MCP servers accessible
- Resource usage within limits

## Sign-off Criteria

- [ ] **Technical Review**: Chart modifications reviewed by platform team
- [ ] **Security Review**: Security contexts and RBAC approved
- [ ] **Documentation Review**: All guides reviewed for completeness
- [ ] **Deployment Test**: Successful test deployment with custom values
- [ ] **Performance Check**: Resource usage acceptable under load

## Notes
- This chart review forms the foundation for Tasks 2, 3, and 13
- Any issues discovered should be documented for resolution
- The chart should remain as close to upstream as possible
- Customizations should be done via values, not template modifications