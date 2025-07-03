# Operations Complete Guide

## Overview

This guide covers the complete operational aspects of the 5D Labs Platform, including deployment, monitoring, maintenance, and troubleshooting procedures.

## Prerequisites

### Infrastructure Requirements

**Kubernetes Cluster**:
- Kubernetes 1.24+ with CRD support
- Node capacity: 4+ cores, 16GB+ RAM per worker node
- Storage: Local SSD preferred, network storage acceptable
- Network: Outbound HTTPS access for GitHub, package registries

**External Dependencies**:
- GitHub repository access with Actions enabled
- Anthropic API key for Claude access
- Container registry (GitHub Container Registry recommended)
- DNS resolution for ingress (optional)

### Required Tools

**Local Development**:
```bash
# Install required CLI tools
curl -fsSL https://get.helm.sh/helm-v3.12.0-linux-amd64.tar.gz | tar -xz
sudo mv linux-amd64/helm /usr/local/bin/

# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl && sudo mv kubectl /usr/local/bin/

# Install GitHub CLI
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update && sudo apt install gh
```

## Deployment Procedures

### 1. Initial Platform Setup

#### Step 1: Prepare Cluster
```bash
# Create namespace
kubectl create namespace orchestrator

# Create namespace for telemetry (optional)
kubectl create namespace telemetry
```

#### Step 2: Install CRDs
```bash
# Install TaskRun CRD
kubectl apply -f infra/crds/taskrun-crd.yaml

# Verify CRD installation
kubectl wait --for condition=established --timeout=60s crd/taskruns.orchestrator.io
kubectl api-resources | grep taskrun
```

#### Step 3: Configure Secrets
```bash
# Create orchestrator secrets
kubectl create secret generic orchestrator-secrets \
  --from-literal=ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
  --from-literal=GITHUB_TOKEN=$GITHUB_TOKEN \
  --namespace orchestrator

# Verify secret creation
kubectl get secrets -n orchestrator
```

#### Step 4: Deploy Controller Configuration
```bash
# Apply controller configuration
kubectl apply -f infra/crds/taskrun-controller-config.yaml

# Verify configuration
kubectl describe configmap taskrun-controller-config -n orchestrator
```

#### Step 5: Deploy Orchestrator
```bash
# Install via Helm
helm install orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --values ./infra/charts/orchestrator/values-production.yaml

# Verify deployment
kubectl get pods -n orchestrator
kubectl logs -n orchestrator -l app=orchestrator -f
```

### 2. Telemetry Stack Deployment (Optional)

#### OpenTelemetry Collector
```bash
# Add Helm repository
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts

# Install collector
helm install otel-collector open-telemetry/opentelemetry-collector \
  --namespace telemetry \
  --create-namespace \
  --values ./infra/telemetry/otel-values.yaml
```

#### VictoriaMetrics & VictoriaLogs
```bash
# Add VictoriaMetrics repository
helm repo add vm https://victoriametrics.github.io/helm-charts/

# Install VictoriaMetrics
helm install victoria-metrics vm/victoria-metrics-cluster \
  --namespace telemetry \
  --values ./infra/telemetry/vm-values.yaml

# Install VictoriaLogs
helm install victoria-logs vm/victoria-logs-single \
  --namespace telemetry \
  --values ./infra/telemetry/vl-values.yaml
```

#### Grafana
```bash
# Add Grafana repository
helm repo add grafana https://grafana.github.io/helm-charts

# Install Grafana
helm install grafana grafana/grafana \
  --namespace telemetry \
  --values ./infra/telemetry/grafana-values.yaml

# Get admin password
kubectl get secret --namespace telemetry grafana -o jsonpath="{.data.admin-password}" | base64 --decode
```

### 3. Workspace Setup

#### Create Workspace PVCs
```bash
# Create workspace for specific service
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: auth-service-workspace-pvc
  namespace: orchestrator
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: local-path
  resources:
    requests:
      storage: 50Gi
EOF
```

