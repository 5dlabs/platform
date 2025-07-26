# Task 3: Create Toolman Service and ConfigMap Templates

## Overview
Review and customize the existing Kubernetes Service and ConfigMap templates for Toolman. These templates are crucial for exposing the Toolman HTTP proxy and defining the MCP server configurations that will be available to Claude agents.

## Context
The Service and ConfigMap are foundational components of the Toolman deployment:
- **Service**: Exposes Toolman on port 3000 for internal cluster access
- **ConfigMap**: Contains the server definitions that Toolman will proxy, serving as the single source of truth for available MCP tools

This ConfigMap is particularly critical as it's read by both Toolman and the Docs Agent for tool discovery.

## Objectives
1. Review and validate the Service template configuration
2. Understand the ConfigMap template and JSON generation logic
3. Document MCP server configuration formats for all transport types
4. Test custom server additions and ConfigMap generation
5. Ensure alignment with the zero-hardcoding architecture principle

## Technical Architecture

### Service Architecture
The Service provides stable network access to Toolman pods:
```
┌─────────────────────┐
│   Cluster Network   │
├─────────────────────┤
│                     │
│  toolman-service    │
│    (port 3000)      │
│         │           │
│         ▼           │
│   Load Balancer     │
│    /    |    \      │
│   /     |     \     │
│  Pod1  Pod2  Pod3   │
└─────────────────────┘
```

### ConfigMap as Single Source of Truth
The ConfigMap serves multiple critical functions:
```
┌──────────────────────┐     ┌──────────────────────┐
│ toolman-servers-     │────▶│    Toolman Pod       │
│ config ConfigMap     │     │  (reads at startup)  │
│                      │     └──────────────────────┘
│ servers-config.json: │
│ {                    │     ┌──────────────────────┐
│   "servers": {       │────▶│    Docs Agent        │
│     "github": {...}, │     │ (discovers tools)    │
│     "k8s": {...}     │     └──────────────────────┘
│   }                  │
│ }                    │     ┌──────────────────────┐
│                      │────▶│    Code Agent        │
│                      │     │ (validates tools)    │
└──────────────────────┘     └──────────────────────┘
```

## Implementation Details

### 1. Service Template Structure
```yaml
# toolman/charts/toolman/templates/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "toolman.fullname" . }}
  labels:
    {{- include "toolman.labels" . | nindent 4 }}
  {{- with .Values.service.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
      name: http
  selector:
    {{- include "toolman.selectorLabels" . | nindent 4 }}
```

**Key Configuration Points:**
- Port 3000 exposed for HTTP traffic
- ClusterIP type for internal access only
- Selector must match deployment pod labels
- Named port "http" for clarity

### 2. ConfigMap Template Structure
```yaml
# toolman/charts/toolman/templates/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "toolman.fullname" . }}-config
  labels:
    {{- include "toolman.labels" . | nindent 4 }}
data:
  servers-config.json: |
    {{
      $servers := dict "servers" .Values.mcpServers
    }}
    {{ $servers | toJson | nindent 4 }}
```

### 3. MCP Server Configuration Formats

#### STDIO Transport
Most common transport for command-line MCP servers:
```json
{
  "github": {
    "transport": "stdio",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "env": {
      "GITHUB_TOKEN": "${GITHUB_TOKEN}"
    }
  },
  "kubernetes": {
    "transport": "stdio",
    "command": "docker",
    "args": [
      "run", "--rm", "-i",
      "-v", "/home/appuser/.kube:/home/appuser/.kube:ro",
      "ginnux/k8s-mcp-server:latest"
    ]
  }
}
```

#### SSE (Server-Sent Events) Transport
```json
{
  "memory": {
    "transport": "sse",
    "url": "http://localhost:3001/sse",
    "headers": {
      "Authorization": "Bearer ${API_TOKEN}"
    }
  }
}
```

#### HTTP Transport
```json
{
  "terraform": {
    "transport": "http",
    "url": "http://terraform-mcp:8080",
    "headers": {
      "Content-Type": "application/json"
    }
  }
}
```

