# Autonomous Agent Prompt: Update Platform CRDs for Tool Specifications

## Context
You need to update the CodeRun and DocsRun Custom Resource Definitions to support user-specified tool configurations. This enables users to explicitly state which MCP tools Claude should have access to when performing tasks.

## Your Mission
Modify the CRD schemas to add a `tools` field that accepts lists of local and remote tools, ensuring proper validation and maintaining backward compatibility.

## Detailed Instructions

### 1. Analyze Existing CRD Structure
```bash
# First, examine the current CRD definitions
# Look for the CRD files in the orchestrator
find orchestrator -name "*crd*.yaml" -type f

# Review the current schema structure
# Pay attention to:
# - API version
# - Existing spec properties
# - Validation patterns used
# - Status fields
```

### 2. Design the Tools Field Schema

Create the schema addition following these requirements:

**For Local Tools:**
- Fixed enum: ["filesystem", "git"]
- These are built into the platform
- Users can specify zero, one, or both
- Order doesn't matter

**For Remote Tools:**
- NO enum (tools are dynamic)
- Use pattern validation for DNS-compatible names
- Support wildcards (e.g., "kubernetes_*")
- Validated at runtime against ConfigMap

**Schema Structure:**
```yaml
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
      uniqueItems: true  # Prevent duplicates
    remote:
      type: array
      description: "Remote MCP tools accessible via toolman"
      items:
        type: string
        # DNS-compatible name pattern
        pattern: "^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$"
      default: []
      uniqueItems: true  # Prevent duplicates
  additionalProperties: false  # No extra fields allowed
```

### 3. Update CodeRun CRD

Modify the CodeRun CRD file:

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: coderuns.platform.5dlabs.com
  labels:
    app.kubernetes.io/name: orchestrator
    app.kubernetes.io/component: crd
spec:
  group: platform.5dlabs.com
  scope: Namespaced
  names:
    plural: coderuns
    singular: coderun
    kind: CodeRun
    shortNames:
    - cr
  versions:
  - name: v1
    served: true
    storage: true
    additionalPrinterColumns:
    - name: Service
      type: string
      jsonPath: .spec.service
    - name: Status
      type: string
      jsonPath: .status.phase
    - name: Age
      type: date
      jsonPath: .metadata.creationTimestamp
    schema:
      openAPIV3Schema:
        type: object
        required:
        - spec
        properties:
          apiVersion:
            type: string
          kind:
            type: string
          metadata:
            type: object
          spec:
            type: object
            required:
            - service
            properties:
              service:
                type: string
                description: "Target service name"
              task:
                type: string
                description: "Task identifier"
              branch:
                type: string
                description: "Git branch to use"
              # ADD THIS NEW FIELD
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
                    uniqueItems: true
                  remote:
                    type: array
                    description: "Remote MCP tools accessible via toolman"
                    items:
                      type: string
                      pattern: "^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$"
                    default: []
                    uniqueItems: true
                additionalProperties: false
          status:
            type: object
            properties:
              phase:
                type: string
                enum:
                - Pending
                - Running
                - Succeeded
                - Failed
```

### 4. Update DocsRun CRD

Apply the same schema addition to DocsRun CRD:
- Use identical tools field structure
- Maintain consistency between CRDs
- Same validation rules apply

### 5. Create Test Resources

**Test 1: Valid Tool Specification**
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-valid-tools
  namespace: default
spec:
  service: test-app
  task: "123"
  tools:
    local:
      - filesystem
      - git
    remote:
      - github
      - kubernetes
      - postgres
```

**Test 2: Empty Tools (Valid)**
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-empty-tools
spec:
  service: test-app
  tools:
    local: []
    remote: []
```

**Test 3: No Tools Field (Valid - Backward Compatible)**
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-no-tools
spec:
  service: test-app
  task: "456"
```

**Test 4: Invalid Local Tool (Should Fail)**
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-invalid-local
spec:
  service: test-app
  tools:
    local:
      - filesystem
      - invalid-tool  # This should fail validation
```

**Test 5: Invalid Remote Tool Pattern (Should Fail)**
```yaml
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-invalid-pattern
spec:
  service: test-app
  tools:
    remote:
      - "Invalid_Tool!"  # This should fail pattern validation
      - "spaces not allowed"
```

### 6. Apply and Validate CRDs

```bash
# Apply the updated CRDs
kubectl apply -f coderun-crd.yaml
kubectl apply -f docsrun-crd.yaml

# Verify the CRDs were updated
kubectl get crd coderuns.platform.5dlabs.com -o yaml | \
  yq eval '.spec.versions[0].schema.openAPIV3Schema.properties.spec.properties.tools' -

# The output should show the tools field schema
```

### 7. Test Validation

```bash
# Test each test case
kubectl apply -f test-valid-tools.yaml
# Expected: created successfully

kubectl apply -f test-invalid-local.yaml  
# Expected: validation error about invalid enum value

kubectl apply -f test-invalid-pattern.yaml
# Expected: validation error about pattern mismatch

# Check that existing resources still work
kubectl get coderuns
# Expected: Existing resources unaffected
```

### 8. Document the Changes

Create documentation covering:

**API Reference:**
```markdown
## CodeRun.spec.tools

Optional field to specify MCP tools for the run.

### Fields:
- `local` (array[string]): Local MCP tools to enable
  - Valid values: "filesystem", "git"
  - Default: []
  
- `remote` (array[string]): Remote MCP tools via toolman
  - Format: DNS-compatible names
  - Pattern: ^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$
  - Examples: "github", "kubernetes", "postgres-prod"
  - Default: []
```

**Migration Guide:**
```markdown
## CRD Update - Tool Specifications

### What Changed
- Added optional `tools` field to CodeRun and DocsRun specs
- Enables user-specified tool configurations

### Backward Compatibility
- Existing resources continue to work unchanged
- The tools field is optional
- No migration required

### Usage Examples
[Include the test examples]
```

### 9. Verify Integration Points

Ensure the CRD changes align with:
- Controller code that will read the tools field
- Validation logic that checks against ConfigMap
- Template generation that uses tool specifications

### 10. Final Validation Checklist

- [ ] CRDs apply without errors
- [ ] Schema validation works correctly
- [ ] Enum validation for local tools
- [ ] Pattern validation for remote tools
- [ ] Backward compatibility verified
- [ ] Test cases cover all scenarios
- [ ] Documentation complete
- [ ] Integration points identified

## Success Criteria

1. **CRD Updates Applied**: Both CodeRun and DocsRun CRDs updated
2. **Validation Working**: Invalid inputs rejected with clear errors
3. **Backward Compatible**: Existing resources unaffected
4. **Tests Passing**: All test scenarios behave as expected
5. **Documentation Complete**: API reference and migration guide ready

## Important Notes

- DO NOT add enum to remote tools - they must remain dynamic
- Ensure pattern allows DNS-compatible names and wildcards
- Keep the field optional for backward compatibility
- Use consistent schema between CodeRun and DocsRun
- Consider future extensibility in the design

Proceed with implementing these CRD updates, testing thoroughly, and documenting the changes for platform users.