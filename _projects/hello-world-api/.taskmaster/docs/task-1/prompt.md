# Task 1: Setup Project Structure and Dependencies - Autonomous Agent Prompt

## Task Overview
Initialize a new Node.js project for the Hello World API service. Set up the complete project structure, install all necessary dependencies, and configure the development environment.

## Execution Instructions

### 1. Project Initialization
Create a new directory named `hello-world-api` and initialize it as a Node.js project:
```bash
mkdir hello-world-api
cd hello-world-api
npm init -y
```

### 2. Dependency Installation
Install the exact versions of all required dependencies:

**Core Dependencies:**
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

**Development Dependencies:**
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### 3. Create Project Structure
Create the following directory structure:
```
/src
  /middleware
  /routes
  /utils
  app.js
  server.js
/tests
  /unit
  /integration
/docs
  openapi.yaml
.env
.dockerignore
Dockerfile
kubernetes.yaml
README.md
```

Use these commands:
```bash
mkdir -p src/{middleware,routes,utils}
mkdir -p tests/{unit,integration}
mkdir docs
touch src/{app.js,server.js}
touch docs/openapi.yaml
touch {.env,.dockerignore,Dockerfile,kubernetes.yaml,README.md}
```

### 4. Configure package.json
Update the `package.json` file to include the following scripts section:
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
  "keywords": ["api", "rest", "express", "hello-world"],
  "author": "",
  "license": "ISC"
}
```

### 5. Create Environment Configuration
Create `.env` file with the following content:
```
PORT=3000
NODE_ENV=development
LOG_LEVEL=info
API_VERSION=1.0.0
```

### 6. Create .gitignore
Create `.gitignore` file with:
```
# Dependencies
node_modules/

# Environment files
.env
.env.local
.env.*.local

# Test coverage
coverage/
*.lcov

# Logs
logs/
*.log
npm-debug.log*

# IDE files
.vscode/
.idea/
*.swp
*.swo
.DS_Store

# Build outputs
dist/
build/
```

### 7. Create .dockerignore
Create `.dockerignore` file with:
```
node_modules/
npm-debug.log
.git/
.gitignore
README.md
.env
coverage/
.nyc_output/
tests/
.eslintrc.js
.vscode/
.idea/
```

### 8. Create Basic README.md
Create `README.md` with initial content:
```markdown
# Hello World API

A simple REST API built with Node.js and Express.js for testing the 5D Labs orchestrator workflow.

## Installation

\`\`\`bash
npm install
\`\`\`

## Development

\`\`\`bash
npm run dev
\`\`\`

## Testing

\`\`\`bash
npm test
\`\`\`

## API Endpoints

- GET /health - Health check
- GET /hello - Basic greeting
- GET /hello/:name - Personalized greeting
- POST /echo - Echo service
- GET /info - Service information
```

### 9. Verification Steps
After completing all steps above, verify the setup:

1. Check all dependencies are installed:
   ```bash
   npm list --depth=0
   ```

2. Verify the folder structure:
   ```bash
   find . -type d -name "node_modules" -prune -o -type d -print | sort
   ```

3. Test that the dev script is configured (it will fail since server.js is empty, but npm should recognize the script):
   ```bash
   npm run dev
   ```

## Expected Outcomes
- A properly structured Node.js project with Express.js
- All dependencies installed with exact versions specified
- Development environment configured with hot reloading
- Basic project documentation in place
- Project ready for API endpoint implementation

## Common Issues and Solutions
- If npm install fails, check your Node.js version (should be 16.x or higher)
- If permission errors occur, avoid using sudo; instead fix npm permissions
- Ensure you're in the correct directory when creating files and folders

## Success Verification
The task is complete when:
1. All dependencies are installed without errors
2. The complete folder structure exists as specified
3. All configuration files are created
4. The package.json contains all required scripts
5. Running `npm run dev` attempts to start the server (will fail due to empty server.js, but the script should execute)