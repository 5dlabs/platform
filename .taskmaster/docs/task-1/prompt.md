# Autonomous Agent Prompt: Review and Customize Toolman Helm Chart

## Context
You are tasked with reviewing and customizing the existing Toolman Helm chart located at `toolman/charts/toolman/` for deployment to our Kubernetes orchestrator environment. Toolman is a critical component that will serve as an HTTP proxy for MCP (Model Context Protocol) servers, enabling Claude to access various remote tools.

## Your Mission
Conduct a comprehensive review of the Toolman Helm chart and prepare it for deployment in our specific environment. Focus on understanding the existing structure, identifying customization needs, and ensuring the chart is production-ready.

## Step-by-Step Instructions

### 1. Initial Chart Exploration
- Navigate to `toolman/charts/toolman/` directory
- List all files to understand the structure
- Read Chart.yaml to understand the chart metadata and version
- Review the README.md if present for any important notes

### 2. Deep Dive into values.yaml
- Analyze every section of values.yaml
- Document all configurable parameters and their defaults
- Pay special attention to:
  - Image configuration (repository, tag, pull policy)
  - Resource limits and requests
  - Service configuration (type, ports)
  - Persistence settings
  - Security contexts
  - MCP server configurations
- Create a comprehensive list of values that need customization

### 3. Template Analysis
- Review each template file in the templates/ directory:
  - deployment.yaml: Container specs, volumes, security
  - service.yaml: Network exposure settings
  - configmap.yaml: MCP server definitions structure
  - persistentvolumeclaim.yaml: Storage requirements
  - Any RBAC-related templates
- For each template, note:
  - Kubernetes API versions used
  - Resource naming conventions
  - Label and annotation patterns
  - Use of Helm functions and variables

### 4. MCP Server Configuration Review
- Examine the pre-configured MCP servers in the ConfigMap
- Understand the structure for each transport type:
  - stdio-based servers
  - SSE-based servers
  - HTTP-based servers
- Document the format for adding new servers
- Note any environment variables or secrets required

### 5. Validation and Testing
- Run `helm lint toolman/charts/toolman/` and address any issues
- Execute dry-run: `helm install toolman-test toolman/charts/toolman/ --dry-run --debug`
- Review the generated YAML for correctness
- Create a test values file with our specific overrides:
  ```yaml
  namespace: orchestrator
  replicaCount: 2
  # Add other customizations
  ```
- Test with custom values: `helm install toolman-test toolman/charts/toolman/ -f test-values.yaml --dry-run`

### 6. Documentation Creation
- Create a `platform-customizations.md` file documenting:
  - Required value overrides for our environment
  - Any template modifications needed
  - Security considerations
  - Resource recommendations
  - Integration points with our orchestrator
- Include example commands for deployment
- Document any prerequisites or dependencies

### 7. Production Readiness Checklist
- [ ] Chart passes helm lint
- [ ] All API versions compatible with our Kubernetes version
- [ ] Resource limits appropriate for expected load
- [ ] Security contexts properly configured
- [ ] Persistence configured if needed
- [ ] Service exposure aligns with network policies
- [ ] ConfigMap structure supports all required MCP servers
- [ ] No hardcoded values that should be configurable
- [ ] Labels and annotations follow our conventions
- [ ] Health checks properly configured

## Expected Deliverables
1. Detailed analysis report of the existing chart
2. Custom values.yaml file for our environment
3. Documentation of any required chart modifications
4. Test results showing successful deployment
5. Production deployment guide

## Important Considerations
- The chart should deploy to the 'orchestrator' namespace
- Toolman will be accessed internally as http://toolman-service:3000
- Ensure compatibility with our zero-hardcoding policy
- The ConfigMap will be the single source of truth for available tools
- Consider high availability with 2+ replicas

## Success Metrics
- Chart deploys successfully without manual intervention
- All pre-configured MCP servers are accessible
- Service is reachable from within the cluster
- Resource usage is within acceptable limits
- Configuration is maintainable and well-documented

Proceed with the review and provide detailed findings at each step. Focus on making the chart production-ready while maintaining simplicity and adherence to Kubernetes best practices.