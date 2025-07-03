# TaskRun CRD Design

## Overview

The TaskRun Custom Resource Definition (CRD) is the core of the 5D Labs Platform's agent deployment system. It provides a Kubernetes-native way to define, manage, and monitor Claude Code agent tasks.

## CRD Schema

### TaskRun Resource Structure

```yaml
apiVersion: orchestrator.io/v1
kind: TaskRun
metadata:
  name: auth-service-task-1001
  namespace: orchestrator
  labels:
    task-id: "1001"
    service: "auth-service"
    priority: "high"
spec:
  taskId: 1001
  serviceName: "auth-service"
  priority: "high"
  markdownFiles:
    - filename: "task.md"
      content: "# Task Implementation Details..."
      fileType: "task"
    - filename: "design-spec.md"
      content: "# Service Design Specification..."
      fileType: "design-spec"
    - filename: "prompt.md"
      content: "# Autonomous Agent Instructions..."
      fileType: "prompt"
  retryAttempt: 1
status:
  phase: "Running"
  conditions:
    - type: "ConfigMapReady"
      status: "True"
      lastTransitionTime: "2024-07-02T10:00:00Z"
    - type: "JobCreated"
      status: "True"
      lastTransitionTime: "2024-07-02T10:01:00Z"
  configMapName: "auth-service-task-1001-run-1-files"
  jobName: "auth-service-task-1001-run-1"
  startTime: "2024-07-02T10:00:00Z"
  message: "Claude agent is implementing authentication service"
```

## Controller Implementation

### Reconciliation Loop

The TaskRun controller follows the standard Kubernetes controller pattern:

1. **Watch**: Monitor TaskRun resources for changes
2. **Reconcile**: Ensure desired state matches actual state
3. **Update Status**: Report current status back to the resource

### Key Controller Features

#### Finalizers
- `taskrun.orchestrator.io/cleanup` ensures proper resource cleanup
- Removes ConfigMaps and Jobs when TaskRun is deleted
- Handles cleanup even if controller is restarted

#### Status Subresource
- Separate status updates prevent conflicts with spec changes
- Conditions track detailed progress information
- Phase provides high-level status overview

#### Server-Side Apply
- Conflict-free updates to Jobs and ConfigMaps
- Handles concurrent modifications gracefully
- Maintains ownership and field management

## ConfigMap Pattern

### File Organization

TaskRun creates ConfigMaps with all task files:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: auth-service-task-1001-run-1-files
data:
  task.md: |
    # Task 1001: Implement Authentication Service
    **Priority:** high
    **Service:** auth-service
    ...
  design-spec.md: |
    # Authentication Service Design
    ## Architecture Overview
    ...
  prompt.md: |
    # Autonomous Agent Instructions
    You are implementing an authentication service...
  CLAUDE.md: |
    # Auth Service - Task Context
    - Task details: @.task/1001/run-1/task.md
    - Design spec: @.task/1001/run-1/design-spec.md
    ...
```

### Init Container Workflow

1. **Mount ConfigMap** at `/config/` in init container
2. **Create Directory Structure**:
   ```
   /workspace/auth-service/
   ├── .task/1001/run-1/
   │   ├── task.md
   │   ├── design-spec.md
   │   ├── prompt.md
   │   └── metadata.yaml
   └── CLAUDE.md (with @imports)
   ```
3. **Copy Files** from ConfigMap to workspace
4. **Generate CLAUDE.md** with proper @import references

## Job Management

### Job Template

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: auth-service-task-1001-run-1
spec:
  template:
    spec:
      initContainers:
      - name: prepare-workspace
        image: busybox:1.36
        command: ["/bin/sh", "-c"]
        args: ["setup script here"]
        volumeMounts:
        - name: task-files
          mountPath: /config
        - name: workspace
          mountPath: /workspace
      containers:
      - name: claude-agent
        image: ghcr.io/5dlabs/platform/claude-code:latest
        command: ["claude", "--resume"]
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: orchestrator-secrets
              key: ANTHROPIC_API_KEY
        volumeMounts:
        - name: workspace
          mountPath: /workspace
      volumes:
      - name: task-files
        configMap:
          name: auth-service-task-1001-run-1-files
      - name: workspace
        persistentVolumeClaim:
          claimName: auth-service-workspace-pvc
```

### Job Monitoring

The controller monitors job status and updates TaskRun accordingly:

- **Job Created**: `JobCreated` condition set to True
- **Job Running**: Phase set to "Running"
- **Job Succeeded**: Phase set to "Succeeded"
- **Job Failed**: Phase set to "Failed", increment retry attempt