#### Workspace Initialization
```bash
# Initialize workspace with service skeleton (optional)
kubectl run workspace-init \
  --image=busybox:1.36 \
  --restart=Never \
  --rm -i --tty \
  --overrides='
{
  "spec": {
    "containers": [{
      "name": "workspace-init",
      "image": "busybox:1.36",
      "command": ["/bin/sh"],
      "args": ["-c", "mkdir -p /workspace/auth-service/src /workspace/auth-service/tests"],
      "volumeMounts": [{
        "name": "workspace",
        "mountPath": "/workspace"
      }]
    }],
    "volumes": [{
      "name": "workspace",
      "persistentVolumeClaim": {
        "claimName": "auth-service-workspace-pvc"
      }
    }]
  }
}' \
  -- /bin/sh
```

## Configuration Management

### 1. Runtime Configuration Updates

#### Controller Configuration
```bash
# Update controller config
kubectl edit configmap taskrun-controller-config -n orchestrator

# Configuration is hot-reloaded automatically
# Verify changes in controller logs
kubectl logs -n orchestrator -l app=orchestrator -f | grep "Configuration reloaded"
```

#### Orchestrator Settings
```bash
# Update Helm values
vim ./infra/charts/orchestrator/values-production.yaml

# Apply changes
helm upgrade orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --values ./infra/charts/orchestrator/values-production.yaml

# Monitor rollout
kubectl rollout status deployment/orchestrator -n orchestrator
```

### 2. Secret Rotation

#### API Key Rotation
```bash
# Update Anthropic API key
kubectl patch secret orchestrator-secrets -n orchestrator \
  --type='json' \
  -p='[{"op": "replace", "path": "/data/ANTHROPIC_API_KEY", "value":"'$(echo -n $NEW_ANTHROPIC_API_KEY | base64)'"}]'

# Restart orchestrator to pick up new secret
kubectl rollout restart deployment/orchestrator -n orchestrator
```

#### GitHub Token Rotation
```bash
# Update GitHub token
kubectl patch secret orchestrator-secrets -n orchestrator \
  --type='json' \
  -p='[{"op": "replace", "path": "/data/GITHUB_TOKEN", "value":"'$(echo -n $NEW_GITHUB_TOKEN | base64)'"}]'
```

### 3. Image Updates

#### Automated Updates (CI/CD)
```bash
# Images are automatically built and pushed by GitHub Actions
# Production values can reference specific versions:

# Update to specific version
helm upgrade orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --set image.tag=main-abc1234 \
  --values ./infra/charts/orchestrator/values-production.yaml
```

#### Manual Updates
```bash
# Update to latest
helm upgrade orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --set image.tag=latest \
  --values ./infra/charts/orchestrator/values-production.yaml

# Verify new image
kubectl get pods -n orchestrator -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].image}{"\n"}{end}'
```

## Monitoring and Alerting

### 1. Health Checks

#### Orchestrator Health
```bash
# Check orchestrator health endpoint
kubectl port-forward -n orchestrator svc/orchestrator 8080:80 &
curl http://localhost:8080/health

# Expected response:
# {
#   "status": "healthy",
#   "version": "0.1.0",
#   "timestamp": "2024-07-02T10:00:00Z"
# }
```

#### TaskRun Status Monitoring
```bash
# List all TaskRuns
kubectl get taskruns -n orchestrator

# Check specific TaskRun
kubectl describe taskrun auth-service-task-1001 -n orchestrator

# Monitor TaskRun status
watch kubectl get taskruns -n orchestrator
```

#### Resource Usage Monitoring
```bash
# Pod resource usage
kubectl top pods -n orchestrator

# Node resource usage
kubectl top nodes

# Persistent volume usage
df -h /var/lib/rancher/k3s/storage/
```

### 2. Metrics Collection

#### Orchestrator Metrics
```bash
# Access metrics endpoint
kubectl port-forward -n orchestrator svc/orchestrator 8080:80 &
curl http://localhost:8080/metrics

# Key metrics to monitor:
# - taskrun_reconciliations_total
# - taskrun_reconciliation_duration_seconds
# - taskrun_jobs_created_total
# - taskrun_active_tasks
```

#### Kubernetes Metrics
```bash
# Install metrics-server if not available
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

# Enable insecure TLS for development
kubectl patch deployment metrics-server -n kube-system --type='json' \
  -p='[{"op": "add", "path": "/spec/template/spec/containers/0/args/-", "value": "--kubelet-insecure-tls"}]'
```

### 3. Log Aggregation

