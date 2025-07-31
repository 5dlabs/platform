# Task 7: Create Kubernetes Deployment Manifests

## Overview
This task creates production-ready Kubernetes manifests for deploying the Hello World API to a Kubernetes cluster. It includes deployment configuration, service exposure, ingress routing, configuration management, and automated deployment scripts.

## Objectives
- Create Kubernetes deployment with proper resource limits and health checks
- Configure service for internal cluster communication
- Set up ingress for external access
- Implement configuration management with ConfigMap
- Create automated deployment scripts
- Ensure high availability with multiple replicas
- Enable zero-downtime deployments

## Technical Approach

### Kubernetes Architecture
The deployment consists of four main components:
1. **Deployment**: Manages application pods with scaling and updates
2. **Service**: Provides stable network endpoint for pods
3. **Ingress**: Routes external traffic to the service
4. **ConfigMap**: Centralizes configuration management

### Deployment Strategy
- Rolling updates with zero downtime
- Health checks for reliability
- Resource limits for cluster stability
- Horizontal scaling capability

## Implementation Details

### Step 1: Create Deployment Manifest (kubernetes/deployment.yaml)
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hello-world-api
  labels:
    app: hello-world-api
spec:
  replicas: 2
  selector:
    matchLabels:
      app: hello-world-api
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: hello-world-api
    spec:
      containers:
      - name: hello-world-api
        image: hello-world-api:latest
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 3000
        env:
        - name: NODE_ENV
          value: "production"
        - name: PORT
          value: "3000"
        resources:
          limits:
            cpu: "0.2"
            memory: "256Mi"
          requests:
            cpu: "0.1"
            memory: "128Mi"
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 15
          periodSeconds: 20
```

### Step 2: Create Service Manifest (kubernetes/service.yaml)
```yaml
apiVersion: v1
kind: Service
metadata:
  name: hello-world-api
  labels:
    app: hello-world-api
spec:
  type: ClusterIP
  ports:
  - port: 80
    targetPort: 3000
    protocol: TCP
    name: http
  selector:
    app: hello-world-api
```

### Step 3: Create Ingress Manifest (kubernetes/ingress.yaml)
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: hello-world-api
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: hello-api.example.com  # Replace with actual domain
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: hello-world-api
            port:
              number: 80
```

### Step 4: Create ConfigMap Manifest (kubernetes/configmap.yaml)
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hello-world-api-config
data:
  NODE_ENV: "production"
  PORT: "3000"
```

### Step 5: Create Deployment Script (scripts/k8s-deploy.sh)
```bash
#!/bin/bash
set -e

echo "Applying Kubernetes manifests..."
kubectl apply -f kubernetes/configmap.yaml
kubectl apply -f kubernetes/deployment.yaml
kubectl apply -f kubernetes/service.yaml
kubectl apply -f kubernetes/ingress.yaml

echo "Waiting for deployment to be ready..."
kubectl rollout status deployment/hello-world-api

echo "Deployment complete!"
echo "Service available at: http://hello-api.example.com"
```

### Step 6: Set Script Permissions
```bash
chmod +x scripts/k8s-deploy.sh
```

### Key Configuration Details

#### Deployment Configuration
- **Replicas**: 2 for high availability
- **Strategy**: RollingUpdate for zero-downtime deployments
- **maxSurge**: 1 (one extra pod during updates)
- **maxUnavailable**: 0 (no downtime during updates)

#### Resource Management
- **CPU Limits**: 200m (0.2 CPU cores)
- **Memory Limits**: 256Mi
- **CPU Requests**: 100m (0.1 CPU cores)
- **Memory Requests**: 128Mi

#### Health Checks
- **Readiness Probe**: Determines when pod is ready for traffic
  - Initial delay: 5 seconds
  - Check interval: 10 seconds
- **Liveness Probe**: Restarts pod if unhealthy
  - Initial delay: 15 seconds
  - Check interval: 20 seconds

#### Service Configuration
- **Type**: ClusterIP (internal only)
- **Port**: 80 (service port)
- **TargetPort**: 3000 (container port)

## Dependencies and Requirements
- Task 6 must be completed (Docker image created)
- Kubernetes cluster available
- kubectl configured with cluster access
- Ingress controller installed in cluster (e.g., nginx-ingress)
- DNS configured for ingress host (or use /etc/hosts for testing)

## Deployment Process

### Initial Deployment
```bash
# Create kubernetes directory
mkdir -p kubernetes

