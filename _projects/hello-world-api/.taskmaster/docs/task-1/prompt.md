# Task 1: Setup Project Structure and Dependencies - AI Agent Prompt

You are tasked with setting up the initial project structure and dependencies for a Node.js/Express.js REST API called "Hello World API". This is the foundational task that establishes the project's architecture and development environment.

## Your Mission

Initialize a new Node.js project with Express.js framework, install all necessary dependencies, and create a well-organized project structure that follows industry best practices for scalable REST API development.

## Step-by-Step Instructions

### 1. Project Initialization
- Create a new directory called `hello-world-api`
- Navigate into the directory
- Initialize a new Node.js project using `npm init -y`

### 2. Install Dependencies

Install the following core dependencies with exact versions:
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

Install the following development dependencies:
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### 3. Create Project Structure

Create the following directory structure:
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
├── .gitignore
├── Dockerfile
├── kubernetes.yaml
├── package.json
└── README.md
```

### 4. Configure package.json

Update the package.json file with the following configuration:
- Set main entry point to `"src/server.js"`
- Add description: `"A simple REST API for testing 5D Labs orchestrator workflow"`
- Add the following scripts:
  ```json
  {
    "start": "node src/server.js",
    "dev": "nodemon src/server.js",
    "test": "jest --coverage",
    "test:watch": "jest --watch",
    "test:unit": "jest tests/unit",
    "test:integration": "jest tests/integration",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix"
  }
  ```
- Add Jest configuration:
  ```json
  {
    "jest": {
      "testEnvironment": "node",
      "coverageDirectory": "coverage",
      "collectCoverageFrom": [
        "src/**/*.js",
        "!src/server.js"
      ]
    }
  }
  ```
- Add engines requirement: `"node": ">=16.0.0"`

### 5. Create Environment Configuration

Create a `.env` file with the following default configuration:
```
# Server Configuration
PORT=3000
NODE_ENV=development

# Logging
LOG_LEVEL=info

# API Configuration
API_VERSION=1.0.0
API_PREFIX=/api/v1
```

### 6. Configure Ignore Files

Create `.gitignore` with standard patterns for Node.js projects including:
- node_modules/
- .env files
- logs/
- coverage/
- IDE files (.vscode/, .idea/)
- OS files (.DS_Store, Thumbs.db)

Create `.dockerignore` with patterns to exclude from Docker builds:
- node_modules/
- test files
- development configurations
- documentation files

### 7. Create Initial Files

Create empty placeholder files:
- `src/app.js` - Will contain Express app configuration
- `src/server.js` - Will contain server startup logic
- `docs/openapi.yaml` - Will contain API documentation
- `Dockerfile` - Will contain container definition
- `kubernetes.yaml` - Will contain K8s deployment specs
- `README.md` - Will contain project documentation

## Validation Steps

After completing the setup, verify:
1. Run `npm install` to ensure all dependencies are installed
2. Check that all directories and files exist in the correct structure
3. Verify package.json has all required scripts
4. Test that `npm run dev` doesn't throw errors (it's okay if the server doesn't start yet)
5. Ensure .env file is created with all required variables
6. Confirm .gitignore and .dockerignore files are properly configured

## Expected Outcome

A fully initialized Node.js project with:
- All required dependencies installed
- Proper project structure for a scalable REST API
- Development scripts configured
- Environment configuration ready
- Project ready for implementing API endpoints

## Important Notes

- Use exact dependency versions specified to ensure compatibility
- Create all directories even if they're empty initially
- Don't write any actual code logic yet - just create the structure
- Ensure all configuration files use proper formatting
- The project should be ready for the next developer to start implementing features

This foundational setup enables efficient development and follows Node.js/Express.js best practices for API projects.