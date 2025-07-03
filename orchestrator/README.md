# Orchestrator

A Rust-based unified orchestration service that processes requests from multiple sources (GitHub, PM Agent, Grafana, CLI) and orchestrates Kubernetes Jobs for AI agent task execution.

## Features

- **GitHub Webhook Processing**: Handles GitHub webhook events (issues, pull requests, etc.)
- **Kubernetes Integration**: Creates and manages Kubernetes Jobs for code processing
- **Health Monitoring**: Built-in health check endpoint
- **Graceful Shutdown**: Proper signal handling for clean container shutdown
- **Security**: Runs as non-root user with minimal privileges
- **Observability**: Structured logging with configurable levels

## Architecture

The orchestrator follows a modular architecture:

1. **Webhook Receiver**: Axum-based HTTP server receives GitHub webhooks
2. **Event Parser**: Parses GitHub webhook payloads and extracts relevant information
3. **Job Orchestrator**: Creates Kubernetes Jobs with appropriate configurations
4. **Kubernetes Client**: Manages Job lifecycle and status monitoring

## Configuration

The controller uses environment variables and a default configuration:

### Server Configuration
- `SERVER_HOST`: Server bind address (default: `0.0.0.0`)
- `SERVER_PORT`: Server port (default: `8080`)

### Kubernetes Configuration
- `KUBERNETES_NAMESPACE`: Target namespace for Jobs (default: `default`)
- `JOB_IMAGE`: Container image for Jobs (default: `anthropic/claude-code:latest`)
- `JOB_TTL_SECONDS`: Job TTL after completion (default: `1800` - 30 minutes)
- `JOB_MEMORY_REQUEST`: Job memory request (default: `2Gi`)
- `JOB_MEMORY_LIMIT`: Job memory limit (default: `4Gi`)
- `JOB_CPU_REQUEST`: Job CPU request (default: `1`)
- `JOB_CPU_LIMIT`: Job CPU limit (default: `2`)

## Building

> **Note**: This orchestrator is a **pure Rust application** and does not require Node.js. The main project Dockerfile (`../Dockerfile`) is for the development environment that includes both Node.js and Rust.

### Local Development

#### Prerequisites
- Rust 1.75+
- Docker (for containerization)
- Kubernetes cluster (for testing)

#### Quick Build
```bash
# Build and test locally
cargo build --release
cargo test

# Run locally (requires Kubernetes access)
cargo run
```

#### Docker Build
```bash
# Simple build
./build.sh

# Build with specific tag
./build.sh -t v1.0.0

# Build and push to registry
./build.sh -r ghcr.io/yourusername -t v1.0.0 -p

# Build without running tests (faster)
./build.sh -b
```

### CI/CD

The project includes a GitHub Actions workflow that:

1. **Builds** multi-architecture Docker images (amd64, arm64)
2. **Tests** the code with cargo test, clippy, and fmt
3. **Scans** for security vulnerabilities with Trivy
4. **Publishes** to GitHub Container Registry (ghcr.io)

#### Triggering Builds

- **Push to main**: Builds and publishes with `latest` tag
- **Pull Requests**: Builds and tests without publishing
- **Releases**: Builds and publishes with semantic version tags

#### Image Tags

The CI system creates multiple tags:
- `latest`: Latest build from main branch
- `main-<sha>`: Specific commit from main
- `v1.2.3`: Semantic version tags from releases
- `pr-123`: Pull request builds (not published)

## Deployment

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orchestrator
spec:
  replicas: 2
  selector:
    matchLabels:
      app: orchestrator
  template:
    metadata:
      labels:
        app: orchestrator
    spec:
      containers:
      - name: orchestrator
        image: ghcr.io/yourusername/platform/orchestrator:latest
        ports:
        - containerPort: 8080
        env:
        - name: KUBERNETES_NAMESPACE
          value: "claude-jobs"
        - name: JOB_IMAGE
          value: "anthropic/claude-code:latest"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: orchestrator-service
spec:
  selector:
    app: orchestrator
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### Docker Compose (Development)

```yaml
version: '3.8'
services:
  orchestrator:
    image: orchestrator:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=orchestrator=debug
      - KUBERNETES_NAMESPACE=default
    # Note: Requires proper Kubernetes configuration
```

## API Endpoints

### Health Check
```
GET /health
```
Returns `200 OK` if the service is healthy.

### GitHub Webhook
```
POST /webhook/github
```
Processes GitHub webhook events. Expects GitHub webhook payload in request body.

## Development

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With logging
RUST_LOG=debug cargo test
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### Local Testing with ngrok

For testing GitHub webhooks locally:

```bash
# Install ngrok
# Start the controller
cargo run

# In another terminal, expose local port
ngrok http 8080

# Configure GitHub webhook URL to point to ngrok URL + /webhook/github
```

## Security Considerations

- **Container Security**: Runs as non-root user (UID 1000)
- **Resource Limits**: Configured memory and CPU limits
- **Network Security**: Only exposes necessary ports
- **Image Scanning**: Trivy security scanning in CI
- **Minimal Dependencies**: Uses distroless-style base image

## Monitoring and Observability

### Logging
- Structured logging with `tracing`
- Configurable log levels via `RUST_LOG`
- Request/response logging for HTTP endpoints

### Metrics
- Built-in health check endpoint
- Container metrics via Docker/Kubernetes
- Custom metrics can be added via prometheus crate

### Health Checks
- HTTP health endpoint at `/health`
- Kubernetes liveness and readiness probes
- Docker health check built into image

## Troubleshooting

### Common Issues

1. **Kubernetes Connection Failed**
   ```
   Error: Failed to initialize Kubernetes client
   ```
   - Ensure proper KUBECONFIG or in-cluster credentials
   - Check RBAC permissions for Job creation

2. **Port Already in Use**
   ```
   Error: Address already in use (os error 48)
   ```
   - Change port via `SERVER_PORT` environment variable
   - Check for conflicting services

3. **Job Creation Failed**
   ```
   Error: Failed to create Kubernetes Job
   ```
   - Verify namespace exists and is accessible
   - Check resource quotas and limits
   - Ensure job image is accessible

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=orchestrator=debug cargo run

# Or in Docker
docker run -e RUST_LOG=debug orchestrator:latest
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

The CI will automatically:
- Run tests and linting
- Build Docker images
- Scan for security issues
- Provide feedback on the PR

## License

[Add your license information here]