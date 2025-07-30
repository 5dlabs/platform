# Task 1: Setup Project Structure and Dependencies

## Overview
This task establishes the foundation for the Hello World API project by initializing a Node.js/Express.js application with all necessary dependencies and creating a well-organized project structure.

## Prerequisites
- Node.js and npm installed on the development machine
- Basic understanding of Node.js package management
- Terminal/command line access

## Implementation Guide

### Step 1: Initialize the Node.js Project
Create the project directory and initialize npm:
```bash
# Create and navigate to project directory
mkdir hello-world-api
cd hello-world-api

# Initialize npm project with defaults
npm init -y
```

### Step 2: Install Core Dependencies
Install the production dependencies required for the API:
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

**Dependency purposes:**
- `express@4.18.2` - Web framework for building REST APIs
- `cors@2.8.5` - Enable Cross-Origin Resource Sharing
- `helmet@7.0.0` - Security headers middleware
- `pino@8.15.0` - High-performance JSON logger
- `pino-http@8.5.0` - HTTP request logging middleware
- `dotenv@16.3.1` - Environment variable management

### Step 3: Install Development Dependencies
Install development tools for testing, linting, and documentation:
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

**Development dependency purposes:**
- `jest@29.6.4` - Testing framework
- `supertest@6.3.3` - HTTP testing library
- `nodemon@3.0.1` - Development server with auto-reload
- `eslint@8.48.0` - Code linting
- `swagger-jsdoc@6.2.8` - Generate OpenAPI documentation from JSDoc
- `swagger-ui-express@5.0.0` - Serve interactive API documentation

### Step 4: Create Project Structure
Create the directory structure:
```bash
# Create source directories
mkdir -p src/middleware src/routes src/utils

# Create test directories
mkdir -p tests/unit tests/integration

# Create documentation directory
mkdir docs

# Create empty files
touch src/app.js src/server.js
touch docs/openapi.yaml
touch .env .dockerignore Dockerfile kubernetes.yaml README.md
```

Final structure:
```
hello-world-api/
├── src/
│   ├── middleware/
│   ├── routes/
│   ├── utils/
│   ├── app.js
│   └── server.js
├── tests/
│   ├── unit/
│   └── integration/
├── docs/
│   └── openapi.yaml
├── .env
├── .dockerignore
├── Dockerfile
├── kubernetes.yaml
├── README.md
└── package.json
```

### Step 5: Configure NPM Scripts
Update `package.json` with the following scripts section:
```json
{
  "scripts": {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "lint": "eslint ."
  }
}
```

### Step 6: Create Environment Configuration
Create `.env` file with basic configuration:
```env
# Server Configuration
PORT=3000
NODE_ENV=development

# API Configuration
API_VERSION=1.0.0
LOG_LEVEL=info
```

### Step 7: Configure .gitignore
Create `.gitignore` file:
```gitignore
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Logs
logs/
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Testing
coverage/
.nyc_output/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Build
dist/
build/
```

### Step 8: Configure .dockerignore
Create `.dockerignore` file:
```dockerignore
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
```

## Verification Steps
1. Run `npm install` to ensure all dependencies are correctly installed
2. Check that `node_modules` directory exists and contains all dependencies
3. Verify the project structure matches the specification
4. Test the development server: `npm run dev` (will fail initially as server.js is empty)
5. Ensure all configuration files are in place

## Common Issues and Solutions
- **npm install fails**: Check Node.js version compatibility (recommend Node.js 16+)
- **Permission errors**: Use appropriate user permissions or configure npm prefix
- **Missing files**: Ensure all `touch` commands executed successfully
- **Script errors**: Verify package.json syntax is valid JSON

## Next Steps
After completing this task:
1. Implement the Express application setup (app.js)
2. Create the server entry point (server.js)
3. Begin implementing API endpoints
4. Set up middleware components