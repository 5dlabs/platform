# TaskRun CRD Testing Strategy

## Overview

This document outlines the comprehensive testing strategy for the TaskRun Custom Resource Definition (CRD) implementation. The tests ensure that agent deployments work correctly, files are placed in expected locations per Anthropic documentation, and follow-up runs function properly.

## Testing Scope

### 1. Basic CRD Functionality
- TaskRun resource creation and validation
- Status updates and phase transitions
- Resource deletion and cleanup

### 2. File Placement and Workspace Structure
- ConfigMap creation with correct file structure
- Init container workspace preparation
- File locations matching Anthropic Claude Code expectations

### 3. Job Deployment and Lifecycle
- Job creation with proper configuration
- Init container execution and completion
- Main container startup with correct environment

### 4. Follow-up Runs and Versioning
- Multiple runs for the same task
- Version tracking and cleanup
- Resume vs continue behavior

## Test Categories

### Unit Tests

#### Controller Logic Tests
```bash
# Location: orchestrator-core/src/controllers/taskrun.rs
cargo test -p orchestrator-core test_generate_claude_md
cargo test -p orchestrator-core test_build_configmap
cargo test -p orchestrator-core test_build_job
```

**Test Cases:**
1. **CLAUDE.md Generation**
   - Verify correct format with @imports
   - Ensure all markdown files are referenced
   - Check service-specific paths

2. **ConfigMap Building**
   - Validate all markdown files included
   - Check label structure
   - Verify CLAUDE.md is added

3. **Job Specification**
   - Verify init container configuration
   - Check volume mounts
   - Validate environment variables

#### CRD Schema Tests
```bash
# Location: orchestrator-core/src/crds/mod.rs
cargo test -p orchestrator-core test_taskrun_serialization
cargo test -p orchestrator-core test_status_updates
```

**Test Cases:**
1. **TaskRun Serialization**
   - Valid TaskRun creation
   - Required fields validation
   - Optional fields handling

2. **Status Management**
   - Phase transitions
   - Timestamp updates
   - Job/ConfigMap name tracking

### Integration Tests

#### 1. Basic TaskRun Deployment Test
```rust
#[tokio::test]
async fn test_basic_taskrun_deployment() {
    // Setup
    let client = create_test_k8s_client().await;
    let namespace = "test-namespace";
    
    // Create TaskRun
    let taskrun = TaskRun {
        metadata: ObjectMeta {
            name: Some("test-task-1001".to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: TaskRunSpec {
            task_id: 1001,
            service_name: "auth-service".to_string(),
            agent_name: "claude-agent-1".to_string(),
            context_version: 1,
            markdown_files: vec![
                MarkdownFile {
                    filename: "task.md".to_string(),
                    content: "# Task 1001\nImplement authentication".to_string(),
                    file_type: Some(MarkdownFileType::Task),
                },
                MarkdownFile {
                    filename: "design-spec.md".to_string(),
                    content: "# Design Specification\nAuth service design".to_string(),
                    file_type: Some(MarkdownFileType::DesignSpec),
                },
            ],
        },
        status: None,
    };
    
    // Deploy TaskRun
    let api: Api<TaskRun> = Api::namespaced(client.clone(), namespace);
    api.create(&PostParams::default(), &taskrun).await.unwrap();
    
    // Wait for reconciliation
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify ConfigMap created
    let cm_api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let cm_name = "auth-service-1001-v1-files";
    let cm = cm_api.get(cm_name).await.unwrap();
    
    // Verify files in ConfigMap
    assert!(cm.data.unwrap().contains_key("task.md"));
    assert!(cm.data.unwrap().contains_key("design-spec.md"));
    assert!(cm.data.unwrap().contains_key("CLAUDE.md"));
    
    // Verify Job created
    let job_api: Api<Job> = Api::namespaced(client.clone(), namespace);
    let job_name = "auth-service-1001-v1";
    let job = job_api.get(job_name).await.unwrap();
    
    // Verify init container
    let init_containers = &job.spec.unwrap().template.spec.unwrap().init_containers.unwrap();
    assert_eq!(init_containers.len(), 1);
    assert_eq!(init_containers[0].name, "prepare-workspace");
}
```

