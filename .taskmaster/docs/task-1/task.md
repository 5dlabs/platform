# Task 1: Create Toolman Helm Chart

Document and verify the existing Toolman deployment in the orchestrator namespace. Toolman has already been successfully deployed using the Helm chart located at `toolman/charts/toolman/`.

## Deployment Status
**TOOLMAN IS ALREADY DEPLOYED** in the `orchestrator` namespace with:
- Deployment: Running with 1 replica
- Service: Available at `http://toolman.orchestrator.svc.cluster.local:3000`
- ConfigMap: `toolman-config` containing MCP server definitions

## Service URL Documentation
For use in other tasks, the Toolman service can be accessed at:
- **Internal Cluster URL**: `http://toolman.orchestrator.svc.cluster.local:3000`
- **Short Service Name**: `http://toolman:3000` (when accessed from within orchestrator namespace)
- **Health Endpoint**: `http://toolman.orchestrator.svc.cluster.local:3000/health`
- **Ready Endpoint**: `http://toolman.orchestrator.svc.cluster.local:3000/ready`
- **MCP Servers List**: `http://toolman.orchestrator.svc.cluster.local:3000/mcp/servers`

## Context
This task is the foundation for deploying Toolman as part of the MCP tool integration system. Toolman will serve as an HTTP proxy/aggregator for various MCP (Model Context Protocol) servers, enabling Claude to access remote tools like GitHub, Kubernetes, Slack, and others through a single endpoint.

## Objectives
1. ~~Verify the ghcr.io/5dlabs/toolman image exists and is accessible~~ ✅ Already deployed
2. ~~Configure image pull secrets if needed~~ ✅ Already configured
3. ~~Review and validate the existing Helm chart configuration~~ ✅ Chart successfully deployed
4. ~~Deploy the chart to the orchestrator namespace~~ ✅ Deployed to MCP namespace instead
5. Verify all resources are created and running correctly
6. Test connectivity to the pre-configured MCP servers
7. Document the service URL for use in other tasks

## Verifying Existing Deployment

### Current Status
Toolman is already deployed and operational in the **orchestrator** namespace with:
- **Deployment**: `toolman`
- **Service**: `toolman` (ClusterIP on port 3000)
- **ConfigMap**: `toolman-config` containing MCP server configurations
- **Local Tools ConfigMap**: `toolman-local-tools` for filesystem and git tools
- **RBAC**: Role and RoleBinding for ConfigMap management

### Quick Verification Commands
```bash
# Check deployment status:
kubectl get deployment toolman -n orchestrator

# Output shows:
NAME      READY   UP-TO-DATE   AVAILABLE   AGE
toolman   1/1     1            1           8d

# Check service:
kubectl get service toolman -n orchestrator

# Output shows:
NAME      TYPE        CLUSTER-IP    EXTERNAL-IP   PORT(S)    AGE
toolman   ClusterIP   10.97.54.95   <none>        3000/TCP   8d
```

### Service Connectivity Tests
```bash
# Test from within cluster (any namespace):
kubectl run test-curl --rm -it --image=curlimages/curl -- \
  curl http://toolman.orchestrator.svc.cluster.local:3000/health

# Test from within orchestrator namespace:
kubectl run test-curl --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman:3000/health

# List available MCP servers:
kubectl run test-curl --rm -it --image=curlimages/curl -- \
  curl http://toolman.orchestrator.svc.cluster.local:3000/mcp/servers
```

### ConfigMap Verification
```bash
# Verify ConfigMap exists:
kubectl get configmap toolman-config -n orchestrator

# View MCP server configurations:
kubectl get configmap toolman-config -n orchestrator -o yaml
```

## Implementation Steps

Since Toolman is already deployed, the focus shifts to verification and documentation:

### Step 1: Verify Current Deployment
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

## Service Information

### Service URL
The Toolman service is accessible at the following URL from within the Kubernetes cluster:
- **Full URL**: `http://toolman.orchestrator.svc.cluster.local:3000`
- **From within orchestrator namespace**: `http://toolman:3000`
- **Service Type**: ClusterIP on port 3000

### Endpoints
- `/health` - Health check endpoint
- `/ready` - Readiness check endpoint
- `/mcp` - Main MCP proxy endpoint
- `/mcp/servers` - List configured MCP servers

### ConfigMaps Created
- `toolman-tool-catalog` - Contains discovered tool metadata (created in orchestrator namespace)

## Related Tasks
- Task 2: Implement Toolman Kubernetes Deployment (builds on this chart review)
- Task 3: Create Toolman Service and ConfigMap Templates (uses findings from this task)
- Task 13: Deploy and Test Toolman Service (final deployment using reviewed chart)