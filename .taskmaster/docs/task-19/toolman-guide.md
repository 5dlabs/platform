# Toolman Guide: Component Cleanup

## Overview
Tools selected for safe component removal.

## Core Tools

### Kubernetes Tools
- `listResources`: Find components
- `getResource`: Check details
- `getEvents`: Monitor removal

### File Operations
- `read_file`: Access configs
- `write_file`: Update files
- `list_directory`: Find resources

## Implementation Flow
1. Use `listResources` to identify
2. `getResource` to verify
3. Remove components
4. Monitor with `getEvents`

## Best Practices
- Verify before removal
- Back up everything
- Remove gradually
- Monitor carefully