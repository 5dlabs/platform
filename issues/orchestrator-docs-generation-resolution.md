# Orchestrator Documentation Generation Failure - Resolution Guide

## Problem Summary
The `orchestrator task init-docs` command fails during job submission with error "Failed to send documentation generation request". This occurs after successful initialization but before the TaskRun creation phase.

## Root Cause Analysis

### Primary Issue: Service Endpoint Mismatch
Based on the error pattern and the presence of a kubectl port-forward, the most likely cause is that the orchestrator CLI is configured to connect to a different endpoint than what's available through the port-forward.

The CLAUDE.md file specifies:
```bash
export ORCHESTRATOR_API_URL="http://orchestrator.orchestrator.svc.cluster.local/api/v1"
```

However, with kubectl port-forward running on `localhost:8080`, the CLI needs to use:
```bash
export ORCHESTRATOR_API_URL="http://localhost:8080/api/v1"
```

## Immediate Resolution Steps

### 1. Update Environment Configuration
```bash
# Set the correct orchestrator API URL for local development
export ORCHESTRATOR_API_URL="http://localhost:8080/api/v1"

# Verify the environment variable is set
echo $ORCHESTRATOR_API_URL
```

### 2. Verify Port-Forward is Active
```bash
# Check if port-forward is running
ps aux | grep "kubectl port-forward"

# If not running, start it:
kubectl port-forward -n orchestrator service/orchestrator 8080:80
```

### 3. Test Service Connectivity
```bash
# Test the health endpoint
curl -v http://localhost:8080/api/v1/health

# If using the cluster URL fails, ensure you're in the cluster context
# or using port-forward for local development
```

### 4. Retry Documentation Generation
```bash
# Navigate to the example directory
cd /Users/jonathonfritz/platform/example

# Run the init-docs command with proper environment
ORCHESTRATOR_API_URL="http://localhost:8080/api/v1" /Users/jonathonfritz/platform/orchestrator/target/release/orchestrator task init-docs
```

## Alternative Solutions

### Option 1: Direct Cluster Access (If Inside Cluster)
If running from within the Kubernetes cluster:
```bash
export ORCHESTRATOR_API_URL="http://orchestrator.orchestrator.svc.cluster.local/api/v1"
```

### Option 2: Using Orchestrator Service Directly
If the orchestrator has a LoadBalancer or NodePort service:
```bash
# Get the external service endpoint
kubectl get svc -n orchestrator orchestrator

# Use the external IP/port
export ORCHESTRATOR_API_URL="http://<EXTERNAL-IP>:<PORT>/api/v1"
```

### Option 3: Manual Documentation Generation
If the orchestrator service remains unavailable:

1. **Use Task Master's built-in documentation**:
   ```bash
   cd /Users/jonathonfritz/platform/example
   task-master generate  # Regenerate task markdown files
   ```

2. **Create a documentation template manually**:
   ```bash
   # Create documentation structure
   mkdir -p .taskmaster/docs/generated
   
   # Generate overview from tasks
   cat > .taskmaster/docs/generated/project-overview.md << 'EOF'
   # Project Overview
   
   Generated from Task Master task structure.
   
   ## Tasks
   $(task-master list --format=markdown)
   
   ## Complexity Analysis
   $(task-master complexity-report --format=markdown)
   EOF
   ```

## Debugging Steps

### 1. Enable Debug Logging
```bash
# Run with debug output
ORCHESTRATOR_DEBUG=true ORCHESTRATOR_API_URL="http://localhost:8080/api/v1" \
  /Users/jonathonfritz/platform/orchestrator/target/release/orchestrator task init-docs
```

### 2. Check Service Logs
```bash
# Get orchestrator pod name
ORCHESTRATOR_POD=$(kubectl get pods -n orchestrator -l app=orchestrator -o jsonpath='{.items[0].metadata.name}')

# View logs
kubectl logs -n orchestrator $ORCHESTRATOR_POD --tail=100 -f
```

### 3. Verify API Authentication
```bash
# Check if API requires authentication
curl -v -X GET http://localhost:8080/api/v1/tasks

# If authentication is required, set the token
export ORCHESTRATOR_API_TOKEN="your-token-here"
```

## Prevention Measures

### 1. Environment Configuration File
Create `.env.orchestrator` in the project root:
```bash
# Local development configuration
ORCHESTRATOR_API_URL=http://localhost:8080/api/v1
# ORCHESTRATOR_API_TOKEN=your-token-here  # If needed
```

### 2. Shell Alias for Consistency
Add to your shell profile:
```bash
alias orchestrator-local='ORCHESTRATOR_API_URL="http://localhost:8080/api/v1" orchestrator'
```

### 3. Pre-flight Check Script
Create `check-orchestrator.sh`:
```bash
#!/bin/bash
echo "Checking orchestrator connectivity..."

# Check environment
if [ -z "$ORCHESTRATOR_API_URL" ]; then
    echo "WARNING: ORCHESTRATOR_API_URL not set"
    echo "Using default: http://localhost:8080/api/v1"
    export ORCHESTRATOR_API_URL="http://localhost:8080/api/v1"
fi

# Test connectivity
if curl -s -f "$ORCHESTRATOR_API_URL/health" > /dev/null 2>&1; then
    echo "✓ Orchestrator service is accessible at $ORCHESTRATOR_API_URL"
else
    echo "✗ Cannot reach orchestrator service at $ORCHESTRATOR_API_URL"
    echo "  Ensure kubectl port-forward is running:"
    echo "  kubectl port-forward -n orchestrator service/orchestrator 8080:80"
    exit 1
fi
```

## Expected Success Output
When properly configured, the command should output:
```
Initializing documentation generator...
Repository: https://github.com/5dlabs/agent-platform.git
Working directory: example
Source branch: main
Target branch: main
Submitting documentation generation job...
Documentation generation job submitted successfully
Task ID: 999999
Status: Submitted
Monitor progress with: orchestrator task status 999999
```

## Additional Notes

1. **Working Directory Context**: The orchestrator correctly detects the working directory as "example" when run from within that directory. This is expected behavior.

2. **Repository Access**: Ensure the orchestrator service has necessary GitHub access tokens configured if the repository is private.

3. **Task Master Integration**: The successful Phase 1 (Task Master planning) indicates the local tooling works correctly. The issue is isolated to the orchestrator service communication.

## Contact for Further Issues
If problems persist after following this guide:
1. Check orchestrator service deployment status
2. Review Kubernetes ingress/service configuration
3. Verify any security policies or network restrictions
4. Consult platform operations team for infrastructure-specific issues