# Apply all manifests
./scripts/k8s-deploy.sh

# Verify deployment
kubectl get deployments
kubectl get pods
kubectl get services
kubectl get ingress
```

### Scaling Operations
```bash
# Scale up
kubectl scale deployment hello-world-api --replicas=3

# Scale down
kubectl scale deployment hello-world-api --replicas=1

# Autoscaling (optional)
kubectl autoscale deployment hello-world-api --min=2 --max=5 --cpu-percent=80
```

### Update Process
```bash
# Update image
kubectl set image deployment/hello-world-api hello-world-api=hello-world-api:v2

# Watch rollout
kubectl rollout status deployment/hello-world-api

# Rollback if needed
kubectl rollout undo deployment/hello-world-api
```

## Testing Strategy

### Deployment Verification
```bash
# Check deployment status
kubectl describe deployment hello-world-api

# Check pod status
kubectl get pods -l app=hello-world-api

# Check pod logs
kubectl logs -l app=hello-world-api

# Check service endpoints
kubectl get endpoints hello-world-api
```

### Health Check Testing
```bash
# Test service internally
kubectl run test-pod --rm -it --image=busybox --restart=Never -- wget -O- http://hello-world-api/health

# Port forward for local testing
kubectl port-forward service/hello-world-api 8080:80
curl http://localhost:8080/health
```

### Ingress Testing
```bash
# Check ingress status
kubectl describe ingress hello-world-api

# Test with curl (requires DNS or /etc/hosts entry)
curl http://hello-api.example.com/health
```

## Success Criteria
- Deployment creates 2 healthy pods
- Pods pass readiness and liveness probes
- Service routes traffic to healthy pods
- Ingress accepts external traffic
- Rolling updates complete without downtime
- Resource limits are enforced
- ConfigMap values are applied
- Deployment script executes successfully

## Troubleshooting Guide

### Issue: Pods not starting
```bash
# Check pod events
kubectl describe pod <pod-name>

# Check logs
kubectl logs <pod-name>

# Common causes:
# - Image not found (check imagePullPolicy)
# - Resource limits too low
# - Health check failing
```

### Issue: Service not reachable
```bash
# Check endpoints
kubectl get endpoints hello-world-api

# Test from within cluster
kubectl run debug --rm -it --image=busybox --restart=Never -- sh
# Then: wget -O- http://hello-world-api/health
```

### Issue: Ingress not working
```bash
# Check ingress controller
kubectl get pods -n ingress-nginx

# Check ingress events
kubectl describe ingress hello-world-api

# Verify DNS resolution
nslookup hello-api.example.com
```

## Production Considerations

### Security
```yaml
# Add security context to deployment
securityContext:
  runAsNonRoot: true
  runAsUser: 1001
  fsGroup: 1001
```

### Monitoring
```yaml
# Add Prometheus annotations
metadata:
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "3000"
    prometheus.io/path: "/metrics"
```

### Resource Optimization
- Monitor actual resource usage
- Adjust limits based on metrics
- Consider vertical pod autoscaling
- Use pod disruption budgets

## Related Tasks
- Task 6: Docker Image Creation (provides container image)
- Task 3: API Endpoints (health check endpoint)
- Task 5: Testing Suite (tests should work with deployed app)
- Task 8: CI/CD Pipeline (will deploy using these manifests)