#### Structured Logging
```bash
# Controller logs with JSON formatting
kubectl logs -n orchestrator -l app=orchestrator -f | jq .

# Agent logs
kubectl logs -n orchestrator -l app=claude-agent -f
```

#### Log Retention
```bash
# Configure log rotation (example for systemd)
# /etc/systemd/journald.conf
[Journal]
SystemMaxUse=1G
SystemMaxFileSize=100M
SystemMaxFiles=10
```

### 4. Alerting Rules

#### Prometheus Alerting Rules
```yaml
# alerts.yaml
groups:
- name: orchestrator
  rules:
  - alert: OrchestratorDown
    expr: up{job="orchestrator"} == 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Orchestrator is down"
      description: "Orchestrator has been down for more than 5 minutes"

  - alert: TaskRunFailureRate
    expr: rate(taskrun_jobs_failed_total[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High TaskRun failure rate"
      description: "TaskRun failure rate is above 10% for the last 5 minutes"

  - alert: HighTaskRunDuration
    expr: histogram_quantile(0.95, rate(taskrun_reconciliation_duration_seconds_bucket[5m])) > 30
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High TaskRun reconciliation duration"
      description: "95th percentile reconciliation duration is above 30 seconds"
```

## Backup and Recovery

### 1. Configuration Backup

#### Automated Backup Script
```bash
#!/bin/bash
# backup-platform.sh

BACKUP_DIR="/backup/platform/$(date +%Y%m%d-%H%M%S)"
mkdir -p $BACKUP_DIR

# Backup CRDs
kubectl get crd taskruns.orchestrator.io -o yaml > $BACKUP_DIR/taskrun-crd.yaml

# Backup ConfigMaps
kubectl get configmap taskrun-controller-config -n orchestrator -o yaml > $BACKUP_DIR/controller-config.yaml

# Backup Secrets (encrypted)
kubectl get secret orchestrator-secrets -n orchestrator -o yaml > $BACKUP_DIR/secrets.yaml

# Backup Helm releases
helm list -n orchestrator -o yaml > $BACKUP_DIR/helm-releases.yaml

# Backup TaskRuns
kubectl get taskruns -n orchestrator -o yaml > $BACKUP_DIR/taskruns.yaml

echo "Backup completed: $BACKUP_DIR"
```

#### Scheduled Backups
```bash
# Add to crontab
0 2 * * * /usr/local/bin/backup-platform.sh
```

### 2. Workspace Backup

#### PVC Snapshot (if supported)
```bash
# Create VolumeSnapshot (requires CSI driver support)
cat <<EOF | kubectl apply -f -
apiVersion: snapshot.storage.k8s.io/v1
kind: VolumeSnapshot
metadata:
  name: auth-service-workspace-snapshot-$(date +%Y%m%d)
  namespace: orchestrator
spec:
  volumeSnapshotClassName: csi-snapclass
  source:
    persistentVolumeClaimName: auth-service-workspace-pvc
EOF
```

#### File-Level Backup
```bash
# Backup workspace files
kubectl run backup-pod \
  --image=busybox:1.36 \
  --restart=Never \
  --rm -i --tty \
  --overrides='
{
  "spec": {
    "containers": [{
      "name": "backup",
      "image": "busybox:1.36",
      "command": ["/bin/sh"],
      "args": ["-c", "tar czf /backup/workspace-$(date +%Y%m%d).tar.gz -C /workspace ."],
      "volumeMounts": [{
        "name": "workspace",
        "mountPath": "/workspace"
      }, {
        "name": "backup",
        "mountPath": "/backup"
      }]
    }],
    "volumes": [{
      "name": "workspace",
      "persistentVolumeClaim": {
        "claimName": "auth-service-workspace-pvc"
      }
    }, {
      "name": "backup",
      "hostPath": {
        "path": "/backup"
      }
    }]
  }
}' \
  -- /bin/sh
```

### 3. Disaster Recovery

