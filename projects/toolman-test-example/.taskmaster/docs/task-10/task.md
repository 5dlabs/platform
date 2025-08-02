# Task 10: Deployment and Documentation

## Overview
Prepare the chat application for production deployment with containerization, orchestration, CI/CD pipelines, and comprehensive documentation. This task encompasses creating production-ready Docker configurations, Kubernetes manifests, automated deployment pipelines, API documentation, user guides, and monitoring infrastructure.

## Technical Architecture

### Deployment Stack
- **Containerization**: Docker with multi-stage builds
- **Orchestration**: Kubernetes for container management
- **CI/CD**: GitHub Actions for automated workflows
- **API Documentation**: OpenAPI/Swagger specification
- **Monitoring**: Prometheus + Grafana + ELK Stack
- **Error Tracking**: Sentry for application errors

### Production Architecture
```
┌─────────────────┐     ┌─────────────────┐
│   CloudFlare    │     │  Load Balancer  │
│      (CDN)      │     │   (Ingress)     │
└────────┬────────┘     └────────┬────────┘
         │                       │
         │              ┌────────┴────────┐
         │              │                 │
    ┌────▼────┐    ┌────▼────┐      ┌────▼────┐
    │Frontend │    │Backend 1│      │Backend N│
    │ (Nginx) │    │ (Node)  │      │ (Node)  │
    └─────────┘    └────┬────┘      └────┬────┘
                        │                 │
                   ┌────▼────┐       ┌────▼────┐
                   │  Redis  │       │Postgres │
                   │(Cluster)│       │(Primary)│
                   └─────────┘       └─────────┘
```

## Implementation Details

### 1. Docker Configuration

#### Frontend Dockerfile
```dockerfile
# frontend/Dockerfile
# Build stage
FROM node:18-alpine AS builder

# Install dependencies for node-gyp
RUN apk add --no-cache python3 make g++

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY yarn.lock* ./

# Install dependencies
RUN npm ci --only=production && \
    npm cache clean --force

# Copy source code
COPY . .

# Build the application
ARG REACT_APP_API_URL
ARG REACT_APP_WS_URL
ENV REACT_APP_API_URL=$REACT_APP_API_URL
ENV REACT_APP_WS_URL=$REACT_APP_WS_URL

RUN npm run build

# Production stage
FROM nginx:alpine

# Install curl for health checks
RUN apk add --no-cache curl

# Copy custom nginx config
COPY nginx.conf /etc/nginx/nginx.conf
COPY default.conf /etc/nginx/conf.d/default.conf

# Copy built application
COPY --from=builder /app/build /usr/share/nginx/html

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost/health || exit 1

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

#### Backend Dockerfile
```dockerfile
# backend/Dockerfile
# Build stage
FROM node:18-alpine AS builder

# Install build dependencies
RUN apk add --no-cache python3 make g++

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY yarn.lock* ./
COPY tsconfig.json ./

# Install all dependencies (including dev)
RUN npm ci

# Copy source code
COPY src ./src

# Build the application
RUN npm run build

# Production stage
FROM node:18-alpine

# Install production dependencies
RUN apk add --no-cache tini

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY yarn.lock* ./

# Install production dependencies only
RUN npm ci --only=production && \
    npm cache clean --force

# Copy built application
COPY --from=builder /app/dist ./dist

# Copy migration files
COPY migrations ./migrations

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

# Change ownership
RUN chown -R nodejs:nodejs /app

USER nodejs

# Use tini for proper signal handling
ENTRYPOINT ["/sbin/tini", "--"]

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node healthcheck.js || exit 1

EXPOSE 3000

CMD ["node", "dist/index.js"]
```

### 2. Kubernetes Manifests

#### Deployment Configuration
```yaml
# k8s/backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chat-backend
  labels:
    app: chat-backend
    version: v1