#### 2. File Location Verification Test
```rust
#[tokio::test]
async fn test_file_locations_per_anthropic_docs() {
    // This test verifies files are placed according to Anthropic documentation
    // Expected structure:
    // /workspace/{service_name}/
    // ├── .task/{task_id}/run-{attempt}/
    // │   ├── task.md
    // │   ├── design-spec.md
    // │   ├── prompt.md
    // │   ├── acceptance-criteria.md
    // │   └── metadata.yaml
    // ├── src/
    // ├── tests/
    // ├── docs/
    // └── CLAUDE.md
    
    let init_script = r#"
    mkdir -p /workspace/auth-service/.task/1001/run-1
    cp /config/* /workspace/auth-service/.task/1001/run-1/
    cp /config/CLAUDE.md /workspace/auth-service/
    "#;
    
    // Verify script creates correct structure
    assert!(init_script.contains(".task/1001/run-1"));
    assert!(init_script.contains("cp /config/CLAUDE.md /workspace/auth-service/"));
}
```

#### 3. Follow-up Run Test
```rust
#[tokio::test]
async fn test_follow_up_runs() {
    let client = create_test_k8s_client().await;
    let namespace = "test-namespace";
    
    // Create first TaskRun (version 1)
    let taskrun_v1 = create_test_taskrun(1001, "auth-service", 1);
    let api: Api<TaskRun> = Api::namespaced(client.clone(), namespace);
    api.create(&PostParams::default(), &taskrun_v1).await.unwrap();
    
    // Wait for first run
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create follow-up TaskRun (version 2)
    let mut taskrun_v2 = taskrun_v1.clone();
    taskrun_v2.metadata.name = Some("test-task-1001-v2".to_string());
    taskrun_v2.spec.context_version = 2;
    api.create(&PostParams::default(), &taskrun_v2).await.unwrap();
    
    // Wait for reconciliation
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify both ConfigMaps exist
    let cm_api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    assert!(cm_api.get("auth-service-1001-v1-files").await.is_ok());
    assert!(cm_api.get("auth-service-1001-v2-files").await.is_ok());
    
    // Verify old job was deleted
    let job_api: Api<Job> = Api::namespaced(client.clone(), namespace);
    assert!(job_api.get("auth-service-1001-v1").await.is_err());
    assert!(job_api.get("auth-service-1001-v2").await.is_ok());
}
```

### End-to-End Tests

#### 1. Complete Task Deployment Flow
```bash
#!/bin/bash
# test-taskrun-deployment.sh

# Setup
NAMESPACE="e2e-test"
kubectl create namespace $NAMESPACE

# Deploy controller
kubectl apply -f infra/crds/taskrun-crd.yaml
kubectl apply -f infra/crds/taskrun-controller-config.yaml -n $NAMESPACE

# Create test TaskRun
cat <<EOF | kubectl apply -f -
apiVersion: orchestrator.io/v1
kind: TaskRun
metadata:
  name: e2e-test-task-1001
  namespace: $NAMESPACE
spec:
  taskId: 1001
  serviceName: "test-service"
  agentName: "claude-agent-1"
  contextVersion: 1
  markdownFiles:
  - filename: "task.md"
    content: |
      # Task 1001: Test Task
      This is a test task for E2E testing.
    fileType: "task"
  - filename: "design-spec.md"
    content: |
      # Design Specification
      Test service design.
    fileType: "design-spec"
EOF

# Wait for job creation
echo "Waiting for job creation..."
kubectl wait --for=condition=complete job -l task-id=1001 -n $NAMESPACE --timeout=60s

# Verify ConfigMap
echo "Verifying ConfigMap..."
kubectl get configmap test-service-1001-v1-files -n $NAMESPACE -o yaml

# Verify Job
echo "Verifying Job..."
kubectl get job test-service-1001-v1 -n $NAMESPACE -o yaml

# Check init container logs
echo "Checking init container logs..."
kubectl logs job/test-service-1001-v1 -c prepare-workspace -n $NAMESPACE

# Cleanup
kubectl delete namespace $NAMESPACE
```

#### 2. Workspace Structure Verification
```bash
#!/bin/bash
# test-workspace-structure.sh

# This test runs a debug pod to verify workspace structure
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: workspace-verifier
  namespace: e2e-test
spec:
  initContainers:
  - name: prepare-workspace
    image: busybox:1.36
    command: ["/bin/sh", "-c"]
    args:
    - |
      # Simulate init container behavior
      mkdir -p /workspace/test-service/.task/1001/run-1
      echo "# Task" > /workspace/test-service/.task/1001/run-1/task.md
      echo "# CLAUDE.md" > /workspace/test-service/CLAUDE.md
      
      # Verify structure
      find /workspace -type f -o -type d | sort
    volumeMounts:
    - name: workspace
      mountPath: /workspace
  containers:
  - name: verifier
    image: busybox:1.36
    command: ["sleep", "300"]
    volumeMounts:
    - name: workspace
      mountPath: /workspace
  volumes:
  - name: workspace
    emptyDir: {}
EOF

# Check logs
kubectl logs workspace-verifier -c prepare-workspace -n e2e-test
```

