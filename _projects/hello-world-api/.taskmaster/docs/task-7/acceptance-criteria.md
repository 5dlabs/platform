# Task 7: Create Kubernetes Deployment Manifests - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. Directory Structure ✓
- [ ] Directory `kubernetes/` exists
- [ ] All manifest files in kubernetes directory
- [ ] Scripts directory contains deployment script

### 2. Deployment Manifest (kubernetes/deployment.yaml) ✓
- [ ] Uses apiVersion: apps/v1
- [ ] Kind is Deployment
- [ ] Metadata includes name and labels
- [ ] Spec defines:
  - [ ] replicas: 2
  - [ ] selector with matchLabels
  - [ ] strategy type: RollingUpdate
  - [ ] maxSurge: 1
  - [ ] maxUnavailable: 0
- [ ] Template spec includes:
  - [ ] Container name: hello-world-api
  - [ ] Image: hello-world-api:latest
  - [ ] imagePullPolicy: IfNotPresent
  - [ ] containerPort: 3000
  - [ ] Environment variables (NODE_ENV, PORT)
  - [ ] Resource limits (cpu: 0.2, memory: 256Mi)
  - [ ] Resource requests (cpu: 0.1, memory: 128Mi)
  - [ ] Readiness probe configured
  - [ ] Liveness probe configured

### 3. Service Manifest (kubernetes/service.yaml) ✓
- [ ] Uses apiVersion: v1
- [ ] Kind is Service
- [ ] Metadata includes name and labels
- [ ] Spec defines:
  - [ ] type: ClusterIP
  - [ ] port: 80
  - [ ] targetPort: 3000
  - [ ] protocol: TCP
  - [ ] name: http
  - [ ] selector matches deployment labels

### 4. Ingress Manifest (kubernetes/ingress.yaml) ✓
- [ ] Uses apiVersion: networking.k8s.io/v1
- [ ] Kind is Ingress
- [ ] Metadata includes name
- [ ] Annotations for nginx ingress
- [ ] Spec defines:
  - [ ] Host rule (hello-api.example.com)
  - [ ] Path: /
  - [ ] pathType: Prefix
  - [ ] Backend service name and port

### 5. ConfigMap Manifest (kubernetes/configmap.yaml) ✓
- [ ] Uses apiVersion: v1
- [ ] Kind is ConfigMap
- [ ] Metadata includes name
- [ ] Data section contains:
  - [ ] NODE_ENV: "production"
  - [ ] PORT: "3000"

### 6. Deployment Script ✓
- [ ] File `scripts/k8s-deploy.sh` exists
- [ ] Script is executable (chmod +x)
- [ ] Applies manifests in correct order:
  1. [ ] ConfigMap
  2. [ ] Deployment
  3. [ ] Service
  4. [ ] Ingress
- [ ] Waits for rollout completion
- [ ] Displays success message

## Test Cases

### Test Case 1: Manifest Validation
```bash
# Validate YAML syntax
kubectl apply --dry-run=client -f kubernetes/deployment.yaml
kubectl apply --dry-run=client -f kubernetes/service.yaml
kubectl apply --dry-run=client -f kubernetes/ingress.yaml
kubectl apply --dry-run=client -f kubernetes/configmap.yaml
```
**Expected:** All commands succeed with no errors

### Test Case 2: Deployment Creation
```bash
kubectl apply -f kubernetes/deployment.yaml
kubectl get deployment hello-world-api
```
**Expected Output:**
```
NAME              READY   UP-TO-DATE   AVAILABLE   AGE
hello-world-api   2/2     2            2           30s
```

### Test Case 3: Pod Health Checks
```bash
kubectl get pods -l app=hello-world-api
kubectl describe pod <pod-name> | grep -E "Liveness:|Readiness:"
```
**Expected:** 
- 2 pods in Running state
- Both probes configured with http-get to /health

### Test Case 4: Service Endpoints
```bash
kubectl apply -f kubernetes/service.yaml
kubectl get endpoints hello-world-api
```
**Expected:** Endpoints show 2 IP addresses (one per pod)

### Test Case 5: Ingress Configuration
```bash
kubectl apply -f kubernetes/ingress.yaml
kubectl describe ingress hello-world-api
```
**Expected:** 
- Host: hello-api.example.com
- Backend: hello-world-api:80

