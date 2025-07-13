# Task 1: Setup Basic Express Server

## Overview

This task creates a minimal Express.js server with a single health check endpoint. This serves as a simple test case for the Task Master workflow and Claude Code execution.

## Task Context

### Description
Create a simple Express.js server with one health check endpoint

### Priority
High - This is the foundation for the example application

### Dependencies
None - This is the initial setup task

### Subtasks
None - This is a simple, single-step task

## Implementation Details

### 1. Initialize Node.js Project

Create a new Node.js project:
```bash
npm init -y
```

### 2. Install Express

Install Express.js as a dependency:
```bash
npm install express
```

### 3. Create Basic Server

Create `server.js` with the following content:
```javascript
const express = require('express');
const app = express();
const port = process.env.PORT || 3000;

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    message: 'Express server is running!'
  });
});

// Start server
app.listen(port, () => {
  console.log(`Server running on port ${port}`);
  console.log(`Health check available at http://localhost:${port}/health`);
});
```

### 4. Add Start Script

Update `package.json` to include a start script:
```json
{
  "scripts": {
    "start": "node server.js"
  }
}
```

## Expected Output

After completing this task:
1. A working Express.js server that starts on port 3000
2. A `/health` endpoint that returns JSON with status information
3. Console logs showing the server has started
4. The ability to run `npm start` to launch the server

## Verification

Test the implementation by:
1. Running `npm start`
2. Making a GET request to `http://localhost:3000/health`
3. Verifying the response contains the expected JSON structure
4. Confirming console logs show the server started successfully