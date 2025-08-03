# Modify GitHub Actions Workflows

## Overview
Update workflows for build-only operation with Argo CD integration.

## Implementation Guide

### Phase 1: Workflow Updates
1. Remove components:
   - Helm deployments
   - intake.yml workflow
2. Build process:
   - Image building
   - Tag generation

### Phase 2: Optimization
1. Build improvements:
   - Layer caching
   - Parallel builds
2. Performance:
   - Dependency cache
   - Process time

### Phase 3: Integration
1. Argo CD updates:
   - Image tags
   - Sync triggers
2. Notifications:
   - Build status
   - Error alerts

### Phase 4: Testing
1. Build verification
2. Tag updates
3. Sync checks

## Technical Requirements
- Remove deploy steps
- Use image tags
- Enable caching
- Add notifications