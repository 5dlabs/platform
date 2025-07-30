# Task 1: Setup Project Structure and Dependencies

## Overview
This task establishes the foundation for the Hello World API by initializing a Node.js project with Express.js framework and setting up a scalable project structure. It includes installing all necessary dependencies and creating the initial directory layout.

## Technical Implementation Guide

### 1. Project Initialization

Start by creating the project directory and initializing Node.js:

```bash
mkdir hello-world-api
cd hello-world-api
npm init -y
```

This creates a basic `package.json` file with default settings.

### 2. Dependency Installation

Install the required dependencies in two categories:

**Core Dependencies:**
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

- **express**: Web framework for building the REST API
- **cors**: Cross-Origin Resource Sharing middleware
- **helmet**: Security headers middleware
- **pino & pino-http**: High-performance JSON logger
- **dotenv**: Environment variable management

**Development Dependencies:**
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

- **jest**: Testing framework
- **supertest**: HTTP assertion library for API testing
- **nodemon**: Automatic server restart during development
- **eslint**: Code linting
- **swagger-jsdoc & swagger-ui-express**: API documentation

### 3. Project Structure Creation

Create the following directory structure:

```bash
# Create directories
mkdir -p src/{middleware,routes,utils}
mkdir -p tests/{unit,integration}
mkdir docs

# Create initial files
touch src/app.js
touch src/server.js
touch docs/openapi.yaml
touch .env
touch .dockerignore
touch Dockerfile
touch kubernetes.yaml
touch README.md
touch .gitignore
```

The structure provides clear separation of concerns:
- `src/`: Source code
  - `middleware/`: Express middleware components
  - `routes/`: API route definitions
  - `utils/`: Utility functions and helpers
- `tests/`: Test files organized by type
- `docs/`: Documentation including OpenAPI spec

### 4. Package.json Configuration

Update `package.json` with the following scripts:

```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple REST API for testing 5D Labs orchestrator workflow",
  "main": "src/server.js",
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint ."
  },
  "engines": {
    "node": ">=16.0.0"
  }
}
```

### 5. Environment Configuration

Create `.env` file with initial configuration:

```env
# Server Configuration
PORT=3000
NODE_ENV=development

# API Configuration
API_VERSION=1.0.0
LOG_LEVEL=info

# Service Information
SERVICE_NAME=hello-world-api
```

### 6. Git Configuration

Create `.gitignore` file:

```
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Test coverage
coverage/

# Logs
logs/
*.log

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
```

Create `.dockerignore` file:

```
node_modules/
npm-debug.log
.env
.env.*
.git/
.gitignore
README.md
.vscode/
.idea/
coverage/
.nyc_output/
tests/
docs/
*.log
```

### 7. Initial File Contents

Create minimal `src/app.js`:

```javascript
const express = require('express');
const cors = require('cors');
const helmet = require('helmet');

const app = express();

// Security middleware
app.use(helmet());
app.use(cors());

// Body parsing
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

module.exports = app;
```

Create `src/server.js`:

```javascript
require('dotenv').config();
const app = require('./app');

const PORT = process.env.PORT || 3000;

const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM signal received: closing HTTP server');
  server.close(() => {
    console.log('HTTP server closed');
  });
});

module.exports = server;
```

## Key Considerations

### Dependency Versions
Specific versions are locked to ensure compatibility and stability. These versions have been tested together and provide a stable foundation.

### Project Structure Rationale
The structure follows Express.js best practices:
- Separation of app configuration (`app.js`) from server startup (`server.js`)
- Organized middleware, routes, and utilities
- Clear test organization for unit and integration tests
- Documentation co-located with code

### Security Considerations
- Helmet.js provides essential security headers
- CORS configuration allows controlled cross-origin access
- Environment variables keep sensitive data out of code
- `.dockerignore` prevents sensitive files from entering containers

### Development Workflow
The npm scripts support a complete development workflow:
- `npm run dev`: Development with auto-restart
- `npm test`: Run tests with coverage
- `npm run lint`: Code quality checks
- `npm start`: Production server startup

## Next Steps
After completing this task, the project will have:
1. A properly initialized Node.js project
2. All required dependencies installed
3. A scalable folder structure
4. Basic configuration files
5. Development scripts ready

This foundation enables the implementation of API endpoints in subsequent tasks.