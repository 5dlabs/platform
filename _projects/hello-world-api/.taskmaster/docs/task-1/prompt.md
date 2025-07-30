# Task 1: Setup Project Structure and Dependencies - Autonomous Agent Prompt

You are tasked with initializing a new Node.js project for a Hello World REST API. This is the foundation task that sets up the project structure and installs all necessary dependencies.

## Your Mission
Create a complete Node.js project setup with Express.js framework, including all required dependencies, folder structure, and configuration files. The project should follow best practices for a production-ready REST API.

## Step-by-Step Instructions

### 1. Initialize the Project
- Create a new directory named `hello-world-api`
- Navigate into the directory
- Run `npm init -y` to create a default package.json

### 2. Install Dependencies
Install the following exact versions to ensure compatibility:

**Core Dependencies:**
```bash
npm install express@4.18.2 cors@2.8.5 helmet@7.0.0 pino@8.15.0 pino-http@8.5.0 dotenv@16.3.1
```

**Development Dependencies:**
```bash
npm install --save-dev jest@29.6.4 supertest@6.3.3 nodemon@3.0.1 eslint@8.48.0 swagger-jsdoc@6.2.8 swagger-ui-express@5.0.0
```

### 3. Create Project Structure
Create the following directories and files:

```
hello-world-api/
├── src/
│   ├── middleware/      (directory)
│   ├── routes/          (directory)
│   ├── utils/           (directory)
│   ├── app.js          (file)
│   └── server.js       (file)
├── tests/
│   ├── unit/           (directory)
│   └── integration/    (directory)
├── docs/
│   └── openapi.yaml    (file)
├── .env               (file)
├── .dockerignore      (file)
├── .gitignore         (file)
├── Dockerfile         (file)
├── kubernetes.yaml    (file)
└── README.md          (file)
```

### 4. Configure package.json
Update the package.json file with these exact scripts:

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

### 5. Create Configuration Files

**.env file:**
```
PORT=3000
NODE_ENV=development
API_VERSION=1.0.0
LOG_LEVEL=info
SERVICE_NAME=hello-world-api
```

**.gitignore file:**
```
node_modules/
.env
.env.local
.env.*.local
coverage/
logs/
*.log
.vscode/
.idea/
*.swp
*.swo
.DS_Store
Thumbs.db
```

**.dockerignore file:**
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

### 6. Create Initial Application Files

**src/app.js:**
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

**src/server.js:**
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

### 7. Create a Basic README.md
```markdown
# Hello World API

A simple REST API built with Node.js and Express.js for testing the 5D Labs orchestrator workflow.

## Installation

\`\`\`bash
npm install
\`\`\`

## Running the Application

Development mode:
\`\`\`bash
npm run dev
\`\`\`

Production mode:
\`\`\`bash
npm start
\`\`\`

## Testing

\`\`\`bash
npm test
\`\`\`

## API Documentation

API documentation will be available at `/docs` endpoint once implemented.
```

## Verification Steps
After completing all steps:

1. Run `npm install` to ensure all dependencies are properly installed
2. Run `npm run dev` to verify the server starts without errors
3. Check that the server prints "Server running on port 3000"
4. Verify all directories and files exist in the correct structure
5. Ensure package.json contains all specified scripts

## Important Notes
- Use exact dependency versions specified to ensure compatibility
- Create all directories even if they're initially empty
- The .env file should not be committed to version control
- Ensure graceful shutdown handling is implemented in server.js
- The project structure should support future scalability

## Success Criteria
The task is complete when:
- All dependencies are installed with correct versions
- Complete folder structure is created
- All configuration files exist with proper content
- Server can start successfully with `npm run dev`
- Package.json contains all required scripts
- Basic Express app is configured with security middleware