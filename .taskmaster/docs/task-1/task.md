# Configure Argo CD Infrastructure

## Overview
Troubleshoot and fix sync/health issues across all 14 Argo CD applications in the Kubernetes cluster.

## Implementation Guide

### Phase 1: Status Audit
1. Create application inventory
2. Document current sync and health status
3. Note specific errors and alerts

### Phase 2: Fix Critical Apps
1. Address OutOfSync 'arc' application
2. Resolve Unknown status for:
   - k8s-mcp
   - rustdocs-mcp 
   - twingate-pastoral
   - twingate-therapeutic

### Phase 3: Validation
1. Verify Git vs deployed resources
2. Test manual sync operations
3. Document special configurations

### Phase 4: Documentation
1. Record common issues and solutions
2. Document custom health checks
3. Create troubleshooting guide

## Technical Approach
- Use Argo CD CLI and APIs
- Review Git repositories and manifests 
- Compare with kubectl output
- Test health check configurations