#### Complete Platform Recovery
```bash
# 1. Restore cluster access
export KUBECONFIG=/path/to/backup/kubeconfig

# 2. Recreate namespaces
kubectl create namespace orchestrator
kubectl create namespace telemetry

# 3. Restore CRDs
kubectl apply -f /backup/platform/latest/taskrun-crd.yaml

# 4. Restore secrets
kubectl apply -f /backup/platform/latest/secrets.yaml

# 5. Restore configuration
kubectl apply -f /backup/platform/latest/controller-config.yaml

# 6. Restore Helm releases
helm install orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --values ./infra/charts/orchestrator/values-production.yaml

# 7. Restore workspaces
# Mount backup volume and restore files to PVCs
```

## Troubleshooting

### 1. Common Issues

#### Controller Not Starting
```bash
# Check pod status
kubectl get pods -n orchestrator

# Check logs for errors
kubectl logs -n orchestrator -l app=orchestrator

# Common causes:
# - RBAC permissions missing
# - ConfigMap not found
# - Secret not found
# - Invalid configuration

# Resolution:
kubectl describe pod -n orchestrator -l app=orchestrator
```

#### TaskRun Stuck in Pending
```bash
# Check TaskRun status
kubectl describe taskrun {name} -n orchestrator

# Check controller logs
kubectl logs -n orchestrator -l app=orchestrator | grep {taskrun-name}

# Common causes:
# - Resource constraints
# - ConfigMap creation failure
# - Job creation failure
# - Invalid task specification

# Check events
kubectl get events -n orchestrator --sort-by='.lastTimestamp'
```

#### Agent Job Failures
```bash
# Check job status
kubectl get jobs -n orchestrator -l task-id={id}

# Check pod logs
kubectl logs -n orchestrator -l task-id={id}

# Common causes:
# - Compilation errors
# - Test failures
# - Resource limits exceeded
# - Network connectivity issues

# Debug by exec into pod
kubectl exec -it {pod-name} -n orchestrator -- /bin/bash
```

#### High Resource Usage
```bash
# Identify resource-intensive pods
kubectl top pods -n orchestrator --sort-by=cpu
kubectl top pods -n orchestrator --sort-by=memory

# Check resource limits
kubectl describe pod {pod-name} -n orchestrator

# Adjust resource limits
kubectl patch taskrun {name} -n orchestrator --type='merge' -p='
{
  "spec": {
    "resources": {
      "limits": {
        "cpu": "4",
        "memory": "8Gi"
      }
    }
  }
}'
```

### 2. Performance Tuning

#### Controller Performance
```bash
# Increase controller resources
helm upgrade orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --set resources.limits.cpu=1 \
  --set resources.limits.memory=2Gi \
  --values ./infra/charts/orchestrator/values-production.yaml
```

#### Agent Performance
```bash
# Configure agent resource defaults
kubectl patch configmap taskrun-controller-config -n orchestrator --type='merge' -p='
{
  "data": {
    "config.yaml": "
agent:
  resources:
    requests:
      cpu: \"2\"
      memory: \"4Gi\"
    limits:
      cpu: \"4\"
      memory: \"8Gi\"
"
  }
}'
```

#### Storage Performance
```bash
# Use faster storage class
kubectl patch pvc auth-service-workspace-pvc -n orchestrator --type='merge' -p='
{
  "spec": {
    "storageClassName": "fast-ssd"
  }
}'
```

### 3. Network Troubleshooting

#### Connectivity Issues
```bash
# Test external connectivity from agent
kubectl run network-test \
  --image=busybox:1.36 \
  --restart=Never \
  --rm -i --tty \
  -- /bin/sh -c "nslookup github.com && ping -c 3 github.com"

# Test internal connectivity
kubectl run network-test \
  --image=busybox:1.36 \
  --restart=Never \
  --rm -i --tty \
  -- /bin/sh -c "nslookup orchestrator.orchestrator.svc.cluster.local"
```

#### DNS Resolution
```bash
# Check CoreDNS status
kubectl get pods -n kube-system -l k8s-app=kube-dns

# Test DNS resolution
kubectl run dns-test \
  --image=busybox:1.36 \
  --restart=Never \
  --rm -i --tty \
  -- nslookup kubernetes.default.svc.cluster.local
```

## Maintenance Procedures

### 1. Regular Maintenance

