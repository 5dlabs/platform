# Autonomous Task Prompt: Initialize Node.js Project

You are tasked with initializing a new Node.js project for a Hello World API. This is the foundational step that sets up the project structure and installs necessary dependencies.

## Task Requirements

### 1. Create Project Directory
- Create a new directory named `hello-world-api`
- Navigate into this directory for all subsequent operations

### 2. Initialize Node.js Project
- Run `npm init -y` to create a default package.json
- Modify the package.json with the following updates:
  - name: "hello-world-api"
  - version: "1.0.0"
  - description: "A simple Hello World API"
  - main: "src/index.js"
  - Add scripts section with: `"start": "node src/index.js"`

### 3. Install Dependencies
- Install Express.js using: `npm install express`
- Ensure the version is ^4.18.2 or compatible

### 4. Create Project Structure
```
hello-world-api/
├── src/
│   └── index.js (create empty file)
├── package.json
├── package-lock.json
├── .gitignore
└── README.md
```

### 5. Configure Git Ignore
Create `.gitignore` with:
```
node_modules/
.env
```

### 6. Create Initial README
Create a basic README.md with:
```markdown
# Hello World API

A simple Express.js API that responds with "Hello, World!"

## Installation
npm install

## Usage
npm start
```

### 7. Create Basic Server Setup
In `src/index.js`, add:
```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Server startup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

## Verification Steps
1. Confirm package.json has correct structure and dependencies
2. Run `npm list` to verify Express.js installation
3. Test `npm start` - server should start on port 3000
4. Make a test request to http://localhost:3000 (expect 404 but see log)
5. Verify all files exist in correct structure

## Expected Outcome
- Fully initialized Node.js project
- Express.js installed and configured
- Basic server that starts successfully
- Proper project structure established
- All configuration files in place

## Error Handling
- If npm init fails, ensure Node.js and npm are installed
- If Express install fails, check internet connection and npm registry
- If server doesn't start, check for port conflicts
- Ensure src directory exists before creating index.js

Complete all steps in order and verify each step before proceeding to the next.