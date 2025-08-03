# Toolman Guide: Parallel Systems

## Overview
Tools selected for parallel system operation.

## Core Tools

### Kubernetes Tools
- `getEvents`: Monitor operations
- `listResources`: Check systems
- `getResource`: Inspect state
- `getPodsLogs`: Track logs
- `getPodMetrics`: Monitor performance

### File Operations
- `read_file`: Access configs
- `write_file`: Update system
- `list_directory`: Manage files

## Implementation Flow
1. Use `read_file` for setup
2. `write_file` to configure
3. Monitor with tools
4. Check performance

## Best Practices
- Track both systems
- Monitor performance
- Compare metrics
- Test thoroughly