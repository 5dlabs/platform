# Task 10: Deployment and Documentation - AI Agent Prompt

You are a senior DevOps engineer tasked with preparing a chat application for production deployment. Your implementation must include containerization with Docker, orchestration with Kubernetes, CI/CD pipelines, comprehensive API documentation, user guides, and production-grade monitoring solutions.

## Primary Objectives

1. **Containerization**: Create production-ready Docker configurations with multi-stage builds, security best practices, and optimized images.

2. **Kubernetes Deployment**: Develop comprehensive Kubernetes manifests including deployments, services, ingress, autoscaling, and monitoring.

3. **CI/CD Pipeline**: Implement automated testing and deployment workflows using GitHub Actions or similar platforms.

4. **API Documentation**: Create OpenAPI/Swagger specifications documenting all endpoints, authentication flows, and WebSocket events.

5. **Monitoring Infrastructure**: Set up comprehensive monitoring, logging, and error tracking for production observability.

## Required Actions

### Phase 1: Research Deployment Patterns (15 minutes)
1. Research best practices:
   ```bash
   # Research Kubernetes patterns
   # Study microservices deployment architectures
   # Analyze Rust ecosystem deployment strategies
   # Review modern CI/CD patterns
   ```

2. Document findings:
   - Container optimization techniques
   - Kubernetes scaling patterns
   - Service mesh considerations
   - GitOps methodologies

3. Security research:
   - Container security scanning
   - Secret management
   - Network policies
   - RBAC configurations

### Phase 2: Docker Configuration (20 minutes)
1. Frontend Dockerfile:
   - Multi-stage build
   - Nginx optimization
   - Security hardening
   - Health checks

2. Backend Dockerfile:
   - Node.js optimization
   - Non-root user
   - Signal handling
   - Layer caching

3. Docker Compose:
   ```yaml
   version: '3.8'
   services:
     frontend:
       build: ./frontend
       ports: ["80:80"]
     backend:
       build: ./backend
       ports: ["3000:3000"]
     postgres:
       image: postgres:14-alpine
     redis:
       image: redis:7-alpine
   ```

### Phase 3: Kubernetes Manifests (25 minutes)
1. Core resources:
   - Deployments with proper resource limits
   - Services for internal communication
   - ConfigMaps for configuration
   - Secrets for sensitive data

2. Ingress configuration:
   - TLS termination
   - WebSocket support
   - Rate limiting
   - Path-based routing

3. Autoscaling:
   - Horizontal Pod Autoscaler
   - Vertical Pod Autoscaler
   - Cluster autoscaling
   - Resource quotas

4. Monitoring integration:
   - Prometheus annotations
   - Service monitors
   - Grafana dashboards
   - Alert rules

### Phase 4: CI/CD Pipeline (20 minutes)
1. GitHub Actions workflow:
   ```yaml
   name: Build and Deploy
   on:
     push:
       branches: [main]
   jobs:
     test:
       - Unit tests
       - Integration tests
       - E2E tests
     build:
       - Docker build
       - Security scanning
       - Image push
     deploy:
       - Kubernetes apply
       - Health checks
       - Smoke tests
   ```

2. Environment management:
   - Development
   - Staging
   - Production
   - Rollback procedures

3. Quality gates:
   - Code coverage
   - Security scanning
   - Performance tests
   - Approval workflows

### Phase 5: API Documentation (15 minutes)
1. OpenAPI specification:
   - All endpoints documented
   - Request/response schemas
   - Authentication details
   - Error responses

2. WebSocket documentation:
   - Event types
   - Message formats
   - Connection handling
   - Error scenarios

3. Integration guides:
   - Authentication flow
   - Rate limiting
   - Pagination
   - File uploads

### Phase 6: Monitoring Setup (15 minutes)
1. Metrics collection:
   - Application metrics
   - Infrastructure metrics
   - Business metrics
   - Custom dashboards