spec:
  replicas: 3
  selector:
    matchLabels:
      app: chat-backend
  template:
    metadata:
      labels:
        app: chat-backend
        version: v1
    spec:
      containers:
      - name: backend
        image: your-registry/chat-backend:latest
        ports:
        - containerPort: 3000
          name: http
        env:
        - name: NODE_ENV
          value: "production"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: chat-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: chat-secrets
              key: redis-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: chat-secrets
              key: jwt-secret
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
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - chat-backend
              topologyKey: kubernetes.io/hostname
---
apiVersion: v1
kind: Service
metadata:
  name: chat-backend
  labels:
    app: chat-backend
spec:
  type: ClusterIP
  ports:
  - port: 3000
    targetPort: 3000
    protocol: TCP
    name: http
  selector:
    app: chat-backend
```

#### Ingress Configuration
```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: chat-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
    nginx.ingress.kubernetes.io/websocket-services: chat-backend
spec:
  tls:
  - hosts:
    - chat.example.com
    - api.chat.example.com
    secretName: chat-tls
  rules:
  - host: chat.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: chat-frontend
            port:
              number: 80
  - host: api.chat.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: chat-backend
            port:
              number: 3000
```

#### Horizontal Pod Autoscaler
```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: chat-backend-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: chat-backend
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
```

### 3. CI/CD Pipeline

#### GitHub Actions Workflow
```yaml
# .github/workflows/deploy.yml
name: Build and Deploy

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [18.x]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Use Node.js ${{ matrix.node-version }}
      uses: actions/setup-node@v3
      with:
        node-version: ${{ matrix.node-version }}
        cache: 'npm'
    
    - name: Install dependencies
      run: |
        cd backend && npm ci
        cd ../frontend && npm ci
    
    - name: Run backend tests
      run: cd backend && npm test
    
    - name: Run frontend tests
      run: cd frontend && npm test -- --coverage --watchAll=false
    
    - name: Run E2E tests
      run: |
        docker-compose -f docker-compose.test.yml up -d
        npm run test:e2e
        docker-compose -f docker-compose.test.yml down

  build:
    needs: test
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Log in to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v2
    
    - name: Build and push backend image
      uses: docker/build-push-action@v4
      with:
        context: ./backend
        push: true
        tags: |
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/backend:latest
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/backend:${{ github.sha }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
    
    - name: Build and push frontend image
      uses: docker/build-push-action@v4
      with:
        context: ./frontend
        push: true
        tags: |
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/frontend:latest
          ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/frontend:${{ github.sha }}
        build-args: |
          REACT_APP_API_URL=${{ secrets.REACT_APP_API_URL }}
          REACT_APP_WS_URL=${{ secrets.REACT_APP_WS_URL }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Configure kubectl
      uses: azure/setup-kubectl@v3
      with:
        version: 'latest'
    
    - name: Set up Kustomize
      run: |
        curl -s "https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh" | bash
        sudo mv kustomize /usr/local/bin/
    
    - name: Update Kubernetes manifests
      run: |
        cd k8s
        kustomize edit set image backend=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/backend:${{ github.sha }}
        kustomize edit set image frontend=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/frontend:${{ github.sha }}
    
    - name: Deploy to Kubernetes
      env:
        KUBE_CONFIG: ${{ secrets.KUBE_CONFIG }}
      run: |
        echo "$KUBE_CONFIG" | base64 -d > kubeconfig
        export KUBECONFIG=kubeconfig
        kustomize build k8s | kubectl apply -f -
        kubectl rollout status deployment/chat-backend
        kubectl rollout status deployment/chat-frontend
```

### 4. API Documentation

#### OpenAPI Specification
```yaml
# docs/openapi.yaml
openapi: 3.0.0
info:
  title: Chat Application API
  version: 1.0.0
  description: Real-time chat application API documentation
  contact:
    name: API Support
    email: support@example.com
servers:
  - url: https://api.chat.example.com
    description: Production server
  - url: http://localhost:3000
    description: Development server

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
  
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
        username:
          type: string
        email:
          type: string
          format: email
        avatar:
          type: string
          nullable: true
        isOnline:
          type: boolean
        lastSeen:
          type: string
          format: date-time
      required:
        - id
        - username
        - email
    
    Message:
      type: object
      properties:
        id:
          type: string
          format: uuid
        content:
          type: string
        userId:
          type: string
          format: uuid
        roomId:
          type: string
          format: uuid
        attachments:
          type: array
          items:
            $ref: '#/components/schemas/Attachment'
        createdAt:
          type: string
          format: date-time
        updatedAt:
          type: string
          format: date-time
      required:
        - id
        - content
        - userId
        - roomId
    
    Room:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        description:
          type: string
          nullable: true
        isPrivate:
          type: boolean
        members:
          type: array
          items:
            $ref: '#/components/schemas/User'
        createdAt:
          type: string
          format: date-time
      required:
        - id
        - name
        - isPrivate
    
    Attachment:
      type: object
      properties:
        id:
          type: string
          format: uuid
        url:
          type: string
          format: uri
        thumbnailUrl:
          type: string
          format: uri
          nullable: true
        name:
          type: string
        type:
          type: string
        size:
          type: integer
      required:
        - id
        - url
        - name
        - type
        - size

paths:
  /auth/register:
    post:
      tags:
        - Authentication
      summary: Register a new user
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                username:
                  type: string
                  minLength: 3
                  maxLength: 30
                email:
                  type: string
                  format: email
                password:
                  type: string
                  minLength: 8
              required:
                - username
                - email
                - password
      responses:
        '201':
          description: User created successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  user:
                    $ref: '#/components/schemas/User'
                  token:
                    type: string
                  refreshToken:
                    type: string
        '400':
          description: Validation error
        '409':
          description: User already exists
  
  /auth/login:
    post:
      tags:
        - Authentication
      summary: Login user
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
              required:
                - email
                - password
      responses:
        '200':
          description: Login successful
          content:
            application/json:
              schema:
                type: object
                properties:
                  user:
                    $ref: '#/components/schemas/User'
                  token:
                    type: string
                  refreshToken:
                    type: string
        '401':
          description: Invalid credentials
  
  /messages:
    get:
      tags:
        - Messages
      summary: Get messages for a room
      security:
        - bearerAuth: []
      parameters:
        - in: query
          name: roomId
          required: true
          schema:
            type: string
            format: uuid
        - in: query
          name: limit
          schema:
            type: integer
            default: 50
        - in: query
          name: cursor
          schema:
            type: string
      responses:
        '200':
          description: Messages retrieved successfully
          content:
            application/json:
              schema:
                type: object
                properties:
                  messages:
                    type: array
                    items:
                      $ref: '#/components/schemas/Message'
                  nextCursor:
                    type: string
                    nullable: true
    
    post:
      tags:
        - Messages
      summary: Send a message
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                roomId:
                  type: string
                  format: uuid
                content:
                  type: string
                attachments:
                  type: array
                  items:
                    type: string
                    format: uuid
              required:
                - roomId
                - content
      responses:
        '201':
          description: Message sent successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Message'
        '400':
          description: Validation error
        '404':
          description: Room not found
  
  /rooms:
    get:
      tags:
        - Rooms
      summary: Get user's rooms
      security:
        - bearerAuth: []
      responses:
        '200':
          description: Rooms retrieved successfully
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Room'
    
    post:
      tags:
        - Rooms
      summary: Create a new room
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
                description:
                  type: string
                isPrivate:
                  type: boolean
                  default: false
                members:
                  type: array
                  items:
                    type: string
                    format: uuid
              required:
                - name
      responses:
        '201':
          description: Room created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Room'
        '400':
          description: Validation error
```

### 5. Monitoring Configuration

#### Prometheus Configuration
```yaml
# monitoring/prometheus.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
    
    scrape_configs:
    - job_name: 'kubernetes-pods'
      kubernetes_sd_configs:
      - role: pod
      relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
    
    - job_name: 'node-exporter'
      kubernetes_sd_configs:
      - role: node
      relabel_configs:
      - source_labels: [__address__]
        regex: '(.*):10250'
        replacement: '${1}:9100'
        target_label: __address__
    
    alerting:
      alertmanagers:
      - static_configs:
        - targets:
          - alertmanager:9093
    
    rule_files:
    - '/etc/prometheus/rules/*.yml'
```

#### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "Chat Application Metrics",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m])) by (method, route)"
          }
        ]
      },
      {
        "title": "Response Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, route))"
          }
        ]
      },
      {
        "title": "Active WebSocket Connections",
        "targets": [
          {
            "expr": "sum(websocket_active_connections)"
          }
        ]
      },
      {
        "title": "Message Delivery Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(message_delivery_time_ms_bucket[5m])) by (le))"
          }
        ]
      }
    ]
  }
}
```

### 6. Error Tracking with Sentry

```typescript
// backend/src/config/sentry.ts
import * as Sentry from '@sentry/node';
import { ProfilingIntegration } from '@sentry/profiling-node';

export function initSentry() {
  Sentry.init({
    dsn: process.env.SENTRY_DSN,
    environment: process.env.NODE_ENV,
    integrations: [
      new Sentry.Integrations.Http({ tracing: true }),
      new Sentry.Integrations.Express({ app }),
      new ProfilingIntegration(),
    ],
    tracesSampleRate: process.env.NODE_ENV === 'production' ? 0.1 : 1.0,
    profilesSampleRate: 1.0,
    beforeSend(event, hint) {
      // Filter out sensitive data
      if (event.request?.cookies) {
        delete event.request.cookies;
      }
      if (event.request?.headers?.authorization) {
        delete event.request.headers.authorization;
      }
      return event;
    },
  });
}

// Error handler middleware
export const sentryErrorHandler = Sentry.Handlers.errorHandler({
  shouldHandleError(error) {
    // Capture only 500 errors in production
    if (process.env.NODE_ENV === 'production') {
      return error.status === 500;
    }
    return true;
  },
});
```

## Documentation Structure

### User Documentation
```markdown
# Chat Application User Guide

## Getting Started
1. Register for an account
2. Verify your email
3. Log in to the application
4. Create or join a room
5. Start chatting!

## Features

### Real-time Messaging
- Send and receive messages instantly
- See when users are typing
- Message delivery indicators

### File Sharing
- Drag and drop files into chat
- Support for images, documents, and more
- Automatic image previews

### Room Management
- Create public or private rooms
- Invite users to rooms
- Set room administrators

## FAQ

### How do I reset my password?
Click "Forgot Password" on the login page...

### Can I delete messages?
Yes, you can delete your own messages within 5 minutes...

### What file types are supported?
Images (JPEG, PNG, GIF), Documents (PDF, DOC, DOCX)...
```

## Security Considerations

### Container Security
- Non-root user in containers
- Minimal base images
- Regular vulnerability scanning
- Secret management with Kubernetes secrets

### Network Security
- TLS encryption for all traffic
- Network policies for pod communication
- Ingress rate limiting
- DDoS protection with CloudFlare

### Application Security
- Environment-based configuration
- Secure session management
- Input validation and sanitization
- Regular security updates

## Deployment Checklist

### Pre-deployment
- [x] All tests passing
- [x] Security scan completed
- [x] Performance benchmarks met
- [x] Documentation updated
- [x] Database migrations tested

### Deployment
- [x] Docker images built
- [x] Kubernetes manifests applied
- [x] Secrets configured
- [x] Health checks passing
- [x] Monitoring active

### Post-deployment
- [x] Smoke tests passing
- [x] Performance metrics normal
- [x] Error rates acceptable
- [x] User acceptance testing
- [x] Rollback plan ready