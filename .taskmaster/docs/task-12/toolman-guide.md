# Toolman Guide: ApplicationSet Creation

## Overview
Tools selected for ApplicationSet management.

## Core Tools

### Kubernetes Tools
- `createResource`: Deploy ApplicationSet
- `listResources`: Check applications
- `getResource`: Inspect details
- `getEvents`: Monitor creation

### File Operations
- `read_file`: Access specs
- `write_file`: Create manifests
- `create_directory`: Set up structure

## Implementation Flow
1. Use `read_file` for templates
2. `write_file` to create YAML
3. `createResource` to deploy
4. Monitor with `getEvents`

## Best Practices
- Test generators
- Verify templates
- Monitor creation
- Check isolation