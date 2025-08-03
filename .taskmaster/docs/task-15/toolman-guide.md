# Toolman Guide: Monitoring Setup

## Overview
Tools selected for monitoring implementation.

## Core Tools

### Kubernetes Tools
- `getEvents`: Monitor operations
- `listResources`: Check components
- `getResource`: Inspect config
- `getPodsLogs`: Check logging
- `getPodMetrics`: Pod metrics
- `getNodeMetrics`: Node metrics

### File Operations
- `read_file`: Access configs
- `write_file`: Update settings
- `list_directory`: Manage files

## Implementation Flow
1. Use `read_file` for configs
2. `write_file` to implement
3. Test with metrics tools
4. Verify with logs

## Best Practices
- Monitor all components
- Test alert flows
- Check metrics
- Validate logging