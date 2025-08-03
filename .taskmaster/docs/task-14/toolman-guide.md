# Toolman Guide: MCP Server Integration

## Overview
Tools selected for workflow integration and testing.

## Core Tools

### Kubernetes Tools
- `createResource`: Test submissions
- `listResources`: Check workflows
- `getResource`: Inspect details
- `getEvents`: Monitor jobs

### File Operations
- `read_file`: Review code
- `write_file`: Update service
- `list_directory`: Manage files

## Implementation Flow
1. Use `read_file` for current code
2. `write_file` to implement
3. Test with K8s tools
4. Monitor with `getEvents`

## Best Practices
- Validate inputs
- Handle errors
- Monitor submissions
- Test thoroughly