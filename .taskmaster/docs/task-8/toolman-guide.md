# Toolman Guide: API Client Implementation

## Overview
Tools selected for API implementation and testing.

## Core Tools

### Kubernetes Tools
- `getEvents`: Monitor API operations
- `listResources`: Check workflow resources
- `getResource`: Inspect workflow state

### File Operations
- `read_file`: Access API spec
- `write_file`: Create client code
- `list_directory`: Manage modules

## Implementation Flow
1. Use `read_file` for API reference
2. `write_file` to create client
3. Test with K8s tools
4. Monitor with `getEvents`

## Best Practices
- Follow API spec exactly
- Test error handling
- Monitor operations
- Document usage