# Toolman Guide: Application Manifests

## Overview
Tools selected for Argo CD manifest creation and testing.

## Core Tools

### Kubernetes Tools
- `createResource`: Deploy applications
- `listResources`: Check status
- `getResource`: Inspect details
- `getEvents`: Monitor sync

### File Operations
- `read_file`: Access API spec
- `write_file`: Create manifests
- `create_directory`: Set up structure

## Implementation Flow
1. Use `read_file` for API docs
2. `write_file` to create YAMLs
3. `createResource` to deploy
4. Monitor with `getEvents`

## Best Practices
- Follow API schema
- Test deployments
- Monitor events
- Document structure