# Task 1: Setup Project Structure and Environment

## Overview

This task establishes the foundational project structure for a modern chat application using React frontend and Node.js backend in a monorepo architecture. The setup includes TypeScript configuration, development tooling, containerization with Docker, and preparation for Kubernetes deployment.

## Prerequisites

- Node.js 18+ and npm/yarn installed
- Docker and Docker Compose installed
- Git installed
- Basic understanding of TypeScript, React, and Node.js

## Implementation Guide

### 1. Research Phase

Before implementation, conduct thorough research on:

- **React + Node.js Monorepo Best Practices**: Modern approaches to structuring full-stack JavaScript applications
- **Containerization Patterns**: Docker best practices for Node.js applications
- **Kubernetes Readiness**: Container configuration patterns that facilitate K8s deployment

### 2. Project Structure Setup

Create the following directory structure:

```bash
/chat-application
├── frontend/                 # React application
│   ├── src/
│   │   ├── components/      # Reusable UI components
│   │   ├── pages/          # Page-level components
│   │   ├── services/       # API service layer
│   │   ├── hooks/          # Custom React hooks
│   │   ├── utils/          # Utility functions
│   │   └── types/          # TypeScript type definitions
│   ├── public/             # Static assets
│   ├── package.json
│   └── tsconfig.json
├── backend/                 # Node.js/Express API
│   ├── src/
│   │   ├── controllers/    # Request handlers
│   │   ├── models/         # Data models
│   │   ├── routes/         # API route definitions
│   │   ├── services/       # Business logic
│   │   ├── middleware/     # Express middleware
│   │   ├── utils/          # Utility functions
│   │   └── types/          # TypeScript type definitions
│   ├── package.json
│   └── tsconfig.json
├── kubernetes/              # K8s configurations
│   └── deployment-configs/
├── shared/                  # Shared code between frontend/backend
│   ├── types/              # Shared TypeScript types
│   └── utils/              # Shared utilities
├── docker-compose.yml       # Development orchestration
├── .gitignore
├── .eslintrc.js            # ESLint configuration
├── .prettierrc             # Prettier configuration
└── README.md
```

### 3. Frontend Configuration (React + TypeScript)

#### Using Vite (Recommended for modern development):

```bash
cd frontend
npm create vite@latest . -- --template react-ts
```

#### Configure TypeScript (tsconfig.json):

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"],
      "@shared/*": ["../shared/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### 4. Backend Configuration (Node.js + Express + TypeScript)

#### Initialize and configure:

```bash
cd backend
npm init -y
npm install express cors dotenv helmet morgan
npm install -D typescript @types/node @types/express @types/cors nodemon ts-node
```

#### TypeScript Configuration (tsconfig.json):

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "moduleResolution": "node",
    "allowSyntheticDefaultImports": true,
    "paths": {
      "@/*": ["./src/*"],
      "@shared/*": ["../shared/*"]
    }
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### 5. Development Tooling Setup

#### ESLint Configuration (.eslintrc.js):

```javascript
module.exports = {
  root: true,
  env: {
    browser: true,
    es2020: true,
    node: true
  },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react-hooks/recommended',
    'prettier'
  ],
  ignorePatterns: ['dist', '.eslintrc.js'],
  parser: '@typescript-eslint/parser',
  plugins: ['react-refresh'],
  rules: {
    'react-refresh/only-export-components': [
      'warn',
      { allowConstantExport: true }
    ],
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }]
  }
};
```

#### Prettier Configuration (.prettierrc):

```json
{
  "semi": true,
  "trailingComma": "none",
  "singleQuote": true,
  "printWidth": 80,
  "tabWidth": 2,
  "useTabs": false,
  "bracketSpacing": true,
  "arrowParens": "avoid"
}
```

### 6. Docker Configuration

#### Frontend Dockerfile:

```dockerfile
# Build stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

#### Backend Dockerfile:

```dockerfile
# Build stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY --from=builder /app/dist ./dist
EXPOSE 3000
CMD ["node", "dist/index.js"]
```

#### Docker Compose (docker-compose.yml):

```yaml
version: '3.8'

services:
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile.dev
    ports:
      - "5173:5173"
    volumes:
      - ./frontend:/app
      - /app/node_modules
    environment:
      - VITE_API_URL=http://localhost:3000/api
    depends_on:
      - backend

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    volumes:
      - ./backend:/app
      - /app/node_modules
    environment:
      - NODE_ENV=development
      - PORT=3000
      - DATABASE_URL=${DATABASE_URL}
    command: npm run dev
```

### 7. Kubernetes Preparation

Create basic deployment configurations that can be expanded later:

#### Backend Deployment (kubernetes/deployment-configs/backend-deployment.yaml):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chat-backend
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
        image: chat-backend:latest
        ports:
        - containerPort: 3000
        env:
        - name: NODE_ENV
          value: "production"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

### 8. Environment Configuration

Create environment files for different stages:

**.env.development**:
```env
NODE_ENV=development
PORT=3000
DATABASE_URL=postgresql://user:password@localhost:5432/chat_dev
JWT_SECRET=dev-secret-key
```

**.env.production**:
```env
NODE_ENV=production
PORT=3000
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
```

### 9. Package.json Scripts

#### Root package.json:
```json
{
  "name": "chat-application",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["frontend", "backend"],
  "scripts": {
    "dev": "docker-compose up",
    "dev:frontend": "cd frontend && npm run dev",
    "dev:backend": "cd backend && npm run dev",
    "build": "npm run build:frontend && npm run build:backend",
    "build:frontend": "cd frontend && npm run build",
    "build:backend": "cd backend && npm run build",
    "lint": "eslint . --ext .ts,.tsx",
    "format": "prettier --write \"**/*.{ts,tsx,js,json,md}\""
  }
}
```

### 10. Git Configuration

**.gitignore**:
```gitignore
# Dependencies
node_modules/
.pnp
.pnp.js

# Production
dist/
build/

# Environment
.env
.env.local
.env.development.local
.env.test.local
.env.production.local

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
.idea/
*.swp
*.swo

# Testing
coverage/
.nyc_output/

# Docker
*.pid
```

## Best Practices

1. **Monorepo Management**: Use npm workspaces or yarn workspaces for dependency management
2. **Code Sharing**: Utilize the shared folder for common types and utilities
3. **Environment Variables**: Never commit sensitive data; use .env files and secrets management
4. **Container Optimization**: Use multi-stage builds to reduce image size
5. **Development Experience**: Ensure hot reloading works for rapid development
6. **Type Safety**: Leverage TypeScript's strict mode for better code quality
7. **Linting**: Run linters in pre-commit hooks to maintain code consistency

## Verification Steps

1. Verify all directories are created correctly
2. Test that both frontend and backend start without errors
3. Confirm hot reloading works in development mode
4. Validate Docker containers build and run successfully
5. Check that environment variables are properly configured
6. Ensure linting and formatting tools work correctly