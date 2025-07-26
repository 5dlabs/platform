# Acceptance Criteria: Task 2 - Analyze and Optimize Toolman Deployment

## Overview
This document defines the acceptance criteria for successfully analyzing the existing Toolman Kubernetes deployment manifest and preparing production-ready configurations for when it's deployed via the Helm chart.

## Current State
- **Deployment Location**: `toolman/charts/toolman/templates/deployment.yaml`
- **Status**: Exists as Helm template, NOT deployed
- **Dependencies**: Will be deployed as part of Task 1 (Helm chart installation)

## Analysis Requirements

### 1. Deployment Template Analysis
- [ ] **Template Review Complete**: Full analysis of deployment.yaml
- [ ] **Variable Mapping**: All Helm template variables documented
- [ ] **Default Values Identified**: Default configurations from values.yaml noted
- [ ] **Feature Assessment**: All deployment features cataloged

### 2. Container Configuration Review
- [ ] **Main Container**: Image, ports, env vars, volumes analyzed
- [ ] **Init Container**: Purpose and configuration understood
- [ ] **Sidecar Containers**: Docker-in-Docker requirements assessed
- [ ] **Security Contexts**: All security settings reviewed

### 3. Production Readiness Assessment
- [ ] **Resource Analysis**: Default CPU/memory settings evaluated
- [ ] **HA Capabilities**: Replica and affinity options reviewed
- [ ] **Health Checks**: Probe configurations assessed
- [ ] **Volume Mounts**: All volumes and mounts validated

### 4. Security Evaluation
- [ ] **User Context**: Non-root execution verified (UID 1001)
- [ ] **Capabilities**: Dropped capabilities confirmed
- [ ] **Privileges**: No unnecessary privileges identified
- [ ] **Secret Handling**: Secure secret references validated

## Configuration Deliverables

### 1. Production Values File
```yaml
# toolman-production-values.yaml must include:
- replicaCount: 3 (minimum for HA)
- image:
    tag: specific version (not latest)
    pullPolicy: IfNotPresent
- resources:
    requests: production-appropriate
    limits: prevent resource exhaustion
- affinity: pod anti-affinity rules
- podDisruptionBudget: for maintenance windows
```

### 2. Template Rendering Validation
- [ ] **Helm Template**: Renders without errors
- [ ] **Production Values**: Applied correctly in rendering
- [ ] **Resource Validation**: kubectl dry-run passes
- [ ] **No Hardcoding**: All values properly parameterized

### 3. Security Recommendations
- [ ] **Non-Root Verified**: Containers run as user 1001
- [ ] **Minimal Privileges**: Only required capabilities
- [ ] **Network Policies**: Compatible configurations
- [ ] **Secret Management**: No exposed credentials

## Test Scenarios

### Scenario 1: Template Validation
```bash
# Lint the chart
helm lint ./toolman/charts/toolman/

# Lint with production values
helm lint ./toolman/charts/toolman/ -f toolman-production-values.yaml

# Expected: No errors or warnings
```

### Scenario 2: Dry Run Testing
```bash
# Test deployment with production values
helm install toolman ./toolman/charts/toolman/ \
  --namespace orchestrator \
  --dry-run --debug \
  -f toolman-production-values.yaml

# Expected: Complete YAML output, no errors
```

### Scenario 3: Resource Rendering
```bash
# Render and extract deployment
helm template toolman ./toolman/charts/toolman/ \
  -f toolman-production-values.yaml | \
  yq eval 'select(.kind == "Deployment")' > deployment.yaml

# Validate rendered deployment
kubectl apply --dry-run=client -f deployment.yaml

# Expected: deployment.apps/toolman created (dry run)
```

### Scenario 4: Security Validation
```bash
# Extract security contexts from rendered deployment
yq eval '.spec.template.spec.securityContext' deployment.yaml
yq eval '.spec.template.spec.containers[0].securityContext' deployment.yaml

# Expected:
# - runAsUser: 1001
# - runAsNonRoot: true
# - No privileged: true
```

## Documentation Requirements

### 1. Deployment Analysis Report
- [ ] **Current Configuration**: Complete feature inventory
- [ ] **Template Structure**: How values map to deployment
- [ ] **Integration Points**: ConfigMap, Service, PVC connections
- [ ] **Recommendations**: Production optimization suggestions

### 2. Production Configuration Guide
- [ ] **Values Override**: Complete production values.yaml
- [ ] **Rationale**: Explanation for each override
- [ ] **Environment Variations**: Dev vs staging vs production
- [ ] **Scaling Guidelines**: When to adjust resources/replicas

### 3. Security Assessment
- [ ] **Current Security**: Existing security configurations
- [ ] **Compliance**: Meets security requirements
- [ ] **Risks**: Any identified security concerns
- [ ] **Mitigations**: Recommended security enhancements

### 4. Operational Readiness
- [ ] **Health Monitoring**: Probe configuration adequacy
- [ ] **Resource Sizing**: Recommendations for different loads
- [ ] **Update Strategy**: Rolling update configuration
- [ ] **Troubleshooting**: Common issues and solutions

## Definition of Done

✅ **Analysis Complete**
- Deployment template thoroughly reviewed
- All features and configurations documented
- Security assessment performed
- Production recommendations provided

✅ **Configuration Ready**
- Production values file created
- Template rendering validated
- Security requirements met
- Resource allocations optimized

✅ **Validation Passed**
- Helm lint successful
- Dry run tests pass
- Security scans clean
- No hardcoded values

✅ **Documentation Delivered**
- Analysis report complete
- Production guide written
- Security assessment documented
- Operational procedures defined

## Sign-off Criteria

- [ ] **Technical Review**: Analysis findings reviewed
- [ ] **Security Review**: Security configurations approved
- [ ] **Production Readiness**: Values file validated
- [ ] **Documentation Review**: All guides complete
- [ ] **Integration Verified**: Works with Helm chart

## Notes
- This analysis informs Task 1 deployment
- Focus on value overrides, not template modifications
- Document any blockers for production deployment
- Ensure recommendations are actionable