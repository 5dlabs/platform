# Acceptance Criteria: Task 3 - Create Toolman Service and ConfigMap Templates

## Overview
This document defines the acceptance criteria for successfully reviewing and customizing the Toolman Service and ConfigMap templates. These components are critical for exposing Toolman and defining available MCP servers.

## Core Requirements

### 1. Service Template
- [ ] **Port Configuration**: Exposes port 3000
- [ ] **Service Type**: ClusterIP for internal access only
- [ ] **Selector Labels**: Match deployment pod labels
- [ ] **Named Ports**: Port named "http" for clarity
- [ ] **Annotations Support**: Template supports custom annotations
- [ ] **Helm Integration**: Uses proper Helm template functions

### 2. ConfigMap Template
- [ ] **JSON Generation**: Produces valid servers-config.json
- [ ] **Transport Types**: Supports stdio, SSE, and HTTP
- [ ] **Environment Variables**: Uses ${} syntax for substitution
- [ ] **No Hardcoding**: Server list comes from values.yaml
- [ ] **Proper Formatting**: JSON is correctly formatted
- [ ] **Size Compliance**: Output under 1MB limit

### 3. MCP Server Configurations
- [ ] **STDIO Servers**: Documented and tested
- [ ] **SSE Servers**: Documented and tested
- [ ] **HTTP Servers**: Documented and tested
- [ ] **Environment Handling**: Secure variable substitution
- [ ] **Extensibility**: Easy to add new servers

## Technical Specifications

### 1. Service Specification
```yaml
# Expected Service output:
apiVersion: v1
kind: Service
metadata:
  name: toolman
  namespace: orchestrator
  labels:
    app.kubernetes.io/name: toolman
    app.kubernetes.io/instance: toolman
spec:
  type: ClusterIP
  ports:
  - port: 3000
    targetPort: http
    protocol: TCP
    name: http
  selector:
    app.kubernetes.io/name: toolman
    app.kubernetes.io/instance: toolman
```

### 2. ConfigMap Specification
```yaml
# Expected ConfigMap output:
apiVersion: v1
kind: ConfigMap
metadata:
  name: toolman-config
  namespace: orchestrator
data:
  servers-config.json: |
    {
      "servers": {
        "github": {
          "transport": "stdio",
          "command": "npx",
          "args": ["-y", "@modelcontextprotocol/server-github"],
          "env": {
            "GITHUB_TOKEN": "${GITHUB_TOKEN}"
          }
        },
        // Additional servers...
      }
    }
```

### 3. Transport Type Formats

#### STDIO Transport
```json
{
  "transport": "stdio",
  "command": "<executable>",
  "args": ["<arg1>", "<arg2>"],
  "env": {
    "KEY": "${VALUE}"
  }
}
```

#### SSE Transport
```json
{
  "transport": "sse",
  "url": "<sse-endpoint>",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### HTTP Transport
```json
{
  "transport": "http",
  "url": "<http-endpoint>",
  "headers": {
    "Content-Type": "application/json"
  }
}
```

## Test Cases

### Test Case 1: Service Template Rendering
```bash
# Render service template
helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/service.yaml

# Verify output contains:
# - port: 3000
# - type: ClusterIP
# - correct selectors
```

### Test Case 2: ConfigMap JSON Validation
```bash
# Extract and validate JSON
helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/configmap.yaml | \
  yq eval '.data."servers-config.json"' - | \
  jq .

# Expected: Valid JSON with servers object
```

### Test Case 3: Custom Server Addition
```bash
# Add custom server to values
cat > custom-values.yaml <<EOF
mcpServers:
  custom-tool:
    transport: stdio
    command: /usr/bin/custom-mcp
    args: ["--server"]
    env:
      API_KEY: "\${CUSTOM_API_KEY}"
EOF

# Verify custom server appears in ConfigMap
helm template toolman ./toolman/charts/toolman/ \
  -f custom-values.yaml \
  --show-only templates/configmap.yaml | \
  grep "custom-tool" | wc -l

# Expected: 1 (server added successfully)
```

### Test Case 4: Service Connectivity
```bash
# Deploy and test
helm install test-toolman ./toolman/charts/toolman/ -n test

# Verify service endpoints
kubectl get endpoints test-toolman -n test

# Test connectivity
kubectl run curl --rm -it --image=curlimages/curl -n test -- \
  curl http://test-toolman:3000/health

# Expected: HTTP 200 response
```

### Test Case 5: All Transport Types
```bash
# Create comprehensive test
cat > all-transports.yaml <<EOF
mcpServers:
  stdio-test:
    transport: stdio
    command: echo
    args: ["test"]
  sse-test:
    transport: sse
    url: http://sse:8080/events
  http-test:
    transport: http
    url: http://api:9090
