# Acceptance Criteria: Task 4 - Update Platform CRDs for Tool Specifications

## Overview
This document defines the acceptance criteria for successfully updating the CodeRun and DocsRun CRDs to support user-specified tool configurations.

## Core Requirements

### 1. Schema Updates
- [ ] **Tools Field Added**: New optional `tools` field in spec
- [ ] **Local Tools Array**: Supports filesystem and git selection
- [ ] **Remote Tools Array**: Supports dynamic tool names
- [ ] **Proper Validation**: Schema enforces correct formats
- [ ] **Backward Compatible**: Existing resources unaffected

### 2. Validation Rules
- [ ] **Local Tool Enum**: Only "filesystem" and "git" allowed
- [ ] **Remote Tool Pattern**: DNS-compatible names only
- [ ] **Optional Field**: Resources work without tools field
- [ ] **Unique Items**: No duplicate tools in arrays
- [ ] **No Extra Properties**: additionalProperties: false

### 3. API Consistency
- [ ] **Same Schema**: CodeRun and DocsRun use identical tools schema
- [ ] **Clear Descriptions**: Each field documented in schema
- [ ] **Appropriate Defaults**: Empty arrays as defaults
- [ ] **Version Compatibility**: v1 API version maintained

## Technical Specifications

### 1. Tools Field Schema
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
```

### 2. Valid Examples
```yaml
# Full specification
tools:
  local: ["filesystem", "git"]
  remote: ["github", "kubernetes", "postgres"]

# Partial specification
tools:
  local: ["filesystem"]
  remote: []

# Empty specification
tools:
  local: []
  remote: []

# No specification (field omitted)
# (tools field not present)
```

### 3. Invalid Examples
```yaml
# Invalid local tool
tools:
  local: ["invalid-tool"]  # Fails enum validation

# Invalid remote tool pattern
tools:
  remote: ["Invalid_Tool"]  # Fails pattern (uppercase)
  
# Extra properties
tools:
  local: ["git"]
  remote: ["github"]
  extra: "field"  # Fails additionalProperties
```

## Test Cases

### Test Case 1: CRD Application
```bash
# Apply updated CRDs
kubectl apply -f coderun-crd.yaml
kubectl apply -f docsrun-crd.yaml

# Verify successful application
kubectl get crd coderuns.platform.5dlabs.com
kubectl get crd docsruns.platform.5dlabs.com

# Expected: CRDs updated without errors
```

### Test Case 2: Valid Tool Specifications
```bash
# Test various valid combinations
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-all-tools
spec:
  service: test
  tools:
    local: ["filesystem", "git"]
    remote: ["github", "kubernetes", "postgres"]
EOF

# Expected: Created successfully
```

### Test Case 3: Enum Validation
```bash
# Test invalid local tool
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-bad-local
spec:
  service: test
  tools:
    local: ["filesystem", "docker"]  # docker not in enum
EOF

# Expected: Validation error mentioning enum constraint
```

### Test Case 4: Pattern Validation
```bash
# Test invalid remote tool patterns
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-bad-pattern
spec:
  service: test
  tools:
    remote: 
      - "UPPERCASE"      # Fails: uppercase
      - "under_score"    # Fails: underscore
      - "special-char!"  # Fails: special char
      - "space name"     # Fails: space
EOF

# Expected: Pattern validation errors
```

### Test Case 5: Backward Compatibility
```bash
# Create resource without tools field
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-no-tools
spec:
  service: test
  task: "123"
  branch: main
EOF

# Expected: Created successfully (tools optional)
```

### Test Case 6: Uniqueness Validation
```bash
# Test duplicate prevention
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-duplicates
spec:
  service: test
  tools:
    local: ["filesystem", "filesystem"]  # Duplicate
    remote: ["github", "github"]         # Duplicate
EOF

# Expected: Validation error for non-unique items
```

### Test Case 7: Empty Arrays
```bash
# Test empty tool arrays
cat <<EOF | kubectl apply -f -
apiVersion: platform.5dlabs.com/v1
kind: CodeRun
metadata:
  name: test-empty
