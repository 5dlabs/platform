# Toolman Guide: GitHub Actions Update

## Overview
Tools selected for workflow modification and testing.

## Core Tools

### Kubernetes Tools
- `createResource`: Test deployments
- `listResources`: Check resources
- `getResource`: Inspect state
- `getEvents`: Monitor updates

### File Operations
- `read_file`: Review workflows
- `write_file`: Update YAML
- `list_directory`: Find files

## Implementation Flow
1. Use `read_file` for current files
2. `write_file` to update
3. Test with K8s tools
4. Monitor with `getEvents`

## Best Practices
- Test workflow changes
- Verify integrations
- Monitor builds
- Check updates