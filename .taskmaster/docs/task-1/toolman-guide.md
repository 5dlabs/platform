# Toolman Guide: Argo CD Configuration

## Overview
Tools selected for Kubernetes resource management and file operations.

## Core Tools

### Kubernetes Tools
- `getEvents`: Monitor cluster events during sync
- `listResources`: Check application resources
- `getResource`: Inspect specific resources
- `describeResource`: Get detailed resource state

### File Operations
- `read_file`: Review manifests and configs
- `write_file`: Update configuration files
- `list_directory`: Navigate project structure

## Implementation Flow
1. Use `listResources` to check app states
2. `getEvents` to monitor sync operations
3. `getResource`/`describeResource` for details
4. File tools for manifest management

## Best Practices
- Monitor events during sync operations
- Compare resource states before/after changes
- Keep manifest files organized
- Document configuration updates