#### Weekly Tasks
```bash
# Clean up completed TaskRuns (older than 7 days)
kubectl delete taskruns -n orchestrator --field-selector="status.phase=Succeeded" \
  --selector="!taskrun.orchestrator.io/keep"

# Clean up failed TaskRuns (older than 3 days)
kubectl delete taskruns -n orchestrator --field-selector="status.phase=Failed" \
  --selector="!taskrun.orchestrator.io/keep"

# Verify system health
kubectl get pods -n orchestrator
kubectl top nodes
kubectl top pods -n orchestrator
```

#### Monthly Tasks
```bash
# Update container images
helm repo update
helm upgrade orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --values ./infra/charts/orchestrator/values-production.yaml

# Security audit
kubectl get pods -n orchestrator -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].image}{"\n"}{end}' | xargs -I {} sh -c 'echo "Checking {}" && trivy image {}'

# Certificate renewal (if using cert-manager)
kubectl get certificates -n orchestrator
```

### 2. Capacity Planning

#### Resource Monitoring
```bash
# Historical resource usage
kubectl top pods -n orchestrator --sort-by=cpu
kubectl top pods -n orchestrator --sort-by=memory

# Node capacity
kubectl describe nodes | grep -A 5 "Allocated resources"

# Storage usage
kubectl get pvc -n orchestrator -o custom-columns=NAME:.metadata.name,SIZE:.spec.resources.requests.storage,USED:.status.capacity.storage
```

#### Scaling Recommendations
```bash
# Monitor concurrent TaskRuns
kubectl get taskruns -n orchestrator --field-selector="status.phase=Running" | wc -l

# Calculate required capacity
# Rule of thumb: 1 CPU + 2GB RAM per concurrent agent
# Add 20% buffer for system overhead
```

### 3. Upgrade Procedures

#### Platform Upgrades
```bash
# 1. Backup current state
./backup-platform.sh

# 2. Test upgrade in development
kind create cluster --name upgrade-test
# ... perform upgrade test ...

# 3. Schedule maintenance window
# 4. Perform rolling upgrade
helm upgrade orchestrator ./infra/charts/orchestrator \
  --namespace orchestrator \
  --values ./infra/charts/orchestrator/values-production.yaml

# 5. Verify functionality
kubectl get pods -n orchestrator
kubectl logs -n orchestrator -l app=orchestrator

# 6. Run smoke tests
./scripts/smoke-test.sh
```

#### Kubernetes Upgrades
```bash
# 1. Drain nodes one by one
kubectl drain {node-name} --ignore-daemonsets --delete-local-data

# 2. Upgrade node
# (cluster-specific procedure)

# 3. Uncordon node
kubectl uncordon {node-name}

# 4. Verify platform functionality
kubectl get pods -n orchestrator
```

## Security Operations

### 1. Security Monitoring

#### Access Auditing
```bash
# Review RBAC permissions
kubectl auth can-i --list --as=system:serviceaccount:orchestrator:orchestrator

# Check for privilege escalation
kubectl get clusterrolebindings -o json | jq '.items[] | select(.subjects[]?.name=="orchestrator")'

# Audit secret access
kubectl get events -n orchestrator | grep -i secret
```

#### Vulnerability Scanning
```bash
# Scan container images
trivy image ghcr.io/5dlabs/platform/orchestrator:latest
trivy image ghcr.io/5dlabs/platform/claude-code:latest

# Scan cluster configuration
kube-bench run --config-dir /opt/kube-bench/cfg --config /opt/kube-bench/cfg/config.yaml
```

### 2. Incident Response

#### Security Incident Checklist
1. **Immediate Response**
   - Isolate affected components
   - Preserve evidence
   - Assess impact scope

2. **Containment**
   - Revoke compromised credentials
   - Update access controls
   - Apply security patches

3. **Recovery**
   - Restore from clean backups
   - Verify system integrity
   - Monitor for persistence

4. **Post-Incident**
   - Document lessons learned
   - Update security procedures
   - Conduct security review

#### Emergency Procedures
```bash
# Emergency shutdown
kubectl scale deployment orchestrator --replicas=0 -n orchestrator

# Revoke all API access
kubectl patch secret orchestrator-secrets -n orchestrator \
  --type='json' \
  -p='[{"op": "replace", "path": "/data/ANTHROPIC_API_KEY", "value":""}]'

# Isolate workspaces
kubectl patch networkpolicy default-deny-all -n orchestrator
```

This completes the operations guide covering all aspects of running the platform in production.