# Toolman Guide: GitOps Migration

## Overview
This guide explains the tools selected for implementing GitOps-based deployment using Argo CD.

## Core Tools

### Kubernetes Operations
- **getEvents**: Monitor Kubernetes events
  - Use for tracking deployment issues
  - Example: Monitor sync events

- **listResources**: List Kubernetes resources
  - Use to verify deployments
  - Example: Check application status

- **getResource**: Get specific resource details
  - Use to inspect configurations
  - Example: Check application settings

- **getPodMetrics**: Monitor pod resources
  - Use for application performance
  - Example: Track sync pod usage

- **getNodeMetrics**: Monitor node resources
  - Use for cluster health
  - Example: Check node capacity

### Helm Management
- **helmGet**: Get Helm release details
  - Use to verify installations
  - Example: Check Argo CD release

- **helmHistory**: View release history
  - Use to track changes
  - Example: Monitor updates

- **helmList**: List all releases
  - Use for release management
  - Example: Check all applications

### File Operations
- **read_file**: Read configuration files
  - Use for manifest management
  - Example: Read Application YAML

- **write_file**: Create new configurations
  - Use for new manifests
  - Example: Create ApplicationSet

- **edit_file**: Update configurations
  - Use for manifest updates
  - Example: Modify sync policy

- **list_directory**: Browse configurations
  - Use for file management
  - Example: List manifests

- **search_files**: Find configuration files
  - Use to locate resources
  - Example: Find application configs

### Research
- **brave_web_search**: Research Argo CD
  - Use for best practices
  - Example: Search sync patterns

## Implementation Flow
1. Use file tools to manage manifests
2. Use Helm tools to track releases
3. Use Kubernetes tools to monitor
4. Use metrics tools to validate
5. Use search for documentation

## Best Practices
- Validate manifests before applying
- Monitor sync operations
- Track resource usage
- Keep configurations versioned
- Document all changes

## Troubleshooting
- Check events for sync issues
- Monitor pod metrics
- Review Helm history
- Validate manifest syntax
- Check node resources