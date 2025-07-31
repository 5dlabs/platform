# Task 10: Deployment and Documentation

## Overview

This task focuses on preparing the application for production deployment with comprehensive Docker containerization, Kubernetes orchestration, CI/CD pipeline setup, and creating thorough API and user documentation. The implementation emphasizes production readiness, security hardening, and comprehensive monitoring.

## Technical Implementation Guide

### 1. Production Docker Configuration

#### Multi-Stage Build for Frontend

```dockerfile
# Frontend Dockerfile
FROM node:18-alpine as dependencies
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:18-alpine as build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine-slim
# Security: Run as non-root user
RUN addgroup -g 1001 -S nginx && adduser -S -D -H -u 1001 -h /var/cache/nginx -s /sbin/nologin -G nginx -g nginx nginx
COPY --from=build /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
COPY security-headers.conf /etc/nginx/conf.d/security-headers.conf
EXPOSE 8080
USER nginx
CMD ["nginx", "-g", "daemon off;"]
```

#### Multi-Stage Build for Backend (Rust)

```dockerfile
# Backend Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY . .
RUN touch src/main.rs
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1001 -s /bin/bash app
WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/static ./static
RUN chown -R app:app /app
USER app
EXPOSE 3000
CMD ["./backend"]
```

#### Nginx Configuration with Security Headers

```nginx
server {
    listen 8080;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self' wss: https:;" always;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://backend:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws {
        proxy_pass http://backend:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 2. Kubernetes Deployment Manifests

#### Namespace and ConfigMap

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: task-master
---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: task-master
data:
  NODE_ENV: "production"
  LOG_LEVEL: "info"
  RUST_LOG: "info,tower_http=debug"
```

#### Backend Deployment

```yaml
# backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
  namespace: task-master
spec:
  replicas: 3
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        fsGroup: 1001
      containers:
      - name: backend
        image: ghcr.io/5dlabs/task-master-backend:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        envFrom:
        - configMapRef:
            name: app-config
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
```

#### Frontend Deployment

```yaml
# frontend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
  namespace: task-master
spec:
  replicas: 2
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        fsGroup: 1001
      containers:
      - name: frontend
        image: ghcr.io/5dlabs/task-master-frontend:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
```

#### Services and Ingress

```yaml
# services.yaml
apiVersion: v1
kind: Service
metadata:
  name: backend
  namespace: task-master
spec:
  selector:
    app: backend
  ports:
  - port: 3000
    targetPort: 3000
  type: ClusterIP
---
apiVersion: v1
kind: Service
metadata:
  name: frontend
  namespace: task-master
spec:
  selector:
    app: frontend
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
---
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: task-master-ingress
  namespace: task-master
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - taskmaster.5dlabs.com
    secretName: taskmaster-tls
  rules:
  - host: taskmaster.5dlabs.com
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: backend
            port:
              number: 3000
      - path: /ws
        pathType: Prefix
        backend:
          service:
            name: backend
            port:
              number: 3000
      - path: /
        pathType: Prefix
        backend:
          service:
            name: frontend
            port:
              number: 80
```

### 3. CI/CD Pipeline with GitHub Actions

#### Build and Test Workflow

```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test-backend:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    - name: Run tests
      run: |
        cd backend
        cargo test --all-features
    - name: Run clippy
      run: |
        cd backend
        cargo clippy -- -D warnings

  test-frontend:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
        cache-dependency-path: frontend/package-lock.json
    - name: Install dependencies
      run: |
        cd frontend
        npm ci
    - name: Run tests
      run: |
        cd frontend
        npm test -- --coverage --watchAll=false
    - name: Run linter
      run: |
        cd frontend
        npm run lint

  build-and-push:
    needs: [test-backend, test-frontend]
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    permissions:
      contents: read
      packages: write
    steps:
    - uses: actions/checkout@v4
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    - name: Build and push Backend
      uses: docker/build-push-action@v5
      with:
        context: ./backend
        push: true
        tags: |
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-backend:latest
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-backend:${{ github.sha }}
    - name: Build and push Frontend
      uses: docker/build-push-action@v5
      with:
        context: ./frontend
        push: true
        tags: |
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-frontend:latest
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-frontend:${{ github.sha }}

  deploy:
    needs: build-and-push
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v4
    - name: Deploy to Kubernetes
      uses: azure/k8s-deploy@v4
      with:
        manifests: |
          k8s/namespace.yaml
          k8s/configmap.yaml
          k8s/backend-deployment.yaml
          k8s/frontend-deployment.yaml
          k8s/services.yaml
          k8s/ingress.yaml
        images: |
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-backend:${{ github.sha }}
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}-frontend:${{ github.sha }}
```

