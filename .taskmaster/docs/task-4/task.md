# Task 4: Update Platform CRDs for Tool Specifications

## Overview
Modify the CodeRun and DocsRun Custom Resource Definitions (CRDs) to accept tool specifications, enabling users to specify which local and remote MCP tools Claude should use for specific tasks. This is a foundational change that enables the core user experience of tool selection.

## Context
Currently, the platform assigns tools based on predefined configuration levels (minimal/default/advanced). This task adds the capability for users to explicitly specify tools per task, supporting interactions like:
- "Start task 6 with filesystem and GitHub tools"
- "Run docs generation with read-only filesystem and web search"
- "Deploy using Kubernetes, Helm, and git tools"

## Objectives
1. Add `tools` field to CodeRun and DocsRun CRD schemas
2. Implement proper validation for tool specifications
3. Maintain backward compatibility (tools field is optional)
4. Follow Kubernetes CRD best practices
5. Enable user control over tool selection

## Technical Design

### CRD Schema Structure
```yaml
tools:
  type: object
  description: "Optional tool configuration for this run"
  properties:
    local:
      type: array
      description: "Local MCP tools to enable"
      items:
        type: string
        enum: ["filesystem", "git"]  # Fixed set of local tools
    remote:
      type: array  
      description: "Remote MCP tools via toolman"
      items:
        type: string
        # NO enum here - remote tools are dynamic!
```

### Design Principles
1. **Local Tools**: Fixed enum since they're built into the platform
2. **Remote Tools**: No enum - discovered dynamically from ConfigMap
3. **Optional Field**: Maintains backward compatibility
4. **Clear Separation**: Local vs remote tool categories

## Implementation Guide

### Step 1: CodeRun CRD Update
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: coderuns.platform.5dlabs.com
spec:
  group: platform.5dlabs.com
  versions:
  - name: v1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          apiVersion:
            type: string
          kind:
            type: string
          metadata:
            type: object
          spec:
            type: object
            properties:
              # ... existing properties ...
              service:
                type: string
                description: "Target service name"
              task:
                type: string
                description: "Task identifier"
              branch:
                type: string
                description: "Git branch to use"
              
              # NEW: Tool specification field
              tools:
                type: object
                description: "Tool configuration for this run (optional)"
                properties:
                  local:
                    type: array
                    description: "Local MCP tools to enable"
                    items:
                      type: string
                      enum: 
                        - filesystem
                        - git
                    default: []
                  remote:
                    type: array
                    description: "Remote MCP tools accessible via toolman"
                    items:
                      type: string
                      pattern: "^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$"
                    default: []
                additionalProperties: false
          status:
            type: object
            # ... existing status fields ...
```

### Step 2: DocsRun CRD Update
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: docsruns.platform.5dlabs.com
spec:
  group: platform.5dlabs.com
  versions:
  - name: v1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          apiVersion:
            type: string
          kind:
            type: string
          metadata:
            type: object
          spec:
            type: object
            properties:
              # ... existing properties ...
              service:
                type: string
                description: "Target service name"
              
              # NEW: Tool specification field (same as CodeRun)
              tools:
                type: object
                description: "Tool configuration for this run (optional)"
                properties:
                  local:
                    type: array
                    description: "Local MCP tools to enable"
                    items:
                      type: string
                      enum: 
                        - filesystem
                        - git
                    default: []
                  remote:
                    type: array
                    description: "Remote MCP tools accessible via toolman"
                    items:
                      type: string
                      pattern: "^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$"
                    default: []
                additionalProperties: false
          status:
            type: object
            # ... existing status fields ...
```

### Step 3: Validation Patterns

#### Local Tool Validation
- Enum ensures only valid local tools
- Empty array means no local tools
- Duplicates should be handled gracefully

#### Remote Tool Validation
- Pattern ensures DNS-compatible names
- No enum - tools validated at runtime against ConfigMap
- Supports wildcards (e.g., "kubernetes_*")
- Empty array means no remote tools