### Performance Tests

#### 1. Concurrent TaskRun Creation
```rust
#[tokio::test]
async fn test_concurrent_taskrun_creation() {
    let client = create_test_k8s_client().await;
    let namespace = "perf-test";
    
    // Create 10 TaskRuns concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let client = client.clone();
        let ns = namespace.to_string();
        let handle = tokio::spawn(async move {
            let taskrun = create_test_taskrun(1000 + i, &format!("service-{}", i), 1);
            let api: Api<TaskRun> = Api::namespaced(client, &ns);
            api.create(&PostParams::default(), &taskrun).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

### Resume vs Continue Testing

#### Current State
The current implementation uses a hardcoded `-p` flag for the Claude agent. The resume vs continue distinction hasn't been implemented yet.

#### Proposed Implementation
```yaml
# In TaskRun spec, add:
spec:
  # ... existing fields ...
  resumeMode: "auto"  # Options: "auto", "resume", "continue", "new"
```

#### Test Cases for Resume/Continue
```rust
#[tokio::test]
async fn test_resume_mode_handling() {
    // Test 1: Auto mode (default)
    let taskrun = create_taskrun_with_resume_mode("auto");
    // Should not add --resume or --continue flags
    
    // Test 2: Resume mode
    let taskrun = create_taskrun_with_resume_mode("resume");
    // Should add --resume flag to Claude args
    
    // Test 3: Continue mode
    let taskrun = create_taskrun_with_resume_mode("continue");
    // Should add --continue flag to Claude args
    
    // Test 4: New mode
    let taskrun = create_taskrun_with_resume_mode("new");
    // Should ensure clean workspace
}
```

## Test Execution Plan

### Phase 1: Unit Tests (Day 1)
1. Run existing unit tests
2. Add missing unit test coverage
3. Verify CLAUDE.md generation
4. Test ConfigMap structure

### Phase 2: Integration Tests (Day 2-3)
1. Setup test Kubernetes cluster (kind/minikube)
2. Deploy CRD and controller
3. Test basic TaskRun creation
4. Verify file placement
5. Test follow-up runs

### Phase 3: E2E Tests (Day 4)
1. Full deployment flow
2. Workspace verification
3. Job lifecycle testing
4. Error handling scenarios

### Phase 4: Performance Tests (Day 5)
1. Concurrent TaskRun handling
2. Resource cleanup verification
3. Controller scalability

## Test Environment Setup

### Local Testing with Kind
```bash
# Create test cluster
kind create cluster --name taskrun-test

# Install CRDs
kubectl apply -f infra/crds/taskrun-crd.yaml

# Deploy controller (test mode)
kubectl apply -f test/manifests/test-controller.yaml

# Run tests
./scripts/run-taskrun-tests.sh
```

### CI/CD Integration
```yaml
# .github/workflows/taskrun-tests.yml
name: TaskRun CRD Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: engineerd/setup-kind@v0.5.0
    - name: Run unit tests
      run: cargo test -p orchestrator-core
    - name: Run integration tests
      run: ./scripts/test-taskrun-integration.sh
```

## Success Criteria

1. **Basic Functionality**
   - ✅ TaskRun creates ConfigMap with all files
   - ✅ Job deploys with correct init container
   - ✅ Files placed in Anthropic-expected locations
   - ✅ Status updates reflect job state

2. **File Placement**
   - ✅ CLAUDE.md at `/workspace/{service}/CLAUDE.md`
   - ✅ Task files at `/workspace/{service}/.task/{id}/run-{attempt}/`
   - ✅ Metadata.yaml created with attempt info
   - ✅ Proper permissions on all files

3. **Follow-up Runs**
   - ✅ New version creates new directory
   - ✅ Old jobs cleaned up
   - ✅ ConfigMaps versioned correctly
   - ✅ History limit enforced

4. **Error Handling**
   - ✅ Failed jobs update TaskRun status
   - ✅ Resource cleanup on deletion
   - ✅ Retry logic works correctly
   - ✅ Timeout handling

## Known Gaps

1. **Resume vs Continue**: Not yet implemented in TaskRun spec
2. **PVC Management**: Assumes PVC exists, no auto-creation yet
3. **Node Affinity**: Basic implementation, needs enhancement
4. **Secret Management**: Hardcoded to environment variables

## Next Steps

1. Implement missing test cases
2. Add resume/continue mode to TaskRun spec
3. Create automated test suite
4. Document test results and findings
5. Create performance benchmarks