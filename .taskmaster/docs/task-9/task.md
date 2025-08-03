# Refactor MCP Server for Argo Workflows

## Overview
Remove old HTTP client and integrate new Argo Workflows client.

## Implementation Guide

### Phase 1: Removal
1. Remove HTTP client code
2. Clean dependencies
3. Update configuration

### Phase 2: Integration
1. Add workflow client:
   - Job submission
   - Status checks
   - Log retrieval
2. Error handling:
   - Argo-specific errors
   - Graceful degradation

### Phase 3: Migration
1. Backward compatibility
2. In-progress jobs
3. Error recovery

### Phase 4: Testing
1. Unit tests
2. Integration tests
3. Load testing

## Technical Requirements
- Remove old client
- Use workflow templates
- Handle errors
- Maintain compatibility