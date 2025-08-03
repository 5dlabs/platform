# Create Feature Branch ApplicationSet

## Overview
Create ApplicationSet for dynamic feature deployments.

## Implementation Guide

### Phase 1: Generator Setup
1. Git configuration:
   - Branch detection
   - Path matching
2. Template design:
   - Dynamic naming
   - Resource mapping

### Phase 2: Namespace
1. Isolation:
   - Creation policy
   - Network rules
2. Resources:
   - Quota limits
   - Cleanup rules

### Phase 3: Automation
1. Sync config:
   - Time windows
   - Auto-sync
2. Branch handling:
   - Detection
   - Cleanup

### Phase 4: Testing
1. Branch creation
2. Isolation verify
3. Cleanup check

## Technical Requirements
- Use infra/gitops/appsets/
- Configure generators
- Set resource limits
- Enable isolation