### 4. API Documentation with OpenAPI/Swagger

#### OpenAPI Specification

```yaml
# openapi.yaml
openapi: 3.0.3
info:
  title: Task Master API
  description: Real-time task management and collaboration platform
  version: 1.0.0
  contact:
    name: API Support
    email: support@5dlabs.com
servers:
  - url: https://taskmaster.5dlabs.com/api
    description: Production server
  - url: http://localhost:3000/api
    description: Development server
security:
  - bearerAuth: []
paths:
  /auth/register:
    post:
      summary: Register new user
      tags: [Authentication]
      security: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [email, password, username]
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
                  minLength: 8
                username:
                  type: string
                  minLength: 3
      responses:
        '201':
          description: User created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AuthResponse'
  /tasks:
    get:
      summary: List all tasks
      tags: [Tasks]
      parameters:
        - name: status
          in: query
          schema:
            type: string
            enum: [pending, in_progress, completed]
        - name: assignee
          in: query
          schema:
            type: string
      responses:
        '200':
          description: List of tasks
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Task'
components:
  schemas:
    Task:
      type: object
      properties:
        id:
          type: string
          format: uuid
        title:
          type: string
        description:
          type: string
        status:
          type: string
          enum: [pending, in_progress, completed]
        assignee:
          $ref: '#/components/schemas/User'
        created_at:
          type: string
          format: date-time
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
```

### 5. Monitoring and Logging Setup

#### Prometheus Metrics

```rust
// src/metrics.rs
use prometheus::{Encoder, TextEncoder, Counter, Histogram, register_counter, register_histogram};
use axum::{response::IntoResponse, http::StatusCode};

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();
    
    static ref HTTP_REQUEST_DURATION: Histogram = register_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).unwrap();
}

pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (StatusCode::OK, buffer)
}
```

#### Structured Logging

```rust
// src/logging.rs
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

// Usage in handlers
#[instrument(skip(db))]
pub async fn create_task(
    State(db): State<PgPool>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    info!(user_id = %payload.user_id, "Creating new task");
    // Implementation
}
```

### 6. Security Hardening

#### Environment Configuration

```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-secret
  namespace: task-master
type: Opaque
stringData:
  url: "postgresql://user:password@postgres:5432/taskmaster?sslmode=require"
---
apiVersion: v1
kind: Secret
metadata:
  name: jwt-secret
  namespace: task-master
type: Opaque
stringData:
  secret: "your-256-bit-secret-key-here"
```

#### Network Policies

```yaml
# k8s/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: backend-network-policy
  namespace: task-master
spec:
  podSelector:
    matchLabels:
      app: backend
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 3000
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 5432
    - protocol: TCP
      port: 6379
```

## Implementation Steps

1. **Docker Setup**
   - Create multi-stage Dockerfiles for frontend and backend
   - Configure Nginx with security headers
   - Set up Docker Compose for local testing
   - Implement health check endpoints

2. **Kubernetes Configuration**
   - Create namespace and RBAC policies
   - Deploy backend and frontend with proper resource limits
   - Configure services and ingress with TLS
   - Set up horizontal pod autoscaling

3. **CI/CD Pipeline**
   - Configure GitHub Actions workflows
   - Implement automated testing stages
   - Set up container registry integration
   - Configure automated deployment to Kubernetes

4. **API Documentation**
   - Write comprehensive OpenAPI specification
   - Document all endpoints with examples
   - Include authentication flow documentation
   - Generate interactive API documentation

5. **Monitoring Setup**
   - Integrate Prometheus metrics
   - Configure structured logging
   - Set up alerting rules
   - Create Grafana dashboards

6. **Security Implementation**
   - Configure secrets management
   - Implement network policies
   - Set up pod security policies
   - Configure RBAC for service accounts

## Testing Strategy

- Test Docker builds with security scanning
- Validate Kubernetes manifests with dry-run
- Test CI/CD pipeline in staging environment
- Verify monitoring and alerting functionality
- Conduct security audit of deployed application
- Load test the deployed application
- Test rollback procedures

## Deliverables

1. Production-ready Docker configurations
2. Complete Kubernetes deployment manifests
3. Functional CI/CD pipeline
4. Comprehensive API documentation
5. User documentation with screenshots
6. Monitoring and alerting setup
7. Security audit report