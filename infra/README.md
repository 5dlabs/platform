# Infrastructure Directory Structure

This directory contains all infrastructure configurations for the platform.

## Directory Organization

### `orchestrator-chart/`
The main Helm chart for deploying the orchestrator service. This includes:
- ServiceAccount and RBAC configuration
- ConfigMaps for configuration
- Deployment and Service definitions
- Ingress configuration
- Automatic per-service workspace PVCs

**Usage:**
```bash
helm install orchestrator ./orchestrator-chart \
  --namespace orchestrator \
  --create-namespace \
  --set secrets.anthropicApiKey="your-key"
```

### `crds/`
Custom Resource Definitions and related configurations:
- `taskrun-crd.yaml` - TaskRun CRD definition
- `taskrun-controller-config.yaml` - Controller configuration ConfigMap
- `test-taskrun.yaml` - Example TaskRun for testing

**Usage:**
```bash
kubectl apply -f crds/taskrun-crd.yaml
kubectl apply -f crds/taskrun-controller-config.yaml
```

### `telemetry/`
OpenTelemetry and monitoring configurations:
- `otel-collector/` - OpenTelemetry collector Helm chart
- `telemetry-dashboards/` - Grafana dashboard definitions

### `cluster-config/`
Cluster-specific configurations that are not part of Helm charts:
- `local-path-config-patch.yaml` - Local path provisioner configuration
- `talos-local-path-volume.yaml` - Talos-specific volume configuration
- `otel-collector-metrics-service.yaml` - Additional OTEL metrics service
- `otel-prometheus-service.yaml` - Prometheus metrics service

**Note:** These are typically one-time configurations or cluster-specific settings.

### `test-resources/`
Test manifests and simulators (not for production use):
- `simulators/` - Claude Code telemetry simulators
- `test-pods/` - Test pods for validation
- Various test jobs and configurations

### `manifests/` (Deprecated)
This directory previously contained individual Kubernetes manifests. These have been:
- Moved to appropriate Helm charts (orchestrator deployment)
- Relocated to `cluster-config/` (cluster-specific configs)
- Moved to `test-resources/` (test manifests)

## Deployment Order

1. **Install CRDs:**
   ```bash
   kubectl apply -f crds/taskrun-crd.yaml
   ```

2. **Deploy Orchestrator:**
   ```bash
   helm install orchestrator ./orchestrator-chart -n orchestrator --create-namespace
   kubectl apply -f crds/taskrun-controller-config.yaml
   ```

3. **Apply cluster configs (if needed):**
   ```bash
   kubectl apply -f cluster-config/
   ```

## Notes

- All production deployments should use Helm charts
- The orchestrator Helm chart includes all necessary RBAC permissions
- Test resources are kept separate from production configurations
- Dashboard configurations are managed through the telemetry-dashboards Helm chart