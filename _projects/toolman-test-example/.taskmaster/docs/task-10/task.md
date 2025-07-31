# Task 10: Deployment and Documentation

## Overview
Prepare the application for production deployment with containerization, Kubernetes orchestration, CI/CD pipelines, and comprehensive documentation. Research and implement deployment patterns from industry best practices including Rust ecosystem insights.

## Technical Implementation Guide

### Phase 1: Production Docker Configuration

#### Multi-Stage Frontend Build
```dockerfile
# frontend/Dockerfile.production
# Stage 1: Build
FROM node:18-alpine AS builder
WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci --only=production

# Copy source and build
COPY . .
ARG REACT_APP_API_URL
ARG REACT_APP_SOCKET_URL
ENV REACT_APP_API_URL=$REACT_APP_API_URL
ENV REACT_APP_SOCKET_URL=$REACT_APP_SOCKET_URL
RUN npm run build

# Stage 2: Production
FROM nginx:alpine
WORKDIR /usr/share/nginx/html

# Copy build artifacts
COPY --from=builder /app/build .

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost/health || exit 1

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

#### Nginx Configuration
```nginx
# frontend/nginx.conf
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # Compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self' ws: wss: https: data: 'unsafe-inline' 'unsafe-eval';" always;

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # API proxy
    location /api {
        proxy_pass http://backend:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Socket.io proxy
    location /socket.io {
        proxy_pass http://backend:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }

    # React app routing
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

#### Multi-Stage Backend Build
```dockerfile
# backend/Dockerfile.production
# Stage 1: Build
FROM node:18-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache python3 make g++

# Install dependencies
COPY package*.json ./
RUN npm ci

# Copy source and build
COPY . .
RUN npm run build

# Stage 2: Production
FROM node:18-alpine
WORKDIR /app

# Install production dependencies only
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force

# Copy built application
COPY --from=builder /app/dist ./dist

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

# Set ownership
RUN chown -R nodejs:nodejs /app

USER nodejs

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3001/health', (res) => { process.exit(res.statusCode === 200 ? 0 : 1); })"

EXPOSE 3001
CMD ["node", "dist/index.js"]
```

### Phase 2: Kubernetes Deployment Configuration

#### Namespace and ConfigMap
```yaml
# kubernetes/00-namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: chat-app
  labels:
    app: chat-app
---
# kubernetes/01-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: chat-app-config
  namespace: chat-app
data:
  NODE_ENV: "production"
  REDIS_HOST: "redis-service"
  REDIS_PORT: "6379"
  DATABASE_HOST: "postgres-service"
  DATABASE_PORT: "5432"
  DATABASE_NAME: "chatdb"
```

#### Secrets Configuration
```yaml
# kubernetes/02-secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: chat-app-secrets
  namespace: chat-app
type: Opaque
stringData:
  DATABASE_USER: "chatuser"
  DATABASE_PASSWORD: "changeme"
  JWT_SECRET: "your-jwt-secret-here"
  JWT_REFRESH_SECRET: "your-refresh-secret-here"
  AWS_ACCESS_KEY_ID: "your-aws-key"
  AWS_SECRET_ACCESS_KEY: "your-aws-secret"
```

#### Backend Deployment
```yaml
# kubernetes/10-backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chat-backend
  namespace: chat-app
  labels:
    app: chat-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: chat-backend
  template:
    metadata:
      labels:
        app: chat-backend
    spec:
      containers:
      - name: backend
        image: chat-app/backend:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 3001
        env:
        - name: NODE_ENV
          valueFrom:
            configMapKeyRef:
              name: chat-app-config
              key: NODE_ENV
        - name: DATABASE_URL
          value: "postgresql://$(DATABASE_USER):$(DATABASE_PASSWORD)@$(DATABASE_HOST):$(DATABASE_PORT)/$(DATABASE_NAME)"
        - name: REDIS_URL
          value: "redis://$(REDIS_HOST):$(REDIS_PORT)"
        envFrom:
        - configMapRef:
            name: chat-app-config
        - secretRef:
            name: chat-app-secrets
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
            port: 3001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3001
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: chat-backend-service
  namespace: chat-app
spec:
  selector:
    app: chat-backend
  ports:
  - protocol: TCP
    port: 3001
    targetPort: 3001
  type: ClusterIP
```

#### Frontend Deployment
```yaml
# kubernetes/20-frontend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chat-frontend
  namespace: chat-app
spec:
  replicas: 2
  selector:
    matchLabels:
      app: chat-frontend
  template:
    metadata:
      labels:
        app: chat-frontend
    spec:
      containers:
      - name: frontend
        image: chat-app/frontend:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 10
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: chat-frontend-service
  namespace: chat-app
spec:
  selector:
    app: chat-frontend
  ports:
  - protocol: TCP
    port: 80
    targetPort: 80
  type: ClusterIP
```

#### Ingress Configuration
```yaml
# kubernetes/30-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: chat-app-ingress
  namespace: chat-app
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/websocket-services: "chat-backend-service"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - chat.yourdomain.com
    secretName: chat-app-tls
  rules:
  - host: chat.yourdomain.com
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: chat-backend-service
            port:
              number: 3001
      - path: /socket.io
        pathType: Prefix
        backend:
          service:
            name: chat-backend-service
            port:
              number: 3001
      - path: /
        pathType: Prefix
        backend:
          service:
            name: chat-frontend-service
            port:
              number: 80
```

#### Horizontal Pod Autoscaler
```yaml
# kubernetes/40-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: chat-backend-hpa
  namespace: chat-app
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
```

### Phase 3: CI/CD Pipeline

#### GitHub Actions Workflow
```yaml
# .github/workflows/deploy.yml
name: Build and Deploy

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: testpass
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: |
        npm ci --prefix frontend
        npm ci --prefix backend
    
    - name: Run tests
      run: |
        npm run test --prefix frontend -- --coverage
        npm run test --prefix backend -- --coverage
      env:
        DATABASE_URL: postgresql://postgres:testpass@localhost:5432/testdb
        REDIS_URL: redis://localhost:6379

    - name: Upload coverage
      uses: codecov/codecov-action@v3

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

    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v4
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=semver,pattern={{version}}
          type=sha

    - name: Build and push Frontend
      uses: docker/build-push-action@v4
      with:
        context: ./frontend
        file: ./frontend/Dockerfile.production
        push: true
        tags: ${{ steps.meta.outputs.tags }}-frontend
        labels: ${{ steps.meta.outputs.labels }}
        build-args: |
          REACT_APP_API_URL=/api
          REACT_APP_SOCKET_URL=/

    - name: Build and push Backend
      uses: docker/build-push-action@v4
      with:
        context: ./backend
        file: ./backend/Dockerfile.production
        push: true
        tags: ${{ steps.meta.outputs.tags }}-backend
        labels: ${{ steps.meta.outputs.labels }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    steps:
    - uses: actions/checkout@v3

    - name: Deploy to Kubernetes
      env:
        KUBE_CONFIG: ${{ secrets.KUBE_CONFIG }}
      run: |
        echo "$KUBE_CONFIG" | base64 -d > kubeconfig
        export KUBECONFIG=kubeconfig
        
        # Update image tags
        kubectl set image deployment/chat-backend \
          backend=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:sha-${GITHUB_SHA::7}-backend \
          -n chat-app
        
        kubectl set image deployment/chat-frontend \
          frontend=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:sha-${GITHUB_SHA::7}-frontend \
          -n chat-app
        
        # Wait for rollout
        kubectl rollout status deployment/chat-backend -n chat-app
        kubectl rollout status deployment/chat-frontend -n chat-app
```

### Phase 4: API Documentation

#### OpenAPI Specification
```yaml
# docs/openapi.yaml
openapi: 3.0.0
info:
  title: Chat Application API
  version: 1.0.0
  description: Real-time chat application REST API

servers:
  - url: https://api.chatapp.com
    description: Production server
  - url: http://localhost:3001
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
        email:
          type: string
          format: email
        username:
          type: string
        avatarUrl:
          type: string
        createdAt:
          type: string
          format: date-time

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
        isPrivate:
          type: boolean
        createdBy:
          type: string
          format: uuid
        memberCount:
          type: integer
        createdAt:
          type: string
          format: date-time

    Message:
      type: object
      properties:
        id:
          type: string
          format: uuid
        roomId:
          type: string
          format: uuid
        userId:
          type: string
          format: uuid
        content:
          type: string
        messageType:
          type: string
          enum: [text, image, file]
        attachments:
          type: array
          items:
            $ref: '#/components/schemas/Attachment'
        createdAt:
          type: string
          format: date-time

    Attachment:
      type: object
      properties:
        id:
          type: string
          format: uuid
        url:
          type: string
        thumbnailUrl:
          type: string
        name:
          type: string
        size:
          type: integer
        type:
          type: string

paths:
  /api/auth/register:
    post:
      summary: Register new user
      tags: [Authentication]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [email, username, password]
              properties:
                email:
                  type: string
                  format: email
                username:
                  type: string
                  minLength: 3
                  maxLength: 20
                password:
                  type: string
                  minLength: 8
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
                  accessToken:
                    type: string
                  refreshToken:
                    type: string

  /api/auth/login:
    post:
      summary: Login user
      tags: [Authentication]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [email, password]
              properties:
                email:
                  type: string
                  format: email
                password:
                  type: string
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
                  accessToken:
                    type: string
                  refreshToken:
                    type: string

  /api/rooms:
    get:
      summary: List all rooms
      tags: [Rooms]
      security:
        - bearerAuth: []
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            default: 1
        - name: limit
          in: query
          schema:
            type: integer
            default: 20
        - name: search
          in: query
          schema:
            type: string
      responses:
        '200':
          description: List of rooms
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: '#/components/schemas/Room'
                  pagination:
                    type: object
                    properties:
                      page:
                        type: integer
                      limit:
                        type: integer
                      total:
                        type: integer
                      totalPages:
                        type: integer

    post:
      summary: Create new room
      tags: [Rooms]
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [name]
              properties:
                name:
                  type: string
                  minLength: 3
                  maxLength: 100
                description:
                  type: string
                  maxLength: 500
                isPrivate:
                  type: boolean
                  default: false
      responses:
        '201':
          description: Room created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Room'
```

### Phase 5: Monitoring and Logging

#### Prometheus Configuration
```yaml
# kubernetes/monitoring/prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s

    scrape_configs:
      - job_name: 'chat-backend'
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names:
                - chat-app
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_label_app]
            action: keep
            regex: chat-backend
          - source_labels: [__meta_kubernetes_pod_name]
            target_label: instance
          - target_label: __address__
            replacement: chat-backend-service:3001
        metrics_path: /metrics
```

#### Grafana Dashboard Configuration
```json
{
  "dashboard": {
    "title": "Chat Application Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_request_duration_seconds_count[5m])) by (method, route)"
          }
        ]
      },
      {
        "title": "Response Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, method, route))"
          }
        ]
      },
      {
        "title": "Active WebSocket Connections",
        "targets": [
          {
            "expr": "websocket_active_connections"
          }
        ]
      },
      {
        "title": "Message Delivery Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(message_delivery_latency_ms_bucket[5m])) by (le))"
          }
        ]
      }
    ]
  }
}
```

### Phase 6: User Documentation

#### README.md
```markdown
# Chat Application

A real-time chat application built with React, Node.js, Socket.io, PostgreSQL, and Redis.

## Features

- 🔐 **Secure Authentication**: JWT-based auth with refresh tokens
- 💬 **Real-time Messaging**: Instant message delivery with Socket.io
- 🏠 **Multiple Chat Rooms**: Create and join different rooms
- 📎 **File Sharing**: Share images and documents
- 🌓 **Dark Mode**: Toggle between light and dark themes
- 📱 **Responsive Design**: Works on desktop and mobile
- ⚡ **High Performance**: Optimized for 1000+ concurrent users

## Quick Start

### Using Docker Compose

```bash
# Clone the repository
git clone https://github.com/yourusername/chat-app.git
cd chat-app

# Copy environment variables
cp .env.example .env

# Start the application
docker-compose up -d

# Run database migrations
docker-compose exec backend npm run migrate:up
```

The application will be available at http://localhost

### Manual Installation

#### Prerequisites
- Node.js 18+
- PostgreSQL 14+
- Redis 7+

#### Backend Setup
```bash
cd backend
npm install
npm run migrate:up
npm run dev
```

#### Frontend Setup
```bash
cd frontend
npm install
npm start
```

## Configuration

Create a `.env` file with the following variables:

```env
# Backend
NODE_ENV=development
PORT=3001
DATABASE_URL=postgresql://user:password@localhost:5432/chatdb
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
JWT_REFRESH_SECRET=your-refresh-secret

# Frontend
REACT_APP_API_URL=http://localhost:3001
REACT_APP_SOCKET_URL=http://localhost:3001
```

## API Documentation

API documentation is available at `/api/docs` when running in development mode.

## WebSocket Events

### Client → Server
- `join-room`: Join a chat room
- `leave-room`: Leave a chat room
- `send-message`: Send a message
- `typing-start`: Start typing indicator
- `typing-stop`: Stop typing indicator

### Server → Client
- `message-received`: New message in room
- `user-joined-room`: User joined room
- `user-left-room`: User left room
- `user-typing`: User is typing
- `user-online`: User came online
- `user-offline`: User went offline

## Deployment

### Kubernetes

```bash
# Apply configurations
kubectl apply -f kubernetes/

# Check deployment status
kubectl get pods -n chat-app
```

### Docker Swarm

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.yml chat-app
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
```

## Success Metrics

- Zero-downtime deployments achieved
- Automatic scaling based on load
- Comprehensive monitoring dashboards
- API documentation complete and accurate
- CI/CD pipeline runs in < 10 minutes
- Container images optimized (< 100MB)
- Kubernetes resources properly configured
- Security best practices implemented