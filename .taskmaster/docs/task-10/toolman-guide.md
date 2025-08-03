# Toolman Guide: Status Monitoring

## Overview
Tools selected for workflow monitoring and logging.

## Core Tools

### Kubernetes Tools
- `getEvents`: Watch status changes
- `listResources`: Check workflows
- `getResource`: Inspect details
- `getPodsLogs`: Stream logs

### File Operations
- `read_file`: Access API spec
- `write_file`: Create monitor
- `list_directory`: Manage code

## Implementation Flow
1. Use `read_file` for API docs
2. `write_file` to implement
3. Test with K8s tools
4. Stream logs with `getPodsLogs`

## Best Practices
- Watch for changes
- Cache effectively
- Stream logs efficiently
- Monitor metrics