## Configuration Management

### Controller Config

Configuration is loaded from the `taskrun-controller-config` ConfigMap:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: taskrun-controller-config
data:
  config.yaml: |
    job:
      backoffLimit: 3
      activeDeadlineSeconds: 3600
      ttlSecondsAfterFinished: 86400
    agent:
      image: "ghcr.io/5dlabs/platform/claude-code:latest"
      resources:
        requests:
          cpu: "1"
          memory: "2Gi"
        limits:
          cpu: "2"
          memory: "4Gi"
    telemetry:
      enabled: true
      otlpEndpoint: "otel-collector-opentelemetry-collector.telemetry.svc.cluster.local:4317"
```

### Runtime Updates

Configuration changes are detected and applied without restarting the controller:

1. **ConfigMap Watcher**: Monitors config changes
2. **Hot Reload**: Updates in-memory configuration
3. **New Deployments**: Use updated configuration immediately

## Error Handling

### Retry Logic

- **Backoff Strategy**: Exponential backoff with jitter
- **Max Retries**: Configurable limit (default: 3)
- **Retry Conditions**: Job failures, timeout errors
- **Manual Retry**: Update `retryAttempt` field to force retry

### Error Reporting

- **Status Conditions**: Detailed error information
- **Events**: Kubernetes events for debugging
- **Logs**: Structured logging with correlation IDs

## Security Considerations

### RBAC Requirements

The controller requires these permissions:

```yaml
rules:
- apiGroups: ["orchestrator.io"]
  resources: ["taskruns"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["orchestrator.io"]
  resources: ["taskruns/status"]
  verbs: ["get", "update", "patch"]
- apiGroups: ["batch"]
  resources: ["jobs"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
```

### Secret Management

- **Agent Secrets**: Anthropic API key, GitHub token
- **Workspace Secrets**: Service-specific credentials
- **TLS Certificates**: For secure communications

## Monitoring and Observability

### Metrics

The controller exposes Prometheus metrics:

- `taskrun_reconciliations_total`: Total reconciliation attempts
- `taskrun_reconciliation_duration_seconds`: Time spent reconciling
- `taskrun_jobs_created_total`: Total jobs created
- `taskrun_configmaps_created_total`: Total ConfigMaps created

### Traces

OpenTelemetry traces provide detailed operation visibility:

- **Reconciliation Traces**: Full reconciliation workflow
- **Job Creation Traces**: Job and ConfigMap creation
- **Error Traces**: Detailed error context

### Logs

Structured logging with correlation:

```json
{
  "level": "info",
  "msg": "TaskRun reconciled successfully",
  "taskrun": "auth-service-task-1001",
  "namespace": "orchestrator",
  "job": "auth-service-task-1001-run-1",
  "phase": "Running",
  "attempt": 1
}
```

## Best Practices

### TaskRun Design

1. **Descriptive Names**: Include service and task ID
2. **Proper Labels**: Enable filtering and grouping
3. **Resource Limits**: Set appropriate CPU/memory limits
4. **Retry Strategy**: Configure based on task complexity

### File Organization

1. **Clear Structure**: Organize files logically in ConfigMaps
2. **Size Limits**: Keep individual files under 1MB
3. **Format Consistency**: Use Markdown for documentation files
4. **Version Control**: Include metadata for tracking

### Monitoring

1. **Health Checks**: Monitor controller health
2. **Resource Usage**: Track CPU/memory consumption
3. **Error Rates**: Monitor job failure rates
4. **Performance**: Track reconciliation latency

## Troubleshooting

### Common Issues

1. **ConfigMap Too Large**: Split large files, use external storage
2. **Job Timeout**: Increase `activeDeadlineSeconds`
3. **Resource Constraints**: Adjust resource requests/limits
4. **RBAC Errors**: Verify controller permissions

### Debugging Commands

```bash
# Check TaskRun status
kubectl describe taskrun auth-service-task-1001 -n orchestrator

# View controller logs
kubectl logs -n orchestrator -l app=orchestrator -f

# Check job status
kubectl get jobs -n orchestrator -l task-id=1001

# Inspect ConfigMap
kubectl describe configmap auth-service-task-1001-run-1-files -n orchestrator
```

## Future Enhancements

1. **Multi-Agent Support**: Deploy multiple agents per task
2. **Dependency Management**: Task dependencies and sequencing
3. **Resource Scheduling**: Intelligent resource allocation
4. **Advanced Retry**: Conditional retry based on error types
5. **Integration APIs**: Webhook notifications, external triggers