### Test Case 6: ConfigMap Data
```bash
kubectl apply -f kubernetes/configmap.yaml
kubectl get configmap hello-world-api-config -o yaml
```
**Expected:** Contains NODE_ENV and PORT values

### Test Case 7: Deployment Script Execution
```bash
./scripts/k8s-deploy.sh
```
**Expected Output:**
```
Applying Kubernetes manifests...
configmap/hello-world-api-config created
deployment.apps/hello-world-api created
service/hello-world-api created
ingress.networking.k8s.io/hello-world-api created
Waiting for deployment to be ready...
deployment "hello-world-api" successfully rolled out
Deployment complete!
Service available at: http://hello-api.example.com
```

## Validation Commands

### Resource Verification
```bash
# Check all resources
kubectl get all -l app=hello-world-api

# Verify labels
kubectl get deployment,service,pods -l app=hello-world-api --show-labels

# Check resource limits
kubectl describe pod <pod-name> | grep -A10 "Limits:"
```

### Health Check Testing
```bash
# Port forward to test locally
kubectl port-forward service/hello-world-api 8080:80

# In another terminal
curl http://localhost:8080/health
# Expected: {"status":"success","message":"Service is healthy"...}
```

### Rolling Update Test
```bash
# Trigger a rolling update
kubectl set env deployment/hello-world-api TEST_VAR=test

# Watch the rollout
kubectl rollout status deployment/hello-world-api

# Verify zero downtime
while true; do curl -s http://localhost:8080/health | grep -q "success" && echo "OK" || echo "FAIL"; sleep 1; done
```

## Success Indicators
- ✅ All manifests apply without errors
- ✅ 2 pods running and healthy
- ✅ Service has 2 endpoints
- ✅ Ingress configured correctly
- ✅ ConfigMap created successfully
- ✅ Deployment script runs without errors
- ✅ Health checks passing
- ✅ Resource limits enforced
- ✅ Zero downtime during updates

## Common Issues and Solutions

### Issue 1: Pods stuck in Pending
**Debug:**
```bash
kubectl describe pod <pod-name>
kubectl get events --sort-by=.metadata.creationTimestamp
```
**Common Causes:**
- Insufficient cluster resources
- Image pull errors

### Issue 2: Readiness probe failing
**Debug:**
```bash
kubectl logs <pod-name>
kubectl exec <pod-name> -- wget -O- http://localhost:3000/health
```
**Solution:** Verify application starts correctly and /health endpoint works

### Issue 3: Service has no endpoints
**Debug:**
```bash
kubectl get pods -l app=hello-world-api
kubectl get endpoints hello-world-api
```
**Solution:** Ensure pod labels match service selector

### Issue 4: Ingress not routing traffic
**Debug:**
```bash
kubectl get ingress hello-world-api
kubectl describe ingress hello-world-api
```
**Solution:** Verify ingress controller is installed and running

## Performance Validation

### Resource Usage Check
```bash
# Check actual resource usage
kubectl top pods -l app=hello-world-api

# Verify within limits
# CPU should be < 200m
# Memory should be < 256Mi
```

### Scaling Test
```bash
# Scale up
kubectl scale deployment hello-world-api --replicas=3

# Verify
kubectl get pods -l app=hello-world-api

# Scale down
kubectl scale deployment hello-world-api --replicas=2
```

## Production Readiness Checklist
- [ ] Image uses registry URL (not local)
- [ ] Ingress host uses real domain
- [ ] TLS configured on ingress
- [ ] Resource limits appropriate for workload
- [ ] Namespace specified in manifests
- [ ] RBAC policies defined
- [ ] Network policies configured
- [ ] Pod disruption budget created
- [ ] Monitoring annotations added

## Manual Testing Procedure
1. **Deploy all resources:**
   ```bash
   ./scripts/k8s-deploy.sh
   ```

2. **Verify deployment health:**
   ```bash
   kubectl get deployment hello-world-api
   kubectl get pods -l app=hello-world-api
   ```

3. **Test service connectivity:**
   ```bash
   kubectl port-forward service/hello-world-api 8080:80
   curl http://localhost:8080/health
   curl http://localhost:8080/hello
   ```

4. **Simulate pod failure:**
   ```bash
   kubectl delete pod <pod-name>
   # Verify new pod created and service still works
   ```

5. **Test rolling update:**
   ```bash
   kubectl set image deployment/hello-world-api hello-world-api=hello-world-api:latest
   kubectl rollout status deployment/hello-world-api
   ```