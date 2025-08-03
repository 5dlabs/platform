# Acceptance Criteria: Architecture Simplification

## 1. API Server Removal
- [ ] API server code removed from main.rs
- [ ] All handler implementations deleted
- [ ] API routing/endpoints removed
- [ ] Dependencies cleaned up
- [ ] No orphaned API code remains
- [ ] Project builds successfully without API

## 2. Controller Isolation
- [ ] Controller runs as standalone service
- [ ] No API dependencies present
- [ ] CRD reconciliation working correctly
- [ ] Logging/monitoring functional
- [ ] Controller tests passing
- [ ] Metrics collection working

## 3. CLI Tool Removal
- [ ] CLI directory completely removed
- [ ] CLI dependencies removed
- [ ] Documentation updated
- [ ] No CLI components remain
- [ ] Build successful without CLI
- [ ] No broken references

## 4. MCP Server Updates
- [ ] Old HTTP client removed
- [ ] Argo Workflows client added
- [ ] Workflow submission functional
- [ ] Error handling implemented
- [ ] Integration tests passing
- [ ] Performance metrics normal

## General Requirements
- [ ] No regressions in core functionality
- [ ] All unit tests passing
- [ ] Integration tests passing
- [ ] Build pipeline successful
- [ ] Documentation updated
- [ ] Clean error logs

## Test Cases

### Controller Tests
1. Test standalone operation
2. Verify CRD reconciliation
3. Check monitoring/logging
4. Test error scenarios
5. Validate metrics

### MCP Server Tests
1. Test workflow submission
2. Verify error handling
3. Check integration points
4. Test performance
5. Validate logging

### System Tests
1. Full end-to-end workflow
2. Error scenario handling
3. Performance validation
4. Load testing
5. Integration verification