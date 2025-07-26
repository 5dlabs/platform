# Autonomous Agent Prompt: Create Express.js Server

You are an autonomous agent tasked with implementing a basic Express.js server. Your goal is to create a functional web server with request logging capabilities that serves as the foundation for the Hello World API.

## Prerequisites

Before starting, verify:
- Node.js project is initialized (package.json exists)
- Express.js is installed (`npm list express` shows it's installed)
- The `src/index.js` file exists

## Task Requirements

### 1. Implement Basic Express Server

Create or update `src/index.js` with the following structure:

```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Your middleware and routes will go here

// Server startup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

### 2. Add Request Logging Middleware

Implement middleware that logs every incoming request with:
- ISO format timestamp
- HTTP method (GET, POST, etc.)
- Request URL

```javascript
// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});
```

### 3. Add Basic Route

Add a placeholder route to test the server:

```javascript
// Basic route placeholder
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
});
```

### 4. Implement 404 Handler

Add a catch-all middleware for undefined routes:

```javascript
// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});
```

### 5. Complete Server Implementation

The final `src/index.js` should have this structure:
1. Express import and app creation
2. Port configuration
3. Logging middleware (must be first)
4. Route handlers
5. 404 handler (must be last)
6. Server listen call

## Validation Steps

1. **Start the server**:
   ```bash
   npm start
   ```
   Verify output: "Server running on http://localhost:3000"

2. **Test request logging**:
   ```bash
   curl http://localhost:3000
   ```
   Verify server logs show: `2024-XX-XX...Z - GET /`

3. **Test 404 handling**:
   ```bash
   curl http://localhost:3000/nonexistent
   ```
   Verify response: `{"error":"Not found"}` with status 404

4. **Test different HTTP methods**:
   ```bash
   curl -X POST http://localhost:3000
   curl -X PUT http://localhost:3000
   ```
   Verify each request is logged with correct method

## Expected Behavior

- Server starts without errors on port 3000
- Every HTTP request is logged to console
- Root path (/) returns "Server is running" with status 200
- Unknown paths return JSON error with status 404
- Server can be stopped with Ctrl+C

## Error Handling

Common issues to check:
- If port 3000 is in use, the server will fail to start
- If Express is not installed, require('express') will fail
- If middleware order is wrong, logging might not work for all requests

## Code Quality Requirements

- Use meaningful variable names
- Add comments to explain middleware purpose
- Maintain consistent indentation
- Ensure all callbacks include proper error handling
- Use appropriate HTTP status codes

Complete the implementation and verify all validation steps pass before considering the task complete.