### Step 4: Usage Examples

#### Example 1: Basic Tool Specification
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: implement-feature-123
spec:
  service: my-app
  task: "123"
  branch: feature/implement-123
  tools:
    local:
      - filesystem
      - git
    remote:
      - github
      - postgres
```

#### Example 2: Read-Only Filesystem
```yaml
apiVersion: platform.5dlabs.com/v1
kind: DocsRun
metadata:
  name: generate-docs
spec:
  service: my-app
  tools:
    local:
      - filesystem  # Docs agent could enforce read-only
    remote:
      - brave-search
```

#### Example 3: Infrastructure Tools
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: deploy-service
spec:
  service: my-app
  task: "deploy"
  tools:
    local:
      - git
    remote:
      - kubernetes
      - helm
      - terraform
```

## Testing Strategy

### 1. CRD Application
```bash
# Apply updated CRDs
kubectl apply -f coderun-crd.yaml
kubectl apply -f docsrun-crd.yaml

# Verify CRDs updated
kubectl get crd coderuns.platform.5dlabs.com -o yaml | \
  yq eval '.spec.versions[0].schema.openAPIV3Schema.properties.spec.properties.tools' -
```

### 2. Validation Testing
```bash
# Test valid specification
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-valid-tools
spec:
  service: test
  tools:
    local: ["filesystem", "git"]
    remote: ["github", "kubernetes"]
EOF

# Test invalid local tool
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-invalid-local
spec:
  service: test
  tools:
    local: ["invalid-tool"]
EOF
# Expected: Validation error

# Test invalid remote tool pattern
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-invalid-remote
spec:
  service: test
  tools:
    remote: ["Invalid_Tool!"]
EOF
# Expected: Pattern validation error
```

### 3. Backward Compatibility
```bash
# Test without tools field
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-no-tools
spec:
  service: test
  task: "1"
EOF
# Expected: Success (tools field optional)
```

### 4. Edge Cases
```bash
# Empty arrays
tools:
  local: []
  remote: []

# Duplicates
tools:
  local: ["filesystem", "filesystem"]
  
# Wildcards
tools:
  remote: ["kubernetes_*", "github_*"]
```

## Migration Considerations

### 1. Existing Resources
- Existing CodeRun/DocsRun resources continue to work
- No migration required - field is optional
- Default behavior unchanged

### 2. Controller Updates
- Controllers must handle both cases:
  - With tools: Use specified configuration
  - Without tools: Use default/generated configuration

### 3. Version Strategy
- Keep v1 API version for compatibility
- Field addition is backward compatible
- No need for conversion webhooks

## Best Practices

### 1. CRD Design
- Use OpenAPI v3 schema for validation
- Provide clear descriptions for each field
- Set appropriate defaults (empty arrays)
- Use patterns for format validation

### 2. Naming Conventions
- Field names: camelCase (tools, not tool_list)
- Array items: plural (local, remote)
- Descriptions: Clear and concise

### 3. Validation Rules
- Local tools: Strict enum validation
- Remote tools: Pattern validation only
- Runtime validation happens in controller

## Security Considerations

1. **Tool Access Control**
   - CRD validates format, not permissions
   - Controller must verify user can access requested tools
   - Consider RBAC for tool restrictions

2. **Input Validation**
   - Pattern prevents injection attacks
   - Enum prevents invalid local tools
   - Controller validates against ConfigMap

## Success Criteria
1. ✅ CRDs accept tool specifications
2. ✅ Validation works correctly
3. ✅ Backward compatibility maintained
4. ✅ Clear error messages for invalid input
5. ✅ Documentation complete

## Related Tasks
- Task 5: Docs Agent reads tool specs when generating config
- Task 7: Code Agent uses tool specs from CRD
- Task 11: Controller validates tools against ConfigMap
- Task 12: Orchestrator passes tools to templates