spec:
  service: test
  tools:
    local: []
    remote: []
EOF

# Expected: Created successfully
```

### Test Case 8: Schema Extraction
```bash
# Verify schema was properly added
kubectl get crd coderuns.platform.5dlabs.com -o json | \
  jq '.spec.versions[0].schema.openAPIV3Schema.properties.spec.properties.tools'

# Expected: Full tools schema with local/remote properties
```

## Validation Checklist

### Schema Validation
- [ ] **CRD Syntax**: Valid Kubernetes CRD syntax
- [ ] **OpenAPI v3**: Uses openAPIV3Schema
- [ ] **Required Fields**: Proper required field specifications
- [ ] **Type Definitions**: All types correctly defined
- [ ] **Descriptions**: User-friendly descriptions

### Functional Validation
- [ ] **Apply Success**: CRDs apply without errors
- [ ] **Create Resources**: Can create resources with tools
- [ ] **Validation Works**: Invalid inputs rejected
- [ ] **Error Messages**: Clear validation error messages
- [ ] **Defaults Applied**: Empty arrays when not specified

### Integration Validation
- [ ] **Controller Compatible**: Schema works with controllers
- [ ] **No Breaking Changes**: Existing resources still work
- [ ] **API Version**: v1 maintained for compatibility
- [ ] **Printer Columns**: Status displays correctly

## Documentation Requirements

### 1. API Documentation
- [ ] **Field Reference**: Complete tools field documentation
- [ ] **Examples**: Valid and invalid examples provided
- [ ] **Pattern Explanation**: Remote tool pattern explained
- [ ] **Enum Values**: Local tool options listed

### 2. User Guide
- [ ] **How to Specify Tools**: Step-by-step guide
- [ ] **Common Patterns**: Typical tool combinations
- [ ] **Troubleshooting**: Common errors and fixes
- [ ] **Best Practices**: Recommendations for tool selection

### 3. Migration Guide
- [ ] **What Changed**: Clear description of changes
- [ ] **Compatibility**: Backward compatibility explained
- [ ] **No Action Required**: Existing resources continue working
- [ ] **New Features**: How to use tool specifications

## Performance Criteria

### 1. Validation Performance
- [ ] **Fast Validation**: < 100ms for tool validation
- [ ] **No Regex DoS**: Pattern cannot cause ReDoS
- [ ] **Efficient Matching**: Linear time complexity

### 2. Resource Impact
- [ ] **Minimal Overhead**: Small schema addition
- [ ] **No Memory Bloat**: Reasonable array limits
- [ ] **Quick Updates**: CRD updates apply quickly

## Security Requirements

### 1. Input Validation
- [ ] **Pattern Safety**: No injection via tool names
- [ ] **Array Limits**: Reasonable maximum array size
- [ ] **No Code Execution**: Pure data validation only

### 2. Access Control
- [ ] **RBAC Compatible**: Works with standard RBAC
- [ ] **No Privilege Escalation**: Tool specs don't grant access
- [ ] **Audit Trail**: Changes trackable via Kubernetes audit

## Definition of Done

✅ **CRD Updates Complete**
- Both CodeRun and DocsRun CRDs updated
- Tools field properly integrated
- Validation rules implemented
- Schema follows best practices

✅ **Testing Verified**
- All test cases pass
- Validation works as expected
- Backward compatibility confirmed
- Error messages are helpful

✅ **Documentation Delivered**
- API reference complete
- User guide written
- Migration notes provided
- Examples comprehensive

✅ **Integration Ready**
- Controllers can read tools field
- Templates can access specifications
- Validation logic can check ConfigMap
- No breaking changes

## Sign-off Requirements

- [ ] **Schema Review**: CRD schema approved by platform team
- [ ] **Security Review**: Validation patterns reviewed for safety
- [ ] **API Review**: Field naming and structure approved
- [ ] **Documentation Review**: All guides complete and clear
- [ ] **Integration Test**: End-to-end flow tested

## Notes
- This change enables the core user experience of tool selection
- The remote tools list must remain dynamic (no enum)
- Local tools enum may be extended in future versions
- Consider future extensibility in the design