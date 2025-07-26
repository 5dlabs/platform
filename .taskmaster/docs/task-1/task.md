# Task 1: Create Toolman Helm Chart Structure

## Overview
Review and customize the existing Toolman Helm chart for deployment to our Kubernetes cluster. The Toolman project already provides a complete Helm chart that needs to be evaluated and potentially customized for our specific orchestrator environment.

## Context
This task is the foundation for deploying Toolman as part of the MCP tool integration system. Toolman will serve as an HTTP proxy/aggregator for various MCP (Model Context Protocol) servers, enabling Claude to access remote tools like GitHub, Kubernetes, Postgres, and more.

## Objectives
1. Understand the existing Toolman Helm chart structure
2. Identify customization points and platform-specific requirements
3. Validate the chart's correctness and deployability
4. Document any necessary overrides for our environment
5. Ensure compatibility with our orchestrator namespace and infrastructure

## Technical Approach

### Chart Review Process
1. **Structural Analysis**
   - Examine directory structure: `toolman/charts/toolman/`
   - Review key files: Chart.yaml, values.yaml, templates/
   - Understand template organization and naming conventions

2. **Configuration Analysis**
   - Deep dive into values.yaml for all configurable options
   - Identify required vs optional parameters
   - Note default values and their implications
   - Review ConfigMap structure for MCP server definitions

3. **Template Examination**
   - Review deployment.yaml for container configuration
   - Check service.yaml for network exposure
   - Examine configmap.yaml for server definitions
   - Verify PVC configuration for persistence needs
   - Assess security contexts and RBAC requirements

4. **Platform Integration**
   - Determine namespace requirements (orchestrator)
   - Identify network policies needed
   - Check for resource limits and requests
   - Verify compatibility with our Kubernetes version

### Key Features to Validate
- **MCP Server Support**: stdio, SSE, and HTTP transport types
- **Pre-configured Servers**: brave-search, memory, terraform, kubernetes, solana, rustdocs, reddit
- **Persistence**: PVC configuration for stateful operations
- **Network**: Service exposure on port 3000
- **Security**: Non-root container execution, security contexts

## Implementation Steps

### Step 1: Chart Structure Review
```bash
# Navigate to chart directory
cd toolman/charts/toolman/

# List chart structure
find . -type f -name "*.yaml" -o -name "*.yml" | sort

# Review Chart.yaml for metadata
cat Chart.yaml

# Check for dependencies
grep -r "dependencies:" .
```

### Step 2: Values Analysis
```bash
# Extract all configurable values
yq eval '.. | select(has("default"))' values.yaml

# Identify required overrides
grep -B2 -A2 "required" values.yaml

# Review server configurations
yq eval '.mcpServers' values.yaml
```

### Step 3: Template Validation
```bash
# Lint the chart
helm lint .

# Dry-run with default values
helm install toolman-test . --dry-run --debug

# Generate templates for review
helm template toolman . > rendered-templates.yaml
```

### Step 4: Custom Values Creation
```yaml
# Create custom-values.yaml for our environment
namespace: orchestrator
replicaCount: 2

image:
  repository: ghcr.io/5dlabs/toolman
  tag: latest
  pullPolicy: IfNotPresent

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 100m
    memory: 256Mi

service:
  type: ClusterIP
  port: 3000

persistence:
  enabled: true
  size: 10Gi
  storageClass: "standard"

# Additional MCP servers can be added here
mcpServers:
  # Existing servers will be inherited
  # Add platform-specific servers as needed
```

### Step 5: Test Deployment
```bash
# Create test namespace
kubectl create namespace toolman-test

# Deploy with custom values
helm install toolman . -n toolman-test -f custom-values.yaml

# Verify deployment
kubectl get all -n toolman-test
kubectl logs -n toolman-test deployment/toolman

# Test service connectivity
kubectl run test-pod --rm -it --image=curlimages/curl -n toolman-test -- \
  curl http://toolman:3000/health
```

## Dependencies
- No task dependencies
- Requires Helm 3.x installed
- Kubernetes cluster access with appropriate permissions
- yq tool for YAML parsing (optional but helpful)

## Success Criteria
1. ✅ Chart passes helm lint without errors
2. ✅ Dry-run successfully generates all resources
3. ✅ Test deployment creates all expected resources
4. ✅ Service is accessible within the cluster
5. ✅ Pre-configured MCP servers are functional
6. ✅ Custom values documented for production use

## Risk Mitigation
- **Risk**: Incompatible Kubernetes API versions
  - **Mitigation**: Test on target cluster version, update apiVersions if needed
  
- **Risk**: Resource constraints in production
  - **Mitigation**: Profile resource usage during testing, set appropriate limits
  
- **Risk**: Persistence requirements unclear
  - **Mitigation**: Start with standard storage class, monitor usage patterns

## Related Tasks
- Task 2: Implement Toolman Kubernetes Deployment (builds on this chart review)
- Task 3: Create Toolman Service and ConfigMap Templates (uses findings from this task)
- Task 13: Deploy and Test Toolman Service (final deployment using reviewed chart)