# Acceptance Criteria: Create Express.js Server

## Required Deliverables

### 1. Server File Implementation ✓
- [ ] File `src/index.js` exists
- [ ] Contains valid JavaScript code
- [ ] No syntax errors

### 2. Express Configuration ✓
- [ ] Express is properly imported: `const express = require('express')`
- [ ] App instance created: `const app = express()`
- [ ] PORT constant defined: `const PORT = 3000`

### 3. Middleware Implementation ✓
- [ ] Request logging middleware is defined
- [ ] Middleware uses `app.use()`
- [ ] Logs include:
  - [ ] ISO timestamp
  - [ ] HTTP method
  - [ ] Request URL
- [ ] Calls `next()` to continue chain

### 4. Route Handlers ✓
- [ ] Root route (`/`) is defined
- [ ] Returns 200 status code
- [ ] Returns "Server is running" text
- [ ] 404 handler is implemented
- [ ] Returns 404 status code
- [ ] Returns JSON error response

### 5. Server Listener ✓
- [ ] `app.listen()` is called with PORT
- [ ] Callback logs startup message
- [ ] Message includes the URL

## Test Cases

### Test Case 1: Server Startup
**Steps:**
1. Run `npm start`
2. Observe console output

**Expected:**
- Message: "Server running on http://localhost:3000"
- No error messages
- Server continues running

### Test Case 2: Request Logging - GET
**Steps:**
1. Start server
2. Execute: `curl http://localhost:3000`
3. Check server console

**Expected:**
- Log entry appears with format: `2024-01-26T10:30:45.123Z - GET /`

### Test Case 3: Request Logging - POST
**Steps:**
1. Start server
2. Execute: `curl -X POST http://localhost:3000/test`
3. Check server console

**Expected:**
- Log entry: `[timestamp] - POST /test`

### Test Case 4: Root Endpoint Response
**Steps:**
1. Start server
2. Execute: `curl -i http://localhost:3000`

**Expected:**
```
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8

Server is running
```

### Test Case 5: 404 Response
**Steps:**
1. Start server
2. Execute: `curl -i http://localhost:3000/nonexistent`

**Expected:**
```
HTTP/1.1 404 Not Found
Content-Type: application/json

{"error":"Not found"}
```

### Test Case 6: Multiple Concurrent Requests
**Steps:**
1. Start server
2. Send 5 requests simultaneously
3. Check logs

**Expected:**
- All 5 requests are logged
- Server remains responsive
- No errors

## Performance Requirements

### Response Time
- [ ] Root endpoint responds in < 50ms
- [ ] 404 responses in < 50ms
- [ ] Logging adds < 5ms overhead

### Stability
- [ ] Server handles 100 sequential requests without crashing
- [ ] Memory usage remains stable
- [ ] No memory leaks detected

## Code Quality Checks

### Structure
- [ ] Middleware defined before routes
- [ ] 404 handler is last middleware
- [ ] Consistent indentation

### Best Practices
- [ ] No global variables (except constants)
- [ ] Proper error handling
- [ ] Clear variable names

## Definition of Done
- [ ] All deliverables implemented
- [ ] All test cases pass
- [ ] No console errors or warnings
- [ ] Code follows Express.js conventions
- [ ] Server is production-ready (for development)

## Edge Cases to Verify

### 1. Invalid HTTP Methods
**Test:** `curl -X INVALID http://localhost:3000`
**Expected:** Request is logged, appropriate error returned

### 2. Large URL Paths
**Test:** Request with 1000+ character path
**Expected:** Logged correctly, 404 returned

### 3. Special Characters in URL
**Test:** `curl http://localhost:3000/test%20space?param=value&special=!@#`
**Expected:** Properly logged and handled

### 4. Rapid Server Restarts
**Test:** Start, stop, and restart server quickly
**Expected:** No port binding errors, clean shutdown

## Security Validation
- [ ] No sensitive information in logs
- [ ] 404 errors don't reveal internal paths
- [ ] No directory traversal vulnerabilities
- [ ] Headers don't expose server details