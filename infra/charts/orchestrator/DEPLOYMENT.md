# Orchestrator Deployment Guide

This guide walks you through deploying the orchestrator using the Helm chart and migrating from the existing manifest-based deployment.

## Current State Analysis

Based on the current deployment in the `orchestrator` namespace, the following resources are currently deployed via manifests:

- **Deployment**: `orchestrator` (1 replica)
- **Service**: `orchestrator` (ClusterIP)
- **ConfigMap**: `orchestrator-config`
- **Secret**: `orchestrator-secrets`
- **ConfigMap**: `claude-code-helm-chart` (contains Claude Code chart files)
- **ServiceAccount**: `orchestrator`
- **RBAC**: Role and RoleBinding for orchestrator permissions

## Migration Strategy

### Option 1: In-Place Migration (Recommended)

This approach updates the existing deployment with minimal downtime.

#### Step 1: Backup Current Configuration

```bash
# Export current configuration
kubectl get configmap orchestrator-config -n orchestrator -o yaml > backup-config.yaml
kubectl get secret orchestrator-secrets -n orchestrator -o yaml > backup-secrets.yaml

# Extract current API keys (base64 decode them)
kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.ANTHROPIC_API_KEY}' | base64 -d
kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.GITHUB_TOKEN}' | base64 -d
```

#### Step 2: Create Values File

```bash
# Create values file with current configuration
cat > orchestrator-values.yaml << EOF
secrets:
  anthropicApiKey: "$(kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.ANTHROPIC_API_KEY}' | base64 -d)"
  githubToken: "$(kubectl get secret orchestrator-secrets -n orchestrator -o jsonpath='{.data.GITHUB_TOKEN}' | base64 -d)"

config:
  kubernetesNamespace: "orchestrator"
  helmChartPath: "/infra/claude-code"
  helmNamespace: "orchestrator"
  helmTimeout: "600s"
  serverHost: "0.0.0.0"
  serverPort: "8080"
  rustLog: "orchestrator=debug,tower_http=debug,axum=debug"

ingress:
  enabled: true
  hosts:
    - host: orchestrator.local
      paths:
        - path: /
          pathType: Prefix

tolerations:
  - key: node-role.kubernetes.io/control-plane
    operator: Exists
    effect: NoSchedule
EOF
```

#### Step 3: Deploy with Helm

```bash
# Install the Helm chart (this will take over management of existing resources)
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values orchestrator-values.yaml

# Verify deployment
kubectl get pods -n orchestrator
kubectl logs -n orchestrator deployment/orchestrator
```

#### Step 4: Cleanup Old Resources (if needed)

```bash
# Check for any orphaned resources
kubectl get all -n orchestrator

# Remove any old manually created resources that aren't managed by Helm
# (The Helm chart should adopt most existing resources)
```

### Option 2: Fresh Deployment

This approach creates a new deployment alongside the old one, then switches traffic.

#### Step 1: Deploy to New Namespace

```bash
# Create values file
cp infra/orchestrator-chart/values-example.yaml orchestrator-prod-values.yaml
# Edit the file with your actual API keys and configuration

# Deploy to new namespace
helm install orchestrator-new ./infra/orchestrator-chart \
  --namespace orchestrator-new \
  --create-namespace \
  --values orchestrator-prod-values.yaml
```

#### Step 2: Test New Deployment

```bash
# Test the new deployment
kubectl port-forward -n orchestrator-new svc/orchestrator 8080:80
curl http://localhost:8080/health

# Test Claude Code deployment functionality
# (Submit a test task to verify the orchestrator can deploy agents)
```

#### Step 3: Switch Traffic

```bash
# Update ingress or load balancer to point to new service
# Or scale down old deployment and rename new one

# Scale down old deployment
kubectl scale deployment orchestrator --replicas=0 -n orchestrator

# Verify new deployment is working
kubectl get pods -n orchestrator-new
```

#### Step 4: Cleanup Old Deployment

```bash
# Remove old resources
kubectl delete namespace orchestrator

# Rename new namespace (optional)
# This requires recreating resources, so consider keeping the new namespace name
```

