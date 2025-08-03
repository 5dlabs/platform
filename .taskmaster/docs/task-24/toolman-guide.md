# Toolman Guide: Intake Workflow

## Overview
Tools selected for workflow implementation.

## Core Tools

### Kubernetes Tools
- `createResource`: Deploy workflow
- `listResources`: Check status
- `getResource`: Inspect config
- `getEvents`: Monitor triggers
- `getPodsLogs`: Debug issues

### File Operations
- `read_file`: Access configs
- `write_file`: Create YAML
- `create_directory`: Set up structure

## Implementation Flow
1. Use `read_file` for specs
2. `write_file` to configure
3. `createResource` to deploy
4. Monitor with tools

## Best Practices
- Test each step
- Monitor events
- Check logs
- Verify triggers