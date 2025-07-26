# Autonomous Agent Prompt: Initialize Node.js Project

You are tasked with setting up a new Node.js project for a Hello World API using Express.js.

## Your Mission
Create a properly structured Node.js project with Express.js installed and configured, ready for API development.

## Required Actions

### 1. Project Setup
- Create a new directory named `hello-world-api`
- Navigate into the directory
- Initialize a Node.js project using `npm init -y`

### 2. Configure package.json
Update the package.json file with:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple Hello World API",
  "main": "src/index.js",
  "scripts": {
    "start": "node src/index.js"
  },
  "dependencies": {
    "express": "^4.18.2"
  }
}
```

### 3. Install Express.js
Run: `npm install express`

### 4. Create Project Structure
```
hello-world-api/
├── src/
│   └── index.js
├── package.json
├── .gitignore
└── README.md
```

### 5. Implement Basic Server
Create `src/index.js` with:
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

### 6. Create .gitignore
Add the following to .gitignore:
```
node_modules/
.env
*.log
.DS_Store
```

### 7. Create Basic README
Create README.md with:
```markdown
# Hello World API

A simple Express.js API that responds with "Hello, World!"

## Installation
\`\`\`bash
npm install
\`\`\`

## Running the Server
\`\`\`bash
npm start
\`\`\`

The server will start on port 3000 by default.
```

## Validation Steps
1. Run `npm list` to verify Express.js is installed
2. Run `npm start` to test server startup
3. Confirm server logs "Server running on http://localhost:3000"
4. Make a test request and verify request logging works
5. Stop the server with Ctrl+C

## Success Criteria
- Project directory exists with correct structure
- package.json is properly configured
- Express.js is installed
- Server starts without errors
- Request logging is functional
- All required files are created

## Error Handling
- If port 3000 is in use, the server should use PORT environment variable
- If npm install fails, check network connectivity
- Ensure Node.js version 20+ is installed

Complete all steps in order and verify each one before proceeding to the next.