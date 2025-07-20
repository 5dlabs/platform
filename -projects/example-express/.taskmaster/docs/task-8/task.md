# Task 8: Prepare Deployment Configuration

## Overview

This task involves preparing the Express.js application for production deployment by containerizing it with Docker, configuring environment variables, and setting up automated deployment workflows. The goal is to make the application production-ready and easily deployable to various hosting platforms.

## Prerequisites

- Completed Tasks 1-7
- Docker installed locally for testing
- Basic knowledge of containerization
- Understanding of environment variables
- GitHub account for CI/CD setup

## Implementation Steps

### 1. Create Dockerfile

Create a multi-stage Dockerfile for optimized production builds:

```dockerfile
# Stage 1: Build stage
FROM node:20-alpine AS builder

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install all dependencies (including devDependencies for building)
RUN npm ci

# Copy source code
COPY . .

# Run any build steps if needed (e.g., TypeScript compilation)
# RUN npm run build

# Stage 2: Production stage
FROM node:20-alpine

# Install dumb-init for proper signal handling
RUN apk add --no-cache dumb-init

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install only production dependencies
RUN npm ci --production && npm cache clean --force

# Copy application code from builder
COPY --from=builder /app/src ./src
COPY --from=builder /app/public ./public
COPY --from=builder /app/.env.example ./.env.example

# Change ownership to nodejs user
RUN chown -R nodejs:nodejs /app

# Switch to non-root user
USER nodejs

# Expose port (use environment variable)
EXPOSE 3000

# Use dumb-init to handle signals properly
ENTRYPOINT ["dumb-init", "--"]

# Start the application
CMD ["node", "src/index.js"]
```

### 2. Create .dockerignore

Optimize build context by excluding unnecessary files:

```
# Dependencies
node_modules/
npm-debug.log
yarn-error.log

# Environment files
.env
.env.local
.env.*.local

# Test files
tests/
coverage/
.nyc_output/
jest.config.js

# Development files
.eslintrc.js
.prettierrc
.editorconfig

# Git files
.git/
.gitignore
.github/

# Documentation
README.md
docs/
*.md

# IDE files
.vscode/
.idea/
*.swp
*.swo
*~

# OS files
.DS_Store
Thumbs.db

# Build artifacts
dist/
build/
```

### 3. Environment Configuration

Create a comprehensive `.env.example` file:

```bash
# Application
NODE_ENV=production
PORT=3000
HOST=0.0.0.0

# Database
DATABASE_URL=./data/app.db
# For production, consider PostgreSQL:
# DATABASE_URL=postgresql://user:password@host:5432/dbname

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-change-this
JWT_REFRESH_SECRET=your-super-secret-refresh-key-change-this
JWT_EXPIRES_IN=15m
JWT_REFRESH_EXPIRES_IN=7d

# Security
BCRYPT_ROUNDS=12
RATE_LIMIT_WINDOW_MS=900000
RATE_LIMIT_MAX_REQUESTS=100

# CORS (comma-separated origins)
CORS_ORIGINS=http://localhost:3000,https://yourdomain.com

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Session
SESSION_SECRET=your-session-secret-change-this

# Optional: External Services
# REDIS_URL=redis://localhost:6379
# SENTRY_DSN=https://your-sentry-dsn
# AWS_REGION=us-east-1
```

### 4. Update Application Configuration

Create `src/config/index.js` for centralized configuration:

```javascript
const path = require('path');

// Load environment variables
require('dotenv').config();

const config = {
  // Environment
  env: process.env.NODE_ENV || 'development',
  isProduction: process.env.NODE_ENV === 'production',
  isDevelopment: process.env.NODE_ENV === 'development',
  isTest: process.env.NODE_ENV === 'test',

  // Server
  port: parseInt(process.env.PORT || '3000', 10),
  host: process.env.HOST || '0.0.0.0',

  // Database
  database: {
    url: process.env.DATABASE_URL || './data/app.db',
    // For SQLite in production, ensure data directory is persistent
    dataDir: path.join(__dirname, '../../data')
  },

  // JWT
  jwt: {
    secret: process.env.JWT_SECRET,
    refreshSecret: process.env.JWT_REFRESH_SECRET,
    expiresIn: process.env.JWT_EXPIRES_IN || '15m',
    refreshExpiresIn: process.env.JWT_REFRESH_EXPIRES_IN || '7d'
  },

  // Security
  security: {
    bcryptRounds: parseInt(process.env.BCRYPT_ROUNDS || '10', 10),
    rateLimitWindowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000', 10),
    rateLimitMaxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10)
  },

  // CORS
  cors: {
    origins: process.env.CORS_ORIGINS
      ? process.env.CORS_ORIGINS.split(',').map(origin => origin.trim())
      : ['http://localhost:3000']
  },

  // Logging
  logging: {
    level: process.env.LOG_LEVEL || 'info',
    format: process.env.LOG_FORMAT || 'json'
  }
};

// Validate required environment variables
const requiredEnvVars = ['JWT_SECRET', 'JWT_REFRESH_SECRET'];
if (config.isProduction) {
  requiredEnvVars.forEach(varName => {
    if (!process.env[varName]) {
      throw new Error(`Missing required environment variable: ${varName}`);
    }
  });
}

module.exports = config;
```

