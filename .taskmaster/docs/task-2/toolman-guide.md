# Toolman Guide: Argo Workflows Integration

## Overview
This guide explains the tools selected for implementing Argo Workflows integration, focusing on workflow management and monitoring.

## Core Tools

### Kubernetes Operations
- **getEvents**: Monitor Kubernetes events
  - Use for debugging workflow issues
  - Example: Check event logs during deployment

- **listResources**: List Kubernetes resources
  - Use to verify workflow deployments
  - Example: List all workflow templates

- **getResource**: Get specific resource details
  - Use to inspect workflow configurations
  - Example: Check specific workflow template

- **getPodMetrics**: Monitor resource usage
  - Use for performance tracking
  - Example: Check workflow pod resource usage

### Helm Management
- **helmGet**: Get Helm release details
  - Use to verify Argo installation
  - Example: Check Argo Workflows release

- **helmHistory**: View release history
  - Use to track configuration changes
  - Example: Monitor Argo updates

### File Operations
- **read_file**: Read configuration files
  - Use for template management
  - Example: Read workflow templates

- **write_file**: Create new configuration files
  - Use for new templates
  - Example: Create workflow YAML

- **edit_file**: Update existing configurations
  - Use for template modifications
  - Example: Update resource limits

- **list_directory**: Browse configuration files
  - Use for file management
  - Example: List template directories

### Research
- **brave_web_search**: Research Argo features
  - Use for best practices
  - Example: Search workflow patterns

## Implementation Flow
1. Use Helm tools to verify Argo setup
2. Use file tools to manage templates
3. Use Kubernetes tools to deploy
4. Use metrics tools to monitor
5. Use search for documentation

## Best Practices
- Verify configurations before applying
- Monitor resource usage regularly
- Keep templates versioned
- Document all configurations

## Troubleshooting
- Check events for workflow issues
- Monitor pod metrics for problems
- Review Helm history for changes
- Validate template syntax