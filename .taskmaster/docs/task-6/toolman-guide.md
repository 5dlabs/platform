# Toolman Guide: Workflow Template Creation

## Overview
Tools selected for Kubernetes resource creation and validation.

## Core Tools

### Kubernetes Tools
- `createResource`: Deploy workflow template
- `listResources`: Check existing templates
- `getResource`: Inspect configuration
- `getEvents`: Monitor execution

### File Operations
- `read_file`: Access API spec
- `write_file`: Create template YAML
- `create_directory`: Set up structure

## Implementation Flow
1. Use `read_file` for API reference
2. `write_file` to create template
3. `createResource` to deploy
4. `getEvents` to monitor

## Best Practices
- Validate against API spec
- Check resource allocation
- Monitor template events
- Test deployment flow