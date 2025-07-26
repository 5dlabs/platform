# Autonomous Agent Prompt: Configure Toolman Service and ConfigMap

## Context
You are tasked with reviewing and customizing the Toolman Service and ConfigMap templates. These are critical components that expose Toolman's HTTP proxy endpoint and define all available MCP servers. The ConfigMap serves as the single source of truth for the entire platform's tool discovery system.

## Your Mission
Ensure the Service and ConfigMap templates are production-ready, well-documented, and support easy addition of new MCP servers without code changes.

## Detailed Instructions

### 1. Service Template Analysis
```bash
# Navigate to templates directory
cd toolman/charts/toolman/templates/

# Examine the service template
cat service.yaml

# Key areas to validate:
# - API version compatibility
# - Service type (should be ClusterIP)
# - Port configuration (must be 3000)
# - Selector labels matching deployment
# - Annotations support
```

**Service Requirements Checklist:**
- [ ] Port 3000 exposed
- [ ] Named port as "http"
- [ ] ClusterIP type for internal access
- [ ] Labels use Helm helpers
- [ ] Supports custom annotations
- [ ] Selector matches deployment pods

### 2. ConfigMap Template Deep Dive
```bash
# Examine the ConfigMap template
cat configmap.yaml

# Understand the JSON generation:
# - How values.yaml maps to JSON
# - Template functions used
# - Formatting and indentation
```

**ConfigMap Analysis Points:**
- JSON generation mechanism
- Variable substitution patterns
- Support for all transport types
- Proper escaping and formatting
- Size considerations

### 3. MCP Server Configuration Documentation

Create comprehensive documentation for each transport type:

#### STDIO Transport Servers
```yaml
# Document this pattern:
server-name:
  transport: stdio
  command: "executable"     # The MCP server executable
  args:                     # Command line arguments
    - "--arg1"
    - "value1"
  env:                      # Environment variables
    API_KEY: "${API_KEY}"   # ${} for runtime substitution
    
# Real example:
github:
  transport: stdio
  command: npx
  args: ["-y", "@modelcontextprotocol/server-github"]
  env:
    GITHUB_TOKEN: "${GITHUB_TOKEN}"
```

#### SSE Transport Servers
```yaml
# Document this pattern:
server-name:
  transport: sse
  url: "http://host:port/sse"  # SSE endpoint
  headers:                      # Optional headers
    Authorization: "Bearer ${TOKEN}"
```

#### HTTP Transport Servers
```yaml
# Document this pattern:
server-name:
  transport: http
  url: "http://host:port"      # HTTP endpoint
  headers:                      # Optional headers
    Content-Type: "application/json"
```

### 4. Values.yaml Review
```bash
# Review existing server configurations
cat ../values.yaml | yq eval '.mcpServers' -

# Document each existing server:
# - Purpose
# - Required environment variables
# - Special considerations
```

### 5. ConfigMap Generation Testing

**Test 1: Default Configuration**
```bash
# Generate ConfigMap with default values
helm template toolman ../ --show-only templates/configmap.yaml

# Extract and validate JSON
helm template toolman ../ --show-only templates/configmap.yaml | \
  yq eval '.data."servers-config.json"' - > servers.json

# Validate JSON structure
jq . servers.json
```

**Test 2: Custom Server Addition**
```bash
# Create custom-servers.yaml
cat > custom-servers.yaml <<EOF
mcpServers:
  # Existing servers inherited
  github:
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "\${GITHUB_TOKEN}"
  
  # New custom server
  my-custom-tool:
    transport: stdio
    command: /opt/custom-mcp
    args: ["--server-mode"]
    env:
      CUSTOM_CONFIG: "\${CUSTOM_CONFIG}"
EOF

# Test generation with custom values
helm template toolman ../ -f custom-servers.yaml \
  --show-only templates/configmap.yaml
```

