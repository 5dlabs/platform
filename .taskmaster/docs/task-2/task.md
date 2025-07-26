# Task 2: Implement Toolman Kubernetes Deployment

## Overview
Review and customize the existing Kubernetes deployment manifest for Toolman to ensure it meets our specific orchestrator requirements. The deployment manifest is already comprehensive but needs validation and potential customization for our production environment.

## Context
Building on Task 1's Helm chart review, this task focuses specifically on the deployment manifest that will run Toolman in our Kubernetes cluster. The deployment is the core component that runs the Toolman HTTP proxy service, enabling Claude agents to access various MCP tools.

## Objectives
1. Thoroughly review the existing deployment.yaml template
2. Validate all deployment features and configurations
3. Ensure proper integration with ConfigMap for MCP servers
4. Optimize resource allocations for our expected workload
5. Verify security contexts and pod security standards
6. Configure high availability with appropriate replica settings

## Technical Deep Dive

### Current Deployment Features
The existing deployment includes sophisticated configurations:

1. **Container Configuration**
   - Main toolman container with ghcr.io/5dlabs/toolman image
   - Optional Docker-in-Docker (DinD) sidecar for Docker-based MCP servers
   - Init container for directory setup and permissions

2. **Volume Management**
   - Config volume for MCP server definitions
   - Temp directory for runtime operations
   - Docker socket mount (when DinD enabled)
   - Persistent storage for stateful data

3. **Security Configuration**
   - Non-root user execution (UID 1001, GID 2375)
   - Security contexts properly configured
   - Read-only root filesystem where applicable

4. **Environment Configuration**
   - Direct environment variables (PORT, PROJECT_DIR, RUST_LOG)
   - SecretRef for sensitive configurations
   - Proper configuration injection patterns

5. **Health Management**
   - Liveness probe configuration
   - Readiness probe configuration
   - Proper startup delays and intervals

### Key Integration Points

1. **ConfigMap Integration**
   ```yaml
   volumes:
   - name: config
     configMap:
       name: toolman-servers-config  # Critical: contains MCP server definitions
   ```

2. **Service Discovery**
   - Deployment labels must match Service selector
   - Proper pod naming for DNS resolution
   - Network policy compatibility

3. **Resource Management**
   - CPU/Memory requests for guaranteed QoS
   - Limits to prevent resource exhaustion
   - Horizontal Pod Autoscaling readiness

## Implementation Guide

### Step 1: Deployment Template Analysis
```yaml
# Key sections to review in deployment.yaml:

apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "toolman.fullname" . }}
  labels:
    {{- include "toolman.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}  # Ensure HA with 2+
  selector:
    matchLabels:
      {{- include "toolman.selectorLabels" . | nindent 6 }}
  template:
    spec:
      # Security Context
      securityContext:
        runAsUser: 1001
        runAsGroup: 2375
        fsGroup: 2375
      
      # Init Container
      initContainers:
      - name: setup
        image: busybox
        command: ['sh', '-c', 'mkdir -p /data/projects && chown -R 1001:2375 /data']
        
      # Main Container
      containers:
      - name: toolman
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        ports:
        - containerPort: 3000
          name: http
        
        # Environment Variables
        env:
        - name: PORT
          value: "3000"
        - name: PROJECT_DIR
          value: "/data/projects"
        - name: RUST_LOG
          value: "{{ .Values.logLevel | default "info" }}"
        
        # Volume Mounts
        volumeMounts:
        - name: config
          mountPath: /config
        - name: data
          mountPath: /data
        - name: tmp
          mountPath: /tmp
        
        # Probes
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
        
        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
        
        # Resources
        resources:
          {{- toYaml .Values.resources | nindent 10 }}
```

### Step 2: Resource Optimization
```yaml
# Recommended resources for production:
resources:
  requests:
    cpu: 500m      # Base CPU for proxy operations
    memory: 512Mi  # Base memory for Rust application
  limits:
    cpu: 2000m     # Allow bursts for multiple MCP operations
    memory: 2Gi    # Headroom for concurrent connections
```

