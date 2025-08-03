# Toolman Guide: MCP Server Refactoring

## Overview
Tools selected for code refactoring and testing.

## Core Tools

### Kubernetes Tools
- `getEvents`: Monitor operations
- `listResources`: Check workflows
- `getResource`: Inspect state

### File Operations
- `read_file`: Review current code
- `write_file`: Update implementation
- `list_directory`: Manage codebase

## Implementation Flow
1. Use `read_file` to analyze code
2. `write_file` for updates
3. Test with K8s tools
4. Monitor with `getEvents`

## Best Practices
- Remove unused code
- Test thoroughly
- Monitor operations
- Document changes