# Create Argo CD Application Manifests

## Overview
Create Application manifests for Argo CD deployments.

## Implementation Guide

### Phase 1: Structure
1. Source config:
   - Repository
   - Path
   - Target cluster
2. Sync policies:
   - Automated sync
   - Prune enabled
   - Self-healing

### Phase 2: Health Checks
1. Service checks:
   - Custom checks
   - Default checks
2. Retry config:
   - Failed syncs
   - Backoff policy

### Phase 3: Resources
1. Tracking setup:
   - Annotations
   - Labels
2. Namespace scope:
   - Permissions
   - Boundaries

### Phase 4: Testing
1. Sync validation
2. Health checks
3. Resource management

## Technical Requirements
- Use infra/gitops/apps/
- Reference API spec
- Configure notifications
- Document structure