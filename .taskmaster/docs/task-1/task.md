# Phase 1: Architecture Simplification

## Overview
This phase focuses on removing the API server and CLI components while transforming the controller into a standalone service. The goal is to simplify the architecture and prepare for Argo Workflows integration.

## Technical Implementation Guide

### 1. API Server Removal
- Remove the Axum-based HTTP API server from orchestrator/core/src/main.rs
- Delete all handler implementations in orchestrator/core/src/handlers/
- Remove API routing and endpoint definitions
- Cleanup dependencies in Cargo.toml
- Verify no orphaned API code remains

### 2. Controller Isolation
- Update controller code to run as standalone service
- Remove any API server dependencies
- Ensure CRD reconciliation logic remains intact
- Add new logging/monitoring for standalone operation
- Test controller in isolation

### 3. CLI Tool Removal
- Delete entire orchestrator/tools/src/cli/ directory
- Remove CLI-related dependencies from project
- Update documentation to reflect CLI removal
- Verify no CLI components remain

### 4. MCP Server Updates
- Remove HTTP client code for old API
- Implement Argo Workflows API client integration
- Add workflow submission logic
- Update error handling for new flow
- Test MCP server changes

## System Design 
- Controller becomes focused reconciliation service
- Workflow submission moves to MCP via Argo
- Simplified dependency chain
- Clear separation of concerns

## Implementation Steps
1. Take inventory of all API endpoints and handlers
2. Create backup of current state
3. Remove API components systematically
4. Update and test controller in isolation
5. Remove CLI tool completely
6. Implement new MCP server logic
7. Test end-to-end workflow

## Testing Requirements
- Controller functions correctly standalone
- No API endpoints remain accessible
- CLI commands properly removed
- MCP server can submit workflows
- Error handling works as expected

## Success Criteria
- API server completely removed
- Controller running independently
- CLI tool entirely removed
- MCP server updated and functional
- No regressions in core functionality
- All tests passing