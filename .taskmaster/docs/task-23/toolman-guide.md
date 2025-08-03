# Toolman Guide: Operator Deployment

## Overview
Tools selected for operator management.

## Core Tools

### Kubernetes Tools
- `createResource`: Deploy operators
- `listResources`: Check status
- `getResource`: Inspect config
- `getEvents`: Monitor system
- `getPodMetrics`: Check resources

### File Operations
- `read_file`: Access docs
- `write_file`: Create manifests
- `create_directory`: Set up structure

## Implementation Flow
1. Use `read_file` for reference
2. `write_file` to configure
3. `createResource` to deploy
4. Monitor with tools

## Best Practices
- Verify configurations
- Monitor resources
- Check health
- Test thoroughly