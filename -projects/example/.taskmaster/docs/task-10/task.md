# Task 10: Prepare Deployment Artifacts and Documentation

## Overview

This task focuses on creating production-ready deployment artifacts, including Docker images, Kubernetes manifests, and comprehensive deployment documentation. The goal is to ensure the Task Board API can be easily deployed, monitored, and maintained in production environments.

## Task Context

### Description
Create Dockerfile, Kubernetes manifests, and deployment documentation.

### Priority
Medium - Essential for moving the application to production.

### Dependencies
- Task 1: Development environment and toolchain
- Task 9: All tests must pass before deployment

### Subtasks
1. Write Dockerfile for Application
2. Create Kubernetes Manifests
3. Document Environment Variables
4. Write Deployment Steps Documentation
5. Verify Deployment

## Architecture Context

Based on the deployment architecture in architecture.md:

### Container Strategy
- Multi-stage Docker builds for optimal image size
- Security-hardened runtime with non-root user
- Minimal base image (debian:bookworm-slim)
- Health checks for container orchestration

### Kubernetes Deployment
- Deployment with 3 replicas for high availability
- Service for load balancing
- ConfigMaps for configuration
- Secrets for sensitive data
- Resource limits and requests

### Environment Configuration
- DATABASE_URL for PostgreSQL connection
- JWT_SECRET for token signing
- RUST_LOG for logging configuration
- Service-specific environment variables

## Implementation Details

### 1. Production Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 -U -s /bin/sh appuser

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (this is cached)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY . .

# Build application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Import user from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/task-board-api /app/task-board-api

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/task-board-api", "health"]

EXPOSE 50051

ENTRYPOINT ["/app/task-board-api"]
```

### 2. Kubernetes Manifests

#### k8s/namespace.yaml
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: task-board
  labels:
    app: task-board-api
```

#### k8s/configmap.yaml
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: task-board-config
  namespace: task-board
data:
  RUST_LOG: "info,task_board_api=debug"
  GRPC_PORT: "50051"
  DB_POOL_MAX_SIZE: "15"
  DB_POOL_MIN_IDLE: "5"
  JWT_EXPIRATION_HOURS: "24"
```

#### k8s/secret.yaml
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: task-board-secrets
  namespace: task-board
type: Opaque
stringData:
  DATABASE_URL: "postgres://username:password@postgres:5432/taskboard"
  JWT_SECRET: "your-secret-key-here-change-in-production"
```

#### k8s/deployment.yaml
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: task-board-api
  namespace: task-board
  labels:
    app: task-board-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: task-board-api
  template:
    metadata:
      labels:
        app: task-board-api
    spec:
      serviceAccountName: task-board-api
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        fsGroup: 1001
      containers:
      - name: api
        image: task-board-api:latest
        imagePullPolicy: Always
        ports:
        - name: grpc
          containerPort: 50051
          protocol: TCP
        envFrom:
        - configMapRef:
            name: task-board-config
        - secretRef:
            name: task-board-secrets
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          grpc:
            port: 50051
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          grpc:
            port: 50051
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
```

#### k8s/service.yaml
```yaml
apiVersion: v1
kind: Service
metadata:
  name: task-board-api
  namespace: task-board
  labels:
    app: task-board-api
spec:
  type: ClusterIP
  selector:
    app: task-board-api
  ports:
  - name: grpc
    port: 50051
    targetPort: 50051
    protocol: TCP
```

#### k8s/service-account.yaml
```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: task-board-api
  namespace: task-board
  labels:
    app: task-board-api
```

#### k8s/network-policy.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: task-board-api
  namespace: task-board
spec:
  podSelector:
    matchLabels:
      app: task-board-api
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress
    ports:
    - protocol: TCP
      port: 50051
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: database
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
```

### 3. Helm Chart (Optional)

#### helm/task-board-api/Chart.yaml
```yaml
apiVersion: v2
name: task-board-api
description: A gRPC-based task management API
type: application
version: 1.0.0
appVersion: "1.0.0"
```

