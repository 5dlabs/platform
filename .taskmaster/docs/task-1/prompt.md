# Architecture Simplification Agent Prompt

You are an AI agent tasked with simplifying the platform architecture by removing API and CLI components while updating the controller to be standalone.

## Context
The platform is transitioning to Argo Workflows from a custom API. Your task is to remove unnecessary components and prepare for this migration.

## Primary Objectives
1. Remove API server completely
2. Remove CLI tool entirely
3. Update controller to standalone
4. Modify MCP server for Argo

## Technical Requirements

### API Removal
- Delete HTTP API server from main.rs
- Remove all handler implementations
- Remove API routing/endpoints
- Clean up dependencies
- Verify no orphaned code

### Controller Updates
- Update for standalone operation
- Remove API dependencies
- Maintain CRD reconciliation
- Add standalone monitoring
- Ensure isolation works

### CLI Removal
- Delete CLI directory
- Remove CLI dependencies
- Update related docs
- Verify full removal

### MCP Server Changes
- Remove old HTTP client
- Add Argo Workflows client
- Implement submission logic
- Update error handling
- Test new workflow

## Success Criteria
1. API server completely removed
2. Controller running standalone
3. CLI tool deleted
4. MCP server updated
5. All tests passing
6. No regressions

## Implementation Notes
- Work systematically through each component
- Test thoroughly after each step
- Document all changes
- Maintain backwards compatibility
- Focus on clean removal