# Remove Rust API Server Code

## Overview
Remove the Axum-based HTTP API server from the orchestrator core, retaining only controller functionality.

## Implementation Guide

### Phase 1: Pre-Removal
1. Verify Argo Workflows system
2. Create backup branch
3. Document current API endpoints

### Phase 2: Code Removal
1. Remove from main.rs:
   - HTTP server init
   - API routes
   - Middleware
2. Delete handlers directory
3. Update main function
4. Clean Cargo.toml

### Phase 3: Configuration
1. Update logging setup
2. Remove API settings
3. Verify controller config

### Phase 4: Testing
1. Build verification
2. Controller reconciliation
3. Resource watching
4. Logging validation

## Technical Requirements
- Dependencies to remove:
  - axum
  - tower
  - API-related crates
- Keep controller initialization
- Maintain CRD reconciliation