2. Logging infrastructure:
   - Centralized logging
   - Log aggregation
   - Search capabilities
   - Retention policies

3. Error tracking:
   - Sentry integration
   - Error grouping
   - Alert configuration
   - Performance monitoring

## Implementation Details

### Docker Best Practices
```dockerfile
# Multi-stage build example
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:18-alpine
RUN apk add --no-cache tini
WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY . .
USER node
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["node", "server.js"]
```

### Kubernetes Production Config
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chat-backend
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      containers:
      - name: backend
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
          initialDelaySeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
          initialDelaySeconds: 5
```

### CI/CD Security
```yaml
# Security scanning in pipeline
- name: Run Trivy security scan
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: ${{ env.IMAGE }}
    format: 'sarif'
    output: 'trivy-results.sarif'

- name: Upload scan results
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: 'trivy-results.sarif'
```

## Monitoring Requirements

### Key Metrics
- Request rate and latency
- Error rates by endpoint
- WebSocket connections
- Database query performance
- Cache hit rates
- Resource utilization

### Alerting Rules
```yaml
- alert: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
  annotations:
    summary: "High error rate detected"
    
- alert: PodMemoryUsage
  expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
  annotations:
    summary: "Pod memory usage above 90%"
```

## Documentation Standards

### API Documentation
- Clear endpoint descriptions
- Example requests/responses
- Error code explanations
- Rate limit information
- Authentication guides

### User Documentation
- Getting started guide
- Feature explanations
- Troubleshooting section
- FAQ
- Video tutorials (optional)

### Deployment Documentation
- Infrastructure requirements
- Deployment procedures
- Configuration management
- Disaster recovery
- Scaling guidelines

## Security Checklist

### Container Security
- [ ] Use minimal base images
- [ ] Run as non-root user
- [ ] Scan for vulnerabilities
- [ ] Sign container images
- [ ] Use read-only filesystems

### Kubernetes Security
- [ ] Enable RBAC
- [ ] Use network policies
- [ ] Encrypt secrets
- [ ] Enable pod security policies
- [ ] Regular security audits

### Application Security
- [ ] Environment-based config
- [ ] Secure secret management
- [ ] TLS everywhere
- [ ] Regular updates
- [ ] Security headers

## Performance Optimization

### Container Optimization
- Multi-stage builds
- Layer caching
- Minimal dependencies
- Compressed assets
- Health check optimization

### Kubernetes Optimization
- Resource limits
- Node affinity
- Pod disruption budgets
- Preemptible nodes
- Autoscaling policies

## Testing Strategy

### Pre-deployment Tests
```bash
# Run all tests
npm test
npm run test:integration
npm run test:e2e

# Security scanning
trivy image backend:latest
kubesec scan deployment.yaml

# Performance tests
k6 run loadtest.js
```

### Post-deployment Validation
- Health endpoint checks
- Smoke test suite
- Performance benchmarks
- Security validation
- User acceptance tests

## Rollout Strategy

### Blue-Green Deployment
```yaml
# Switch traffic between blue and green
kubectl patch service chat-backend -p '{"spec":{"selector":{"version":"green"}}}'
```

### Canary Deployment
```yaml
# Gradual rollout with Flagger
apiVersion: flagger.app/v1beta1
kind: Canary
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: chat-backend
  progressDeadlineSeconds: 600
  service:
    port: 3000
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
```

## Final Deliverables

Before marking complete:
- [ ] Docker images optimized and secure
- [ ] Kubernetes manifests production-ready
- [ ] CI/CD pipeline fully automated
- [ ] API documentation comprehensive
- [ ] User guide complete
- [ ] Monitoring dashboards configured
- [ ] Error tracking operational
- [ ] Security scanning passed
- [ ] Load testing successful
- [ ] Documentation reviewed

Execute this deployment systematically, ensuring production readiness, security compliance, and comprehensive documentation for both developers and end users.