### Step 3: High Availability Configuration
```yaml
# Anti-affinity for pod distribution:
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchExpressions:
          - key: app.kubernetes.io/name
            operator: In
            values:
            - toolman
        topologyKey: kubernetes.io/hostname
```

### Step 4: ConfigMap Volume Validation
```yaml
# Ensure proper ConfigMap mounting:
volumes:
- name: config
  configMap:
    name: toolman-servers-config
    items:
    - key: servers-config.json
      path: servers-config.json
```

### Step 5: Docker-in-Docker Sidecar (if needed)
```yaml
# DinD sidecar for Docker-based MCP servers:
- name: dind
  image: docker:dind
  securityContext:
    privileged: true  # Required for DinD
  env:
  - name: DOCKER_TLS_CERTDIR
    value: ""
  volumeMounts:
  - name: docker-socket
    mountPath: /var/run
```

## Testing Strategy

### 1. Deployment Validation
```bash
# Deploy to test namespace
kubectl create namespace toolman-test
helm install toolman ./toolman/charts/toolman/ \
  -n toolman-test \
  -f custom-values.yaml

# Verify deployment
kubectl get deployment toolman -n toolman-test -o yaml
kubectl describe deployment toolman -n toolman-test
```

### 2. Pod Health Verification
```bash
# Check pod status
kubectl get pods -n toolman-test -l app.kubernetes.io/name=toolman

# Verify all containers running
kubectl get pods -n toolman-test -o jsonpath='{.items[*].status.containerStatuses[*].ready}'

# Check logs
kubectl logs -n toolman-test deployment/toolman --all-containers=true
```

### 3. Volume Mount Testing
```bash
# Verify ConfigMap mounted correctly
kubectl exec -n toolman-test deployment/toolman -- ls -la /config/
kubectl exec -n toolman-test deployment/toolman -- cat /config/servers-config.json

# Check persistent volume
kubectl exec -n toolman-test deployment/toolman -- df -h /data
```

### 4. Resource Usage Monitoring
```bash
# Monitor resource consumption
kubectl top pod -n toolman-test -l app.kubernetes.io/name=toolman

# Watch for restarts or OOM kills
kubectl get events -n toolman-test --field-selector involvedObject.kind=Pod
```

## Success Criteria
1. ✅ Deployment creates specified number of replicas
2. ✅ All containers start and remain healthy
3. ✅ ConfigMap properly mounted and accessible
4. ✅ Health checks passing consistently
5. ✅ Resource usage within defined limits
6. ✅ No pod restarts or failures
7. ✅ Anti-affinity rules distributing pods

## Customization Recommendations

### 1. Production Values Override
```yaml
# production-values.yaml
replicaCount: 3  # For high availability

image:
  pullPolicy: IfNotPresent
  # Consider using specific version tags instead of 'latest'
  tag: "v1.0.0"

resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: 2000m
    memory: 2Gi

# Enable persistence for production
persistence:
  enabled: true
  size: 20Gi
  storageClass: "fast-ssd"

# Production logging
logLevel: "info"  # or "debug" for troubleshooting

# Node selection for dedicated nodes
nodeSelector:
  workload-type: "tools"

# Tolerations if using tainted nodes
tolerations:
- key: "tools-only"
  operator: "Equal"
  value: "true"
  effect: "NoSchedule"
```

### 2. Security Enhancements
```yaml
# Additional security contexts
securityContext:
  runAsNonRoot: true
  runAsUser: 1001
  runAsGroup: 2375
  fsGroup: 2375
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
```

## Risk Mitigation

1. **Resource Exhaustion**
   - Set appropriate limits
   - Monitor usage patterns
   - Implement HPA if needed

2. **ConfigMap Changes**
   - Use ConfigMap reloader
   - Implement graceful reload
   - Version ConfigMaps

3. **Pod Disruptions**
   - Set PodDisruptionBudget
   - Use preStop hooks
   - Implement graceful shutdown

## Dependencies
- Task 1: Helm chart structure must be reviewed
- Task 3: ConfigMap template must align with deployment
- Task 16: Note mentions ConfigMap structure (though task 16 doesn't exist in our list)

## Related Tasks
- Task 3: Service and ConfigMap creation
- Task 13: Final deployment and testing
- Task 5: Docs agent integration with deployed service