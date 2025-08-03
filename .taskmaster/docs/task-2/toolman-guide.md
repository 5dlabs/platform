# Toolman Guide: Argo Workflows Configuration

## Overview
Tools selected for Kubernetes resource creation and file management.

## Core Tools

### Kubernetes Tools
- `createResource`: Deploy service accounts and configs
- `listResources`: Monitor resources
- `getResource`: Inspect specific resources
- `describeResource`: Detailed resource analysis
- `getEvents`: Track configuration changes

### File Operations
- `read_file`: Access existing configs
- `write_file`: Create new config files
- `create_directory`: Set up workflow templates
- `list_directory`: Manage project structure

## Implementation Flow
1. Use `createResource` for service accounts/RBAC
2. File tools for template management
3. `listResources`/`getResource` for validation
4. `getEvents` to monitor changes

## Best Practices
- Validate resources after creation
- Document all configurations
- Use consistent directory structure
- Monitor events for issues