**Test 3: All Transport Types**
```bash
# Create all-transports.yaml testing each type
cat > all-transports.yaml <<EOF
mcpServers:
  # STDIO example
  stdio-test:
    transport: stdio
    command: echo
    args: ["test"]
    
  # SSE example  
  sse-test:
    transport: sse
    url: http://sse-server:8080/events
    headers:
      X-API-Key: "\${SSE_KEY}"
      
  # HTTP example
  http-test:
    transport: http
    url: http://http-server:9090
    headers:
      Authorization: "Bearer \${HTTP_TOKEN}"
EOF

# Validate all transport types work
helm template toolman ../ -f all-transports.yaml \
  --show-only templates/configmap.yaml | \
  yq eval '.data."servers-config.json"' - | jq .
```

### 6. Service Connectivity Testing
```bash
# Deploy test instance
helm install toolman-svc-test ../ -n test --create-namespace

# Verify service created
kubectl get svc -n test toolman-svc-test

# Check endpoints
kubectl get endpoints -n test toolman-svc-test

# Test internal connectivity
kubectl run test-curl --rm -it --image=curlimages/curl -n test -- \
  curl -v http://toolman-svc-test:3000/health

# Verify DNS resolution
kubectl run test-dns --rm -it --image=busybox -n test -- \
  nslookup toolman-svc-test.test.svc.cluster.local
```

### 7. Production Configuration Guide

Create a production configuration guide covering:

**Service Customization:**
```yaml
service:
  type: ClusterIP          # Internal only
  port: 3000               # Standard port
  annotations:
    # Monitoring
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
    prometheus.io/path: "/metrics"
    
    # Service mesh
    service.beta.kubernetes.io/aws-load-balancer-internal: "true"
```

**ConfigMap Best Practices:**
```yaml
mcpServers:
  # Group by function
  ## Development Tools
  github:
    # ...
    
  ## Search Tools  
  brave-search:
    # ...
    
  ## Infrastructure Tools
  kubernetes:
    # ...
    
  ## Database Tools
  postgres:
    # ...
```

### 8. Validation Checklist

**Service Validation:**
- [ ] Renders valid Kubernetes Service
- [ ] Port 3000 properly exposed
- [ ] Labels match deployment selectors
- [ ] Supports monitoring annotations
- [ ] DNS name predictable

**ConfigMap Validation:**
- [ ] Generates valid JSON
- [ ] All transport types supported
- [ ] Environment variable substitution works
- [ ] No hardcoded secrets
- [ ] Size under 1MB limit

**Integration Validation:**
- [ ] Service discovers deployment pods
- [ ] ConfigMap mounts in deployment
- [ ] Toolman reads configuration
- [ ] MCP servers accessible

### 9. Documentation Deliverables

1. **Service Configuration Guide**
   - How to customize service settings
   - Annotation options
   - Network policy considerations
   - Troubleshooting connectivity

2. **MCP Server Addition Guide**
   - Step-by-step for each transport type
   - Environment variable patterns
   - Testing new servers
   - Common pitfalls

3. **ConfigMap Management Guide**
   - Updating server configurations
   - Version control strategies
   - Rollback procedures
   - Size optimization

4. **Integration Testing Guide**
   - Verifying service connectivity
   - Testing ConfigMap changes
   - End-to-end validation
   - Monitoring setup

## Success Criteria

1. **Service Functionality**
   - Exposes port 3000 successfully
   - Pods are discovered automatically
   - Internal DNS resolution works
   - No external exposure

2. **ConfigMap Correctness**
   - Valid JSON generation
   - All servers properly formatted
   - Environment variables use ${} syntax
   - No syntax errors

3. **Extensibility**
   - New servers added via values.yaml only
   - No template modifications needed
   - All transport types supported
   - Clear documentation

4. **Production Readiness**
   - Security best practices followed
   - Monitoring hooks included
   - Update procedures documented
   - Troubleshooting guide complete

Proceed with the analysis and provide detailed findings for each component. Remember that this ConfigMap is the foundation of the entire tool discovery system - accuracy and clarity are paramount.