# Task 7: Create Kubernetes Deployment Manifests - Autonomous Agent Prompt

You are an experienced Kubernetes engineer tasked with creating production-ready deployment manifests for the Hello World API. You need to create all necessary Kubernetes resources for a reliable, scalable deployment.

## Your Mission
Create Kubernetes manifests for deployment, service, ingress, and configuration, along with an automated deployment script. Ensure the deployment follows best practices for reliability, scalability, and zero-downtime updates.

## Detailed Instructions

### 1. Create Kubernetes Directory
First, create the directory structure:
```bash
mkdir -p kubernetes
```

### 2. Create Deployment Manifest (kubernetes/deployment.yaml)
Create a deployment manifest with the following specifications:

**Key Requirements:**
- 2 replicas for high availability
- Rolling update strategy with zero downtime
- Resource limits and requests
- Health checks (readiness and liveness probes)
- Environment variables

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

**Important Configuration Points:**
- `maxUnavailable: 0` ensures zero downtime during updates
- `imagePullPolicy: IfNotPresent` for local development
- Resource requests ensure pod scheduling
- Resource limits prevent resource exhaustion
- Probes use the /health endpoint from Task 3

### 3. Create Service Manifest (kubernetes/service.yaml)
Create a service to provide stable networking:

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

**Key Points:**
- ClusterIP type for internal access
- Port 80 for standard HTTP
- Targets container port 3000
- Selector matches deployment labels

### 4. Create Ingress Manifest (kubernetes/ingress.yaml)
Create an ingress for external access:

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

**Configuration Notes:**
- Uses nginx ingress controller annotations
- Host should be replaced with actual domain
- PathType: Prefix matches all paths
- Routes to the service on port 80

### 5. Create ConfigMap Manifest (kubernetes/configmap.yaml)
Create a ConfigMap for configuration:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hello-world-api-config
data:
  NODE_ENV: "production"
  PORT: "3000"
```

**Purpose:**
- Centralizes configuration
- Can be updated without rebuilding image
- Can be mounted as environment variables

### 6. Create Deployment Script (scripts/k8s-deploy.sh)
Create an automated deployment script:

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

### 7. Make Script Executable
```bash
chmod +x scripts/k8s-deploy.sh
```

## Testing Your Implementation

### Local Testing with Minikube/Kind
```bash
# Start local cluster (if using minikube)
minikube start

# Build and load image
docker build -t hello-world-api:latest .
minikube image load hello-world-api:latest  # For minikube
# OR
kind load docker-image hello-world-api:latest  # For kind

# Deploy
./scripts/k8s-deploy.sh

# Check deployment
kubectl get all -l app=hello-world-api

# Test with port-forward
kubectl port-forward service/hello-world-api 8080:80
curl http://localhost:8080/health
```

### Verification Commands
```bash
# Check deployment status
kubectl get deployment hello-world-api
# Should show READY 2/2

# Check pods
kubectl get pods -l app=hello-world-api
# Should show 2 pods Running

# Check service
kubectl get service hello-world-api
# Should show ClusterIP with port 80

# Check ingress
kubectl get ingress hello-world-api
# Should show the configured host
```

## Expected Outputs

### Successful Deployment
```
NAME                               READY   STATUS    RESTARTS   AGE
pod/hello-world-api-xxxxx-xxxxx   1/1     Running   0          1m
pod/hello-world-api-xxxxx-xxxxx   1/1     Running   0          1m

NAME                      TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)   AGE
service/hello-world-api   ClusterIP   10.96.xxx.xxx   <none>        80/TCP    1m

NAME                          READY   UP-TO-DATE   AVAILABLE   AGE
deployment/hello-world-api    2/2     2            2           1m
```

### Health Check Verification
```bash
# Check readiness probe
kubectl describe pod <pod-name> | grep -A5 "Readiness:"
# Should show: http-get http://:3000/health

# Check liveness probe
kubectl describe pod <pod-name> | grep -A5 "Liveness:"
# Should show: http-get http://:3000/health
```

## Common Issues and Solutions

### Issue 1: ImagePullBackOff
**Solution:** Ensure image exists locally or in registry
```bash
docker images | grep hello-world-api
# If using minikube: minikube image load hello-world-api:latest
```

### Issue 2: CrashLoopBackOff
**Solution:** Check pod logs
```bash
kubectl logs <pod-name>
kubectl describe pod <pod-name>
```

### Issue 3: Ingress not working
**Solution:** Ensure ingress controller is installed
```bash
# For minikube
minikube addons enable ingress

# Check ingress controller
kubectl get pods -n ingress-nginx
```

### Issue 4: Service not accessible
**Solution:** Verify endpoints
```bash
kubectl get endpoints hello-world-api
# Should list pod IPs
```

## Best Practices Checklist
- [ ] Deployment uses multiple replicas
- [ ] Rolling update configured for zero downtime
- [ ] Resource limits and requests defined
- [ ] Health checks configured
- [ ] Service uses appropriate type (ClusterIP)
- [ ] Ingress configured with proper annotations
- [ ] ConfigMap separates configuration
- [ ] Labels consistent across resources
- [ ] Deployment script includes status check

## Production Modifications

For production use, modify these items:
1. Change image to use registry URL
2. Update ingress host to real domain
3. Consider adding TLS to ingress
4. Use ConfigMap references in deployment
5. Add namespace definitions
6. Consider adding HorizontalPodAutoscaler

Complete this task by creating all manifest files and the deployment script exactly as specified. The deployment should be ready for both local testing and production use with minimal modifications.