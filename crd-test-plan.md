# CRD Test Plan: CodeRun & DocsRun Validation

## 🎯 Test Objectives
- Verify both CRDs are properly deployed and accessible
- Confirm controller correctly processes CodeRun and DocsRun resources
- Validate Job creation with correct configurations
- Test status updates and condition reporting
- Ensure proper resource cleanup

## 📋 Pre-Test Verification

```bash
# Verify CRDs exist
kubectl get crd | grep orchestrator

# Check controller is running
kubectl get pods -n orchestrator -l app=orchestrator

# Monitor controller logs (keep open in separate terminal)
kubectl logs -n orchestrator deployment/orchestrator -f
```

## 🧪 Test 1: CodeRun CRD

### Test Resource: `test-coderun.yaml`
```yaml
apiVersion: orchestrator.platform/v1
kind: CodeRun
metadata:
  name: test-coderun-001
  namespace: orchestrator
spec:
  # Required fields
  taskId: 12345
  service: "test-service"
  repositoryUrl: "https://github.com/5dlabs/platform"
  platformRepositoryUrl: "https://github.com/5dlabs/platform"
  branch: "feature/test-branch"
  workingDirectory: "orchestrator"
  model: "claude-3.5-sonnet"
  githubUser: "test-user"

  # Optional fields (testing defaults and custom values)
  localTools: "bash,git,kubectl"
  remoteTools: "github-api,jira"
  toolConfig: "debug=true,timeout=3600"
  contextVersion: 1  # Should default to 1
  promptModification: "Focus on testing the new CRD functionality"
  promptMode: "append"  # Should default to "append"
```

### Expected Behavior:
1. **Resource Creation**: `kubectl apply` succeeds
2. **Controller Processing**: Logs show reconciliation starting
3. **Job Creation**: Job named like `test-service-task12345-attempt1`
4. **ConfigMap Creation**: ConfigMap with templates created
5. **Status Updates**: Phase transitions through "Pending" → "Running"
6. **Resource Cleanup**: Job cleaned up after completion/timeout

## 🧪 Test 2: DocsRun CRD

### Test Resource: `test-docsrun.yaml`
```yaml
apiVersion: orchestrator.platform/v1
kind: DocsRun
metadata:
  name: test-docsrun-001
  namespace: orchestrator
spec:
  # All required fields
  repositoryUrl: "https://github.com/5dlabs/platform"
  workingDirectory: "_projects/simple-api"
  sourceBranch: "feature/example-project-and-cli"
  model: "claude-3-5-sonnet-20241022"
  githubUser: "pm0-5dlabs"
```

### Expected Behavior:
1. **Resource Creation**: `kubectl apply` succeeds
2. **Controller Processing**: Logs show reconciliation starting
3. **Job Creation**: Job named like `test-user-platform-docs`
4. **ConfigMap Creation**: ConfigMap with docs templates created
5. **Status Updates**: Phase transitions appropriately
6. **Resource Cleanup**: Job cleaned up after completion/timeout

## 🔍 Test Execution Steps

### Step 1: Create Test Resources
```bash
# Apply CodeRun test
kubectl apply -f test-coderun.yaml

# Apply DocsRun test
kubectl apply -f test-docsrun.yaml
```

### Step 2: Monitor Resources
```bash
# Check CRD instances
kubectl get coderuns -n orchestrator
kubectl get docsruns -n orchestrator

# Check created Jobs
kubectl get jobs -n orchestrator

# Check ConfigMaps
kubectl get configmaps -n orchestrator

# Detailed status
kubectl describe coderun test-coderun-001 -n orchestrator
kubectl describe docsrun test-docsrun-001 -n orchestrator
```

### Step 3: Validate Job Specifications
```bash
# Check Job details to verify correct configuration
kubectl get job <job-name> -n orchestrator -o yaml

# Verify environment variables are set correctly
# Verify volumes are mounted properly (PVC for code, emptyDir for docs)
# Verify SSH key mounting
```

### Step 4: Monitor Status Updates
```bash
# Watch status changes
kubectl get coderun test-coderun-001 -n orchestrator -w
kubectl get docsrun test-docsrun-001 -n orchestrator -w
```

### Step 5: Cleanup
```bash
# Remove test resources
kubectl delete coderun test-coderun-001 -n orchestrator
kubectl delete docsrun test-docsrun-001 -n orchestrator

# Verify finalizers work and resources are cleaned up
kubectl get jobs -n orchestrator
kubectl get configmaps -n orchestrator
```

## ✅ Success Criteria

### For CodeRun:
- [ ] Resource applies without errors
- [ ] Controller logs show CodeRun reconciliation
- [ ] Job created with correct name pattern: `{service}-task{taskId}-attempt{contextVersion}`
- [ ] Job has PVC volume mounted
- [ ] Job has SSH key volume mounted
- [ ] ConfigMap created with code templates
- [ ] Status updates correctly (phase, message, conditions)
- [ ] Finalizer cleanup works

### For DocsRun:
- [ ] Resource applies without errors
- [ ] Controller logs show DocsRun reconciliation
- [ ] Job created with correct name pattern: `{githubUser}-{repo}-docs`
- [ ] Job has emptyDir volume (no PVC)
- [ ] Job has SSH key volume mounted
- [ ] ConfigMap created with docs templates
- [ ] Status updates correctly (phase, message, conditions)
- [ ] Finalizer cleanup works

## 🚨 Potential Issues to Watch For

1. **Missing SSH Keys**: Jobs may fail if `github-ssh-{username}` secret doesn't exist
2. **ANTHROPIC_API_KEY**: Jobs may fail if API key secret is missing/misconfigured
3. **Template Errors**: ConfigMap creation may fail if templates have syntax errors
4. **Permission Issues**: Controller may lack RBAC permissions for new CRDs
5. **Resource Conflicts**: Old TaskRun resources might interfere

## 🔧 Troubleshooting Commands

```bash
# Check controller RBAC permissions
kubectl auth can-i create coderuns --as=system:serviceaccount:orchestrator:orchestrator

# Check if required secrets exist
kubectl get secret -n orchestrator | grep github-ssh
kubectl get secret -n orchestrator | grep anthropic

# View controller detailed logs
kubectl logs -n orchestrator deployment/orchestrator --previous

# Debug failing Jobs
kubectl describe job <job-name> -n orchestrator
kubectl logs job/<job-name> -n orchestrator
```

---

**📝 Notes for Review:**
- Test resources use safe, non-destructive configurations
- Both tests target the same repository but different branches/workflows
- Tests include both required and optional fields to validate defaults
- Cleanup procedures ensure no resource leakage
- Success criteria are concrete and measurable

**❓ Questions for Feedback:**
1. Should we test edge cases (missing optional fields, invalid values)?
2. Should we include tests for the retry/versioning functionality?
3. Any specific GitHub repository/branch preferences for testing?
4. Should we test both CRDs simultaneously or separately?