#### helm/task-board-api/values.yaml
```yaml
replicaCount: 3

image:
  repository: task-board-api
  pullPolicy: IfNotPresent
  tag: ""

serviceAccount:
  create: true
  name: ""

service:
  type: ClusterIP
  port: 50051

ingress:
  enabled: false

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80

config:
  rustLog: "info,task_board_api=debug"
  dbPoolMaxSize: "15"
  dbPoolMinIdle: "5"
  jwtExpirationHours: "24"

secrets:
  databaseUrl: ""
  jwtSecret: ""
```

### 4. Docker Compose for Development

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: taskboard
      POSTGRES_PASSWORD: taskboard_password
      POSTGRES_DB: taskboard_db
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U taskboard"]
      interval: 10s
      timeout: 5s
      retries: 5

  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "50051:50051"
    environment:
      DATABASE_URL: postgres://taskboard:taskboard_password@postgres:5432/taskboard_db
      RUST_LOG: debug
      JWT_SECRET: development-secret-key
    depends_on:
      postgres:
        condition: service_healthy
    command: ["./task-board-api"]

volumes:
  postgres_data:
```

### 5. Deployment Scripts

#### scripts/build.sh
```bash
#!/bin/bash
set -e

VERSION=${1:-latest}
REGISTRY=${2:-docker.io/yourusername}

echo "Building Task Board API version: $VERSION"

# Build Docker image
docker build -t task-board-api:$VERSION .

# Tag for registry
docker tag task-board-api:$VERSION $REGISTRY/task-board-api:$VERSION

echo "Build complete. To push: docker push $REGISTRY/task-board-api:$VERSION"
```

#### scripts/deploy.sh
```bash
#!/bin/bash
set -e

NAMESPACE=${1:-task-board}
VERSION=${2:-latest}

echo "Deploying Task Board API to namespace: $NAMESPACE"

# Create namespace if it doesn't exist
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Apply configurations
kubectl apply -f k8s/ -n $NAMESPACE

# Update image
kubectl set image deployment/task-board-api api=task-board-api:$VERSION -n $NAMESPACE

# Wait for rollout
kubectl rollout status deployment/task-board-api -n $NAMESPACE

echo "Deployment complete!"
```

### 6. Environment Documentation

#### Environment Variables

| Variable | Description | Required | Default | Example |
|----------|-------------|----------|---------|---------|
| DATABASE_URL | PostgreSQL connection string | Yes | - | postgres://user:pass@host:5432/db |
| JWT_SECRET | Secret key for JWT signing | Yes | - | your-secret-key-here |
| RUST_LOG | Logging configuration | No | info | debug,task_board_api=trace |
| GRPC_PORT | Port for gRPC server | No | 50051 | 50051 |
| DB_POOL_MAX_SIZE | Maximum database connections | No | 15 | 20 |
| DB_POOL_MIN_IDLE | Minimum idle connections | No | 5 | 3 |
| JWT_EXPIRATION_HOURS | Token expiration time | No | 24 | 48 |

### 7. Deployment Documentation

#### deployment.md
```markdown
# Task Board API Deployment Guide

## Prerequisites
- Docker 20.10+
- Kubernetes 1.24+
- kubectl configured
- PostgreSQL 14+ database

## Quick Start

### Local Development
```bash
docker compose up -d
```

### Production Deployment

1. Build and push image:
```bash
./scripts/build.sh v1.0.0 your-registry.com
docker push your-registry.com/task-board-api:v1.0.0
```

2. Configure secrets:
```bash
kubectl create secret generic task-board-secrets \
  --from-literal=DATABASE_URL=your-db-url \
  --from-literal=JWT_SECRET=your-secret \
  -n task-board
```

3. Deploy:
```bash
./scripts/deploy.sh task-board v1.0.0
```

## Monitoring

Check deployment status:
```bash
kubectl get pods -n task-board
kubectl logs -f deployment/task-board-api -n task-board
```

## Troubleshooting

Common issues and solutions...
```

## Dependencies

- Task 1: Base toolchain and project setup
- Task 9: Tests must pass before creating deployment artifacts

## Testing Strategy

### Build Verification
1. Build Docker image locally
2. Run container with test configuration
3. Verify health checks pass
4. Test basic functionality

### Deployment Testing
1. Deploy to staging environment
2. Run smoke tests against deployed service
3. Verify metrics and logs
4. Test rollback procedures

### Security Scanning
1. Scan Docker image for vulnerabilities
2. Verify non-root user execution
3. Check for exposed secrets
4. Test network policies