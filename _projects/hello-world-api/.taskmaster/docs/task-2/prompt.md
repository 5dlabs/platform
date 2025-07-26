# Autonomous Agent Prompt: Create Express.js Server

You are tasked with implementing the core Express.js server for the Hello World API project.

## Your Mission
Create a fully functional Express.js server with request logging middleware and basic route handling.

## Prerequisites
- Ensure Task 1 is completed (Node.js project initialized with Express installed)
- Verify src/ directory exists

## Required Actions

### 1. Create Main Server File
Create `src/index.js` with the following structure:

```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Your middleware and routes will go here

app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

### 2. Implement Request Logging Middleware
Add middleware that logs every incoming request with:
- ISO timestamp
- HTTP method
- Request URL

```javascript
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});
```

### 3. Add Basic Route Handler
Implement a root route that confirms the server is running:

```javascript
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});
```

### 4. Implement 404 Handler
Add a catch-all middleware for undefined routes:

```javascript
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});
```

### 5. Complete File Structure
Your final `src/index.js` should have this order:
1. Imports and constants
2. Request logging middleware
3. Route handlers
4. 404 error handler
5. Server listener

## Validation Steps

### Test 1: Server Startup
```bash
npm start
```
Expected output: `Server running on http://localhost:3000`

### Test 2: Request Logging
Make a request:
```bash
curl http://localhost:3000
```
Expected console log: `2024-01-26T10:30:45.123Z - GET /`

### Test 3: Root Endpoint
```bash
curl -i http://localhost:3000
```
Expected:
- Status: 200 OK
- Body: "Server is running"

### Test 4: 404 Handler
```bash
curl -i http://localhost:3000/unknown
```
Expected:
- Status: 404 Not Found
- Body: `{"error":"Not found"}`

## Success Criteria
- Server starts without errors
- All requests are logged to console
- Root path returns 200 status
- Unknown paths return 404 status
- No unhandled errors or warnings

## Common Issues
1. **Port already in use**: Kill existing process or use different port
2. **Cannot find module 'express'**: Run `npm install`
3. **Syntax errors**: Check for missing semicolons or brackets

## Code Quality Requirements
- Use consistent indentation (2 spaces)
- Add meaningful comments
- Follow Express.js conventions
- Keep middleware functions concise

Complete all steps and run all validation tests before marking this task as complete.