### 5. Create docker-compose files

Development compose file (`docker-compose.yml`):

```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=development
      - DATABASE_URL=./data/app.db
    volumes:
      - ./src:/app/src
      - ./public:/app/public
      - ./data:/app/data
    command: npm run dev
```

Production compose file (`docker-compose.prod.yml`):

```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "80:3000"
    environment:
      - NODE_ENV=production
    env_file:
      - .env
    volumes:
      - ./data:/app/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

### 6. Add Health Check Endpoint

Create `src/routes/health.js`:

```javascript
const express = require('express');
const router = express.Router();
const db = require('../utils/db');

router.get('/health', async (req, res) => {
  const health = {
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: process.env.NODE_ENV,
    version: process.env.npm_package_version || '1.0.0'
  };

  try {
    // Check database connection
    const dbCheck = db.prepare('SELECT 1 as result').get();
    health.database = dbCheck ? 'connected' : 'disconnected';
  } catch (error) {
    health.status = 'unhealthy';
    health.database = 'error';
    health.error = error.message;
    return res.status(503).json(health);
  }

  res.json(health);
});

router.get('/health/ready', (req, res) => {
  // Check if application is ready to receive traffic
  const ready = {
    ready: true,
    timestamp: new Date().toISOString()
  };

  // Add any readiness checks here
  // For example, check if all required services are initialized

  if (!ready.ready) {
    return res.status(503).json(ready);
  }

  res.json(ready);
});

module.exports = router;
```

### 7. Production Optimizations

Update `src/index.js` with production optimizations:

```javascript
const express = require('express');
const helmet = require('helmet');
const cors = require('cors');
const morgan = require('morgan');
const compression = require('compression');
const config = require('./config');

const app = express();

// Trust proxy (important for deployment behind reverse proxies)
if (config.isProduction) {
  app.set('trust proxy', 1);
}

// Compression middleware for production
if (config.isProduction) {
  app.use(compression());
}

// Security middleware
app.use(helmet({
  contentSecurityPolicy: config.isProduction ? undefined : false
}));

// CORS configuration
app.use(cors({
  origin: config.cors.origins,
  credentials: true
}));

// Logging
if (config.isProduction) {
  app.use(morgan('combined'));
} else {
  app.use(morgan('dev'));
}

// Body parsing
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Static files with caching
app.use(express.static('public', {
  maxAge: config.isProduction ? '1d' : 0
}));

// Routes
app.use('/health', require('./routes/health'));
app.use('/auth', require('./routes/auth'));
app.use('/api', require('./middleware/authenticate'), require('./routes/api'));

// 404 handler
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Error handler
app.use((err, req, res, next) => {
  const status = err.status || 500;
  const message = config.isProduction && status === 500 
    ? 'Internal server error' 
    : err.message;

  // Log error details
  console.error('Error:', {
    message: err.message,
    status,
    stack: err.stack,
    url: req.url,
    method: req.method
  });

  res.status(status).json({
    error: message,
    ...(config.isDevelopment && { stack: err.stack })
  });
});

// Graceful shutdown
const server = app.listen(config.port, config.host, () => {
  console.log(`Server running on http://${config.host}:${config.port}`);
  console.log(`Environment: ${config.env}`);
});

process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  console.log('SIGINT received, shutting down gracefully');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});

