# Autonomous Agent Prompt: Deployment and Documentation

You are tasked with preparing the chat application for production deployment with comprehensive containerization, Kubernetes orchestration, CI/CD pipelines, and complete documentation.

## Objective
Create a production-ready deployment setup with Docker multi-stage builds, Kubernetes manifests, automated CI/CD pipelines, and comprehensive API and user documentation. Research deployment patterns from multiple sources including Rust ecosystem best practices.

## Detailed Requirements

### 1. Research Phase
Before implementation, research:
- Kubernetes deployment patterns and service configurations
- Best practices from Rust ecosystem for deployment and monitoring
- Modern microservices deployment architectures
- CI/CD pipeline patterns and automation strategies
- Container optimization techniques
- Production monitoring approaches

Document findings to inform implementation decisions.

### 2. Docker Configuration
Create multi-stage builds:
- **Frontend**: Build stage + nginx serving
- **Backend**: Build stage + minimal runtime
- Security hardening (non-root user)
- Health checks for both services
- Optimized layer caching
- Minimal final image size

### 3. Nginx Configuration
Configure nginx for frontend:
- Gzip compression
- Security headers (CSP, HSTS, etc.)
- Static asset caching
- API and WebSocket proxying
- SPA routing support
- Health check endpoint

### 4. Kubernetes Manifests
Create comprehensive K8s configs:
- Namespace with proper labels
- ConfigMaps for environment variables
- Secrets for sensitive data
- Deployments with resource limits
- Services for internal networking
- Ingress with TLS support
- Horizontal Pod Autoscaler
- Network policies (optional)
- PodDisruptionBudget (optional)

### 5. CI/CD Pipeline
Implement GitHub Actions workflow:
- Multi-stage pipeline (test, build, deploy)
- Parallel job execution
- Service containers for testing
- Docker image building and pushing
- Automated Kubernetes deployment
- Environment-specific deployments
- Rollback capabilities

### 6. API Documentation
Create OpenAPI/Swagger spec:
- All endpoints documented
- Request/response schemas
- Authentication flows
- Error responses
- WebSocket events
- Example requests
- Interactive documentation

### 7. User Documentation
Write comprehensive guides:
- Quick start guide
- Feature descriptions
- Configuration options
- Deployment instructions
- Troubleshooting guide
- Contributing guidelines
- Architecture overview

### 8. Monitoring Setup
Configure observability:
- Prometheus metrics collection
- Grafana dashboards
- Log aggregation setup
- Error tracking (Sentry)
- Uptime monitoring
- Performance alerts

### 9. Security Hardening
Implement security measures:
- Container security scanning
- Least privilege principles
- Network segmentation
- Secret management
- TLS everywhere
- Security headers
- Rate limiting

### 10. Performance Optimization
Optimize for production:
- Multi-stage builds for size
- Layer caching strategies
- Resource limits and requests
- Autoscaling policies
- CDN integration
- Database connection pooling

## Expected Deliverables

1. Dockerfile.production for frontend and backend
2. nginx.conf with optimizations
3. Complete Kubernetes manifests set
4. GitHub Actions workflow file
5. OpenAPI specification
6. README.md with full documentation
7. Monitoring configuration files
8. docker-compose.yml for local deployment
9. Deployment scripts
10. Security checklist

## Kubernetes Structure

```
kubernetes/
├── 00-namespace.yaml
├── 01-configmap.yaml
├── 02-secrets.yaml
├── 10-backend-deployment.yaml
├── 11-backend-service.yaml
├── 20-frontend-deployment.yaml
├── 21-frontend-service.yaml
├── 30-ingress.yaml
├── 40-hpa.yaml
├── 50-postgres.yaml
├── 51-redis.yaml
└── monitoring/
    ├── prometheus-config.yaml
    └── grafana-dashboard.json
```

## CI/CD Pipeline Stages

1. **Test Stage**
   - Run unit tests
   - Run integration tests
   - Code coverage analysis
   - Security scanning

2. **Build Stage**
   - Build Docker images
   - Tag with version/commit
   - Push to registry
   - Image scanning

3. **Deploy Stage**
   - Update K8s manifests
   - Apply to cluster
   - Wait for rollout
   - Run smoke tests

## Documentation Standards

### API Documentation
- Use OpenAPI 3.0 specification
- Include authentication details
- Provide request/response examples
- Document error codes
- Explain rate limits

### User Documentation
- Clear installation steps
- Screenshots where helpful
- Troubleshooting section
- FAQ section
- Video tutorials (optional)

## Monitoring Requirements

### Metrics to Track
- Request rate and latency
- Error rates by endpoint
- Active connections
- Message throughput
- Resource utilization
- Cache hit rates

### Dashboards
- System overview
- API performance
- WebSocket metrics
- User activity
- Error tracking
- Resource usage

## Security Checklist

- [ ] Containers run as non-root
- [ ] Secrets not in images
- [ ] Network policies defined
- [ ] TLS certificates configured
- [ ] Security headers enabled
- [ ] Rate limiting active
- [ ] Vulnerability scanning
- [ ] Access controls implemented

## Testing the Deployment

1. **Local Testing**
   ```bash
   docker-compose up
   # Verify all services start
   # Test basic functionality
   ```

2. **Kubernetes Testing**
   ```bash
   kubectl apply -f kubernetes/
   kubectl get pods -n chat-app
   # Verify pods are running
   # Test ingress access
   ```

3. **Load Testing**
   ```bash
   # Use Artillery or K6
   # Test with 1000+ users
   # Monitor resource usage
   ```

Begin by researching deployment patterns, then create Docker configurations, followed by Kubernetes manifests, and finally documentation and monitoring setup.