## Deployment Verification

### Health Checks

```bash
# Check pod status
kubectl get pods -n orchestrator -l app.kubernetes.io/name=orchestrator

# Check health endpoint
kubectl port-forward -n orchestrator svc/orchestrator 8080:80
curl http://localhost:8080/health

# Check via ingress (if configured)
curl http://orchestrator.local/health
```

### Configuration Verification

```bash
# Verify ConfigMap
kubectl get configmap -n orchestrator -l app.kubernetes.io/name=orchestrator

# Verify Secret (without exposing values)
kubectl get secret -n orchestrator -l app.kubernetes.io/name=orchestrator

# Verify RBAC
kubectl auth can-i create jobs --as=system:serviceaccount:orchestrator:orchestrator -n orchestrator
kubectl auth can-i create configmaps --as=system:serviceaccount:orchestrator:orchestrator -n orchestrator
```

### Claude Code Chart Verification

```bash
# Verify the Claude Code chart is mounted
kubectl exec -n orchestrator deployment/orchestrator -- ls -la /infra/claude-code/

# Check chart files
kubectl exec -n orchestrator deployment/orchestrator -- cat /infra/claude-code/Chart.yaml
```

## Monitoring and Logging

### Logs

```bash
# View orchestrator logs
kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator -f

# View previous logs (if pod restarted)
kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator --previous
```

### Metrics

```bash
# If metrics are enabled, check metrics endpoint
kubectl port-forward -n orchestrator svc/orchestrator 8080:80
curl http://localhost:8080/metrics
```

## Troubleshooting

### Common Issues

1. **Pod not starting**
   ```bash
   kubectl describe pod -n orchestrator -l app.kubernetes.io/name=orchestrator
   kubectl logs -n orchestrator -l app.kubernetes.io/name=orchestrator
   ```

2. **RBAC permissions**
   ```bash
   kubectl auth can-i create jobs --as=system:serviceaccount:orchestrator:orchestrator -n orchestrator
   ```

3. **ConfigMap/Secret issues**
   ```bash
   kubectl get configmap,secret -n orchestrator
   kubectl describe configmap orchestrator-config -n orchestrator
   ```

4. **Ingress not working**
   ```bash
   kubectl get ingress -n orchestrator
   kubectl describe ingress orchestrator -n orchestrator
   ```

### Recovery Procedures

If something goes wrong during migration:

```bash
# Rollback Helm release
helm rollback orchestrator -n orchestrator

# Or restore from backup
kubectl apply -f backup-config.yaml
kubectl apply -f backup-secrets.yaml

# Scale up original deployment if it was scaled down
kubectl scale deployment orchestrator --replicas=1 -n orchestrator
```

## Maintenance

### Updating the Orchestrator

```bash
# Update image tag
helm upgrade orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --set image.tag="v1.1.0" \
  --reuse-values

# Update configuration
helm upgrade orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values updated-values.yaml
```

### Backup and Restore

```bash
# Backup Helm values
helm get values orchestrator -n orchestrator > orchestrator-backup-values.yaml

# Backup all resources
kubectl get all,configmap,secret,ingress -n orchestrator -o yaml > orchestrator-backup.yaml

# Restore if needed
helm install orchestrator ./infra/orchestrator-chart \
  --namespace orchestrator \
  --values orchestrator-backup-values.yaml
```

## Security Considerations

1. **API Keys**: Store in Kubernetes secrets, never in values files
2. **RBAC**: Use namespaced roles when possible
3. **Network Policies**: Consider implementing network policies for pod-to-pod communication
4. **Image Security**: Use specific image tags, not `latest` in production
5. **TLS**: Enable TLS for ingress in production environments

## Performance Tuning

1. **Resources**: Adjust CPU/memory limits based on workload
2. **Replicas**: Consider running multiple replicas for high availability
3. **Storage**: Use fast storage classes for shared workspace PVC
4. **Node Affinity**: Pin orchestrator to specific nodes if needed