### 4. Values.yaml Configuration
```yaml
# values.yaml example
service:
  type: ClusterIP
  port: 3000
  annotations: {}
  
mcpServers:
  # GitHub MCP Server
  github:
    transport: stdio
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-github"
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
  
  # Brave Search
  brave-search:
    transport: stdio
    command: npx
    args:
      - "-y" 
      - "@modelcontextprotocol/server-brave-search"
    env:
      BRAVE_API_KEY: "${BRAVE_API_KEY}"
  
  # PostgreSQL
  postgres:
    transport: stdio
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-postgres"
    env:
      POSTGRES_URL: "${POSTGRES_URL}"
```

## Testing Strategy

### 1. Service Template Validation
```bash
# Generate service YAML
helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/service.yaml

# Verify structure
# - Port configuration correct
# - Labels match deployment
# - Service type appropriate
```

### 2. ConfigMap Generation Testing
```bash
# Test with default servers
helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/configmap.yaml

# Verify JSON validity
helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/configmap.yaml | \
  yq eval '.data."servers-config.json"' - | \
  jq .
```

### 3. Custom Server Addition
```bash
# Create test-values.yaml
cat > test-values.yaml <<EOF
mcpServers:
  custom-tool:
    transport: stdio
    command: /usr/local/bin/custom-mcp
    args: ["--mode", "server"]
    env:
      API_KEY: "\${CUSTOM_API_KEY}"
EOF

# Generate with custom values
helm template toolman ./toolman/charts/toolman/ \
  -f test-values.yaml \
  --show-only templates/configmap.yaml
```

### 4. Integration Testing
```bash
# Deploy and verify
helm install toolman-test ./toolman/charts/toolman/ -n test

# Check service
kubectl get svc toolman-test -n test -o yaml

# Verify ConfigMap
kubectl get configmap toolman-test-config -n test -o yaml

# Test connectivity
kubectl run curl --rm -it --image=curlimages/curl -n test -- \
  curl http://toolman-test:3000/health
```

## Best Practices

### 1. ConfigMap Management
- **Version Control**: Consider versioning ConfigMaps for rollback
- **Size Limits**: Keep under 1MB Kubernetes limit
- **Validation**: Always validate JSON before applying
- **Documentation**: Document each server's purpose

### 2. Service Configuration
- **Internal Only**: Use ClusterIP for security
- **Health Checks**: Ensure endpoints are monitored
- **Labels**: Consistent labeling for service mesh integration
- **Annotations**: Add monitoring/tracing annotations

### 3. Adding New MCP Servers
```yaml
# Template for adding new servers to values.yaml
new-mcp-server:
  transport: stdio|sse|http      # Choose transport type
  
  # For stdio transport:
  command: "executable-path"
  args: 
    - "arg1"
    - "arg2"
  env:
    ENV_VAR: "${ENV_VAR}"       # Use ${} for runtime substitution
  
  # For SSE/HTTP transport:
  url: "http://server:port/path"
  headers:
    Authorization: "Bearer ${TOKEN}"
```

## Security Considerations

1. **Secret Management**
   - Never hardcode sensitive values
   - Use `${VAR}` syntax for runtime substitution
   - Consider using Kubernetes Secrets

2. **Network Policies**
   - Service should only be accessible within cluster
   - Consider NetworkPolicies for additional restrictions

3. **ConfigMap Access**
   - Limit RBAC permissions to read ConfigMap
   - Audit access to server configurations

## Success Criteria
1. ✅ Service exposes Toolman on port 3000
2. ✅ ConfigMap generates valid JSON
3. ✅ All transport types supported
4. ✅ Custom servers can be added easily
5. ✅ No hardcoded server lists
6. ✅ Integration with deployment verified

## Related Tasks
- Task 1: Helm chart structure provides the foundation
- Task 2: Deployment uses these Service and ConfigMap
- Task 5: Docs Agent reads this ConfigMap for discovery
- Task 11: Tool validation checks against this ConfigMap
- Task 13: Final deployment of these components