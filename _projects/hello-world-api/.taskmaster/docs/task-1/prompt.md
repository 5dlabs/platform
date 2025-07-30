# Task 1: Setup Project Structure and Dependencies - Autonomous Agent Prompt

You are an expert Node.js developer tasked with setting up the initial project structure for a REST API called "Hello World API". This is the foundation task that establishes the project structure and installs all necessary dependencies.

## Your Mission
Initialize a new Node.js/Express.js project with a professional folder structure and all required dependencies for building a production-ready REST API.

## Detailed Instructions

### 1. Project Initialization
- Create a new directory called `hello-world-api` if it doesn't exist
- Navigate into the directory
- Initialize a new npm project using `npm init -y`

### 2. Dependency Installation
Install the following exact versions of production dependencies:
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

Then install development dependencies:
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### 3. Project Structure Creation
Create the following directory structure:
```
/src
  /middleware    (Express middleware components)
  /routes        (API route handlers)
  /utils         (Utility functions and helpers)
  app.js         (Express app configuration)
  server.js      (Server entry point)
/tests
  /unit          (Unit test files)
  /integration   (Integration test files)
/docs
  openapi.yaml   (OpenAPI specification)
.env             (Environment variables)
.dockerignore    (Docker ignore patterns)
Dockerfile       (Container definition)
kubernetes.yaml  (K8s deployment manifest)
README.md        (Project documentation)
```

### 4. Configure NPM Scripts
Update the `package.json` file to include these scripts:
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

### 5. Environment Configuration
Create a `.env` file with:
```
PORT=3000
NODE_ENV=development
API_VERSION=1.0.0
LOG_LEVEL=info
```

### 6. Git Ignore Configuration
Create `.gitignore` with standard Node.js patterns including:
- node_modules/
- .env files
- Log files
- Coverage reports
- IDE configurations
- OS-specific files

### 7. Docker Ignore Configuration
Create `.dockerignore` with patterns to exclude:
- node_modules/
- Test files
- Development configurations
- Git files
- Documentation files

## Success Criteria
- All dependencies are installed with exact versions specified
- Complete folder structure exists as defined
- Configuration files (.env, .gitignore, .dockerignore) are created
- NPM scripts are properly configured in package.json
- Running `npm install` completes without errors
- The project structure is ready for development

## Important Notes
- Use exact dependency versions as specified (do not use ^ or ~ prefixes)
- Create all directories and files even if they're initially empty
- Ensure proper file permissions for script execution
- The project should be initialized in a clean state ready for development

Execute all steps systematically and verify each step completes successfully before proceeding to the next.