module.exports = app;
```

### 8. Create GitHub Actions Workflow

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy

on:
  push:
    branches: [main, production]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run tests
        run: npm test
      
      - name: Run linter
        run: npm run lint

  build:
    needs: test
    runs-on: ubuntu-latest
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
      
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha
      
      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' || github.ref == 'refs/heads/production'
    
    steps:
      - name: Deploy to production
        run: |
          echo "Deploy to your hosting platform"
          # Add your deployment commands here
          # Examples:
          # - SSH to server and pull new image
          # - Trigger webhook for deployment service
          # - Update Kubernetes deployment
          # - Deploy to cloud platform (AWS, GCP, Azure)
```

### 9. Add npm Scripts

Update `package.json` with deployment scripts:

```json
{
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js",
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "lint": "eslint src/",
    "lint:fix": "eslint src/ --fix",
    "docker:build": "docker build -t express-app .",
    "docker:run": "docker run -p 3000:3000 --env-file .env express-app",
    "docker:dev": "docker-compose up",
    "docker:prod": "docker-compose -f docker-compose.prod.yml up -d",
    "docker:down": "docker-compose down",
    "migrate": "node scripts/migrate.js",
    "seed": "node scripts/seed.js"
  }
}
```

### 10. Create Deployment Documentation

Create `DEPLOYMENT.md`:

```markdown
# Deployment Guide

## Prerequisites

- Docker and Docker Compose installed
- Environment variables configured
- SSL certificates (for HTTPS)

## Local Deployment

1. Build the Docker image:
   ```bash
   npm run docker:build
   ```

2. Run with Docker Compose:
   ```bash
   npm run docker:prod
   ```

## Cloud Deployment Options

### Option 1: Deploy to Heroku

1. Install Heroku CLI
2. Create Heroku app:
   ```bash
   heroku create your-app-name
   ```

3. Set environment variables:
   ```bash
   heroku config:set JWT_SECRET=your-secret
   heroku config:set JWT_REFRESH_SECRET=your-refresh-secret
   ```

4. Deploy:
   ```bash
   git push heroku main
   ```

### Option 2: Deploy to AWS ECS

1. Build and push to ECR
2. Create ECS task definition
3. Create ECS service
4. Configure load balancer

### Option 3: Deploy to DigitalOcean App Platform

1. Connect GitHub repository
2. Configure environment variables
3. Set build and run commands
4. Deploy

## Environment Variables

Required for production:
- NODE_ENV=production
- JWT_SECRET
- JWT_REFRESH_SECRET
- DATABASE_URL (if using external database)

## Health Checks

The application provides health check endpoints:
- `/health` - Basic health status
- `/health/ready` - Readiness probe

## Monitoring

Consider adding:
- Application Performance Monitoring (APM)
- Error tracking (Sentry)
- Logging aggregation (ELK stack)
- Metrics collection (Prometheus)

## Security Checklist

- [ ] Environment variables secured
- [ ] HTTPS enabled
- [ ] Rate limiting configured
- [ ] CORS properly configured
- [ ] Security headers enabled
- [ ] Database backups configured
- [ ] Monitoring alerts set up
```

## File Structure

After implementation:
```
.
├── .github/
│   └── workflows/
│       └── deploy.yml
├── src/
│   ├── config/
│   │   └── index.js
│   └── routes/
│       └── health.js
├── .dockerignore
├── Dockerfile
├── docker-compose.yml
├── docker-compose.prod.yml
├── .env.example
└── DEPLOYMENT.md
```

## Testing Strategy

1. **Build Testing**: Verify Docker image builds successfully
2. **Container Testing**: Test container runs and responds
3. **Environment Testing**: Verify environment variable handling
4. **Health Check Testing**: Ensure health endpoints work
5. **Production Simulation**: Test with production configuration locally

## Dependencies

```json
{
  "dependencies": {
    "compression": "^1.7.4",
    "dotenv": "^16.4.5"
  }
}
```

## Common Issues and Solutions

1. **Port conflicts**: Ensure PORT environment variable is set correctly
2. **Database persistence**: Mount volume for SQLite data directory
3. **Environment variables**: Use .env file or container orchestration secrets
4. **Memory limits**: Set appropriate container memory limits
5. **Signal handling**: Use dumb-init for proper shutdown

## Next Steps

- Set up monitoring and alerting
- Configure automated backups
- Implement blue-green deployments
- Add container orchestration (Kubernetes)
- Set up CDN for static assets