EOF

# Verify all transport types render correctly
helm template toolman ./toolman/charts/toolman/ \
  -f all-transports.yaml \
  --show-only templates/configmap.yaml | \
  yq eval '.data."servers-config.json"' - | \
  jq '.servers | keys'

# Expected: ["http-test", "sse-test", "stdio-test"]
```

### Test Case 6: Label Matching
```bash
# Verify service selectors match deployment labels
SERVICE_LABELS=$(helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/service.yaml | \
  yq eval '.spec.selector' -)

DEPLOYMENT_LABELS=$(helm template toolman ./toolman/charts/toolman/ \
  --show-only templates/deployment.yaml | \
  yq eval '.spec.template.metadata.labels' -)

# Compare labels (should match)
```

## Validation Checklist

### Service Validation
- [ ] **Renders Successfully**: No template errors
- [ ] **Correct Port**: Port 3000 exposed
- [ ] **Internal Only**: ClusterIP type
- [ ] **Labels Match**: Selectors align with deployment
- [ ] **Annotations Work**: Custom annotations applied
- [ ] **DNS Compatible**: Name follows Kubernetes conventions

### ConfigMap Validation
- [ ] **Valid JSON**: No syntax errors
- [ ] **All Servers Present**: Default servers included
- [ ] **Extensible**: New servers easily added
- [ ] **Environment Variables**: ${} syntax preserved
- [ ] **No Secrets**: No hardcoded sensitive data
- [ ] **Size Limit**: Under 1MB total size

### Integration Validation
- [ ] **Service Discovery**: Finds deployment pods
- [ ] **ConfigMap Mounting**: Mounts in deployment
- [ ] **Toolman Startup**: Reads configuration successfully
- [ ] **MCP Access**: Servers accessible via proxy

## Documentation Requirements

### 1. Service Configuration Guide
- [ ] **Overview**: Purpose and architecture
- [ ] **Customization**: How to modify settings
- [ ] **Annotations**: Available options documented
- [ ] **Troubleshooting**: Common issues and fixes

### 2. MCP Server Addition Guide
- [ ] **Transport Types**: All three documented
- [ ] **Examples**: Working examples for each type
- [ ] **Environment Variables**: Substitution patterns
- [ ] **Testing Process**: How to verify new servers

### 3. ConfigMap Management
- [ ] **Update Process**: How to modify servers
- [ ] **Validation**: Pre-deployment checks
- [ ] **Rollback**: Recovery procedures
- [ ] **Best Practices**: Organization and naming

## Performance Criteria

### 1. Template Rendering
- [ ] **Speed**: Renders in < 1 second
- [ ] **Memory**: No excessive memory use
- [ ] **Complexity**: O(n) with server count

### 2. ConfigMap Size
- [ ] **Current Size**: Document baseline
- [ ] **Growth Rate**: Estimate per server
- [ ] **Maximum Servers**: Calculate limit

### 3. Service Performance
- [ ] **Connection Time**: < 100ms internal
- [ ] **Throughput**: Supports expected load
- [ ] **Stability**: No connection drops

## Security Requirements

### 1. Service Security
- [ ] **Internal Only**: No external exposure
- [ ] **Network Policies**: Compatible with policies
- [ ] **TLS Ready**: Can add TLS if needed
- [ ] **No Privileged Ports**: Uses port > 1024

### 2. ConfigMap Security
- [ ] **No Secrets**: Uses ${} substitution
- [ ] **RBAC Ready**: Appropriate permissions
- [ ] **Audit Trail**: Changes trackable
- [ ] **Validation**: Prevents injection

## Definition of Done

✅ **Templates Reviewed**
- Service template analyzed and understood
- ConfigMap template JSON generation verified
- All transport types documented
- Integration points identified

✅ **Testing Complete**
- All test cases passing
- Custom servers successfully added
- Service connectivity verified
- ConfigMap validation successful

✅ **Documentation Delivered**
- Service configuration guide complete
- MCP server addition guide ready
- ConfigMap management documented
- Troubleshooting procedures written

✅ **Production Ready**
- Security requirements met
- Performance validated
- Integration tested
- No hardcoded values

## Sign-off Requirements

- [ ] **Technical Review**: Templates approved by platform team
- [ ] **Security Review**: No security concerns identified
- [ ] **Documentation Review**: Guides clear and complete
- [ ] **Integration Test**: End-to-end flow verified
- [ ] **Performance Test**: Meets performance criteria

## Notes
- This ConfigMap is the single source of truth for tool discovery
- Changes here affect the entire platform
- Maintain backward compatibility when possible
- Consider ConfigMap size limits for scale