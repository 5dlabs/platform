# Autonomous Agent Prompt: Deploy Toolman Helm Chart

## Context
You are tasked with deploying the existing Toolman Helm chart located at `toolman/charts/toolman/` to the orchestrator namespace in our Kubernetes cluster. The chart is fully developed and ready for deployment - your focus is on successful deployment and verification rather than development or extensive customization.

## Current Situation
- **Chart Status**: Fully developed and ready at `toolman/charts/toolman/`
- **Deployment Status**: NOT deployed to cluster yet
- **Namespace State**: orchestrator namespace exists but only contains the orchestrator deployment
- **Image**: Uses ghcr.io/5dlabs/toolman

## Your Mission
Deploy the Toolman Helm chart to the orchestrator namespace, ensuring all resources are created successfully and the service is operational.

## Step-by-Step Instructions

### 1. Pre-Deployment Checks
```bash
# Verify image accessibility
docker pull ghcr.io/5dlabs/toolman:latest || echo "May need authentication"

# Check if ghcr-secret exists in orchestrator namespace
kubectl get secret ghcr-secret -n orchestrator

# If secret doesn't exist and is needed, create it:
# kubectl create secret docker-registry ghcr-secret \
#   --docker-server=ghcr.io \
#   --docker-username=<username> \
#   --docker-password=<token> \
#   -n orchestrator

# Verify namespace exists
kubectl get namespace orchestrator
```

### 2. Chart Validation
```bash
# Navigate to chart directory
cd toolman/charts/toolman/

# Run helm lint
helm lint .

# Perform dry-run with debug
helm install toolman . \
  --namespace orchestrator \
  --dry-run --debug

# Review the generated manifests
helm template toolman . --namespace orchestrator > /tmp/toolman-manifests.yaml
cat /tmp/toolman-manifests.yaml
```

### 3. Prepare Deployment Values
Create a minimal values override file for the orchestrator deployment:
  ```yaml
# orchestrator-values.yaml
  namespace: orchestrator

# Image pull secret if needed
imagePullSecrets:
  - name: ghcr-secret

# Ensure service name is consistent
fullnameOverride: "toolman"

# Any other necessary overrides based on dry-run results
```

### 4. Deploy the Chart
```bash
# Install the chart
helm install toolman . \
  --namespace orchestrator \
  --values orchestrator-values.yaml

# Monitor the deployment
kubectl get pods -n orchestrator -l app.kubernetes.io/name=toolman -w

# Check deployment status
kubectl rollout status deployment/toolman -n orchestrator
```

### 5. Verify Deployment
```bash
# Check all resources created
kubectl get all -n orchestrator -l app.kubernetes.io/name=toolman

# Verify ConfigMap with MCP servers
kubectl get configmap -n orchestrator | grep toolman
kubectl describe configmap toolman-config -n orchestrator

# Check PVC if persistence is enabled
kubectl get pvc -n orchestrator

# Verify service endpoints
kubectl get endpoints toolman -n orchestrator
```

### 6. Test Service Connectivity
```bash
# Port-forward for local testing
kubectl port-forward -n orchestrator svc/toolman 3000:3000 &

# Test health endpoint
curl http://localhost:3000/health

# Test ready endpoint
curl http://localhost:3000/ready

# List available MCP servers
curl http://localhost:3000/mcp/servers

# Kill port-forward
kill %1
```

### 7. Test Internal Cluster Access
```bash
# Test from within cluster
kubectl run test-curl --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman:3000/health

# Test full FQDN
kubectl run test-curl --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman.orchestrator.svc.cluster.local:3000/ready

# Test MCP server list
kubectl run test-curl --rm -it --image=curlimages/curl -n orchestrator -- \
  curl http://toolman.orchestrator.svc.cluster.local:3000/mcp/servers
```

### 8. Verify MCP Server Functionality
Test that the pre-configured MCP servers are accessible:
```bash
# Check logs for any errors
kubectl logs -n orchestrator deployment/toolman

# Look for successful MCP server initialization
kubectl logs -n orchestrator deployment/toolman | grep -i "server\|mcp"

# If there are init containers, check their logs too
kubectl logs -n orchestrator deployment/toolman -c init-container-name
```

### 9. Document Results
Create a deployment summary including:
- Exact helm install command used
- Any values overrides applied
- List of created resources
- Service endpoints and access methods
- Any issues encountered and resolutions
- Health check results

## Expected Outcomes
1. ✅ Toolman deployment running with healthy pods
2. ✅ Service accessible at http://toolman.orchestrator.svc.cluster.local:3000
3. ✅ ConfigMap contains all 7 pre-configured MCP servers
4. ✅ Health and ready endpoints responding
5. ✅ No errors in pod logs
6. ✅ If persistence enabled, PVC is bound

## Troubleshooting Guide

### Image Pull Issues
```bash
# Check image pull errors
kubectl describe pod -n orchestrator -l app.kubernetes.io/name=toolman

# If authentication needed, ensure secret is referenced in values
imagePullSecrets:
  - name: ghcr-secret
```

### Pod Not Starting
```bash
# Check pod events
kubectl get events -n orchestrator --field-selector involvedObject.name=toolman-xxxxx

# Check init container logs if present
kubectl logs -n orchestrator <pod-name> -c <init-container-name>
```

### Service Not Accessible
```bash
# Verify endpoints
kubectl get endpoints toolman -n orchestrator

# Check service definition
kubectl get svc toolman -n orchestrator -o yaml

# Verify pod labels match service selector
kubectl get pods -n orchestrator --show-labels
```

## Important Notes
- The chart is already complete - avoid making modifications unless absolutely necessary
- Focus on successful deployment rather than customization
- Document any required secrets or configurations for future deployments
- Ensure the deployment is reproducible with the documented commands

Proceed with the deployment and provide detailed results at each step.