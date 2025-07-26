# Acceptance Criteria: Create Express.js Server

## Overview
This document defines the acceptance criteria for Task 2: Create Express.js Server. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Server Implementation
- [ ] The `src/index.js` file contains a complete Express.js server implementation
- [ ] Express is properly imported using `require('express')`
- [ ] An Express application instance is created
- [ ] The server listens on port 3000
- [ ] Server startup logs a message to console

### 2. Request Logging Middleware
- [ ] A logging middleware function is implemented
- [ ] The middleware is registered with `app.use()` before any routes
- [ ] The middleware logs every incoming request
- [ ] Log format includes ISO timestamp
- [ ] Log format includes HTTP method
- [ ] Log format includes request URL
- [ ] The middleware calls `next()` to continue processing

### 3. Basic Route Implementation
- [ ] A GET route is defined for the root path '/'
- [ ] The route returns status code 200
- [ ] The route returns a response indicating the server is running
- [ ] The response is sent using appropriate Express methods

### 4. 404 Error Handling
- [ ] A catch-all middleware is implemented for undefined routes
- [ ] The 404 handler is placed after all defined routes
- [ ] The handler returns status code 404
- [ ] The handler returns a JSON response with error message
- [ ] The error response follows format: `{"error": "Not found"}`

### 5. Server Lifecycle
- [ ] Server starts successfully when running `npm start`
- [ ] Server displays startup message: "Server running on http://localhost:3000"
- [ ] Server can be stopped gracefully with Ctrl+C
- [ ] No errors are thrown during normal operation

## Test Cases

### Test Case 1: Server Startup
```bash
npm start
```
**Expected Output:**
```
Server running on http://localhost:3000
```

### Test Case 2: Request Logging - GET Request
```bash
# In a new terminal
curl http://localhost:3000
```
**Expected Server Log:**
```
2024-01-15T10:30:45.123Z - GET /
```

### Test Case 3: Request Logging - POST Request
```bash
curl -X POST http://localhost:3000
```
**Expected Server Log:**
```
2024-01-15T10:30:46.456Z - POST /
```

### Test Case 4: Basic Route Response
```bash
curl -v http://localhost:3000
```
**Expected Response:**
- Status: 200 OK
- Body: "Server is running"

### Test Case 5: 404 Handler
```bash
curl -v http://localhost:3000/nonexistent
```
**Expected Response:**
- Status: 404 Not Found
- Body: `{"error":"Not found"}`
- Content-Type: application/json

### Test Case 6: Multiple Sequential Requests
```bash
# Execute multiple requests
curl http://localhost:3000
curl http://localhost:3000/api
curl -X POST http://localhost:3000/data
curl -X PUT http://localhost:3000/update
curl -X DELETE http://localhost:3000/remove
```
**Expected:** Each request should be logged with correct timestamp, method, and URL

### Test Case 7: Request with Query Parameters
```bash
curl "http://localhost:3000/search?q=test&limit=10"
```
**Expected Server Log:**
```
2024-01-15T10:30:47.789Z - GET /search?q=test&limit=10
```

### Test Case 8: Server Port Configuration
```bash
# Verify server is listening on port 3000
lsof -i :3000  # Mac/Linux
# or
netstat -an | grep 3000  # Windows
```
**Expected:** Shows node process listening on port 3000

## Code Quality Criteria

### Structure Requirements
- [ ] Code follows consistent indentation (2 or 4 spaces)
- [ ] Middleware functions are properly structured with (req, res, next) parameters
- [ ] No unused variables or imports
- [ ] Appropriate use of const for constants

### Middleware Order
- [ ] Logging middleware is registered before any routes
- [ ] Routes are defined after middleware but before error handlers
- [ ] 404 handler is the last middleware registered

### Best Practices
- [ ] No hardcoded values (port defined as constant)
- [ ] Proper use of HTTP status codes
- [ ] JSON responses use appropriate Express methods
- [ ] Console logs use appropriate formatting

## Performance Criteria

- [ ] Server starts within 2 seconds
- [ ] Request logging adds minimal latency (<5ms)
- [ ] Server handles at least 100 requests/second (basic load)
- [ ] Memory usage remains stable during operation

## Security Criteria

- [ ] No sensitive information logged to console
- [ ] Error messages don't expose internal details
- [ ] Server doesn't crash on malformed requests
- [ ] Proper HTTP headers are set by Express

## Definition of Done

The task is complete when:
1. All acceptance criteria checkboxes are marked complete
2. All test cases pass successfully
3. Code follows the specified quality standards
4. Server runs stable for at least 5 minutes under test load
5. Another developer can start and test the server using only npm start

## Additional Validation

Run this validation script to verify implementation:

```javascript
// save as validate-server.js
const http = require('http');

console.log('Validating Express server...\n');

// Test 1: Server responds
http.get('http://localhost:3000', (res) => {
  console.log(`✓ Server responds: ${res.statusCode === 200 ? 'PASS' : 'FAIL'}`);
});

// Test 2: 404 handler works
http.get('http://localhost:3000/should-not-exist', (res) => {
  let data = '';
  res.on('data', chunk => data += chunk);
  res.on('end', () => {
    const is404 = res.statusCode === 404;
    const hasError = data.includes('error');
    console.log(`✓ 404 handler: ${is404 && hasError ? 'PASS' : 'FAIL'}`);
  });
});

setTimeout(() => {
  console.log('\nValidation complete!');
  process.exit(0);
}, 1000);
```