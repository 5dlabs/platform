# Acceptance Criteria: Create Express.js Server

## Definition of Done
The Express.js server is considered successfully created when all the following criteria are met:

## Required Outcomes

### 1. Server File Structure ✓
- [ ] File `src/index.js` exists
- [ ] File contains valid JavaScript code
- [ ] No syntax errors present

### 2. Server Implementation ✓
- [ ] Express is properly imported
- [ ] Express app is initialized
- [ ] PORT is set to 3000
- [ ] Request logging middleware is implemented
- [ ] Root route handler exists
- [ ] 404 handler is implemented
- [ ] Server listen configuration is complete

### 3. Middleware Chain ✓
Order must be exactly:
1. Request logging middleware
2. Route handlers
3. 404 catch-all handler

### 4. Logging Functionality ✓
- [ ] Logs show ISO 8601 timestamp
- [ ] Logs show HTTP method (GET, POST, etc.)
- [ ] Logs show request URL path
- [ ] Format: `2024-01-15T10:30:45.123Z - GET /`

### 5. Route Handling ✓
- [ ] GET / returns "Server is running"
- [ ] GET / returns HTTP 200 status
- [ ] Undefined routes return HTTP 404
- [ ] 404 response is JSON: `{"error":"Not found"}`

## Test Cases

### Test Case 1: Server Startup
```bash
npm start
```
**Expected Output:**
```
Server running on http://localhost:3000
```

### Test Case 2: Root Endpoint
```bash
curl -i http://localhost:3000
```
**Expected Response:**
```
HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8

Server is running
```
**Expected Log:**
```
2024-01-15T10:30:45.123Z - GET /
```

### Test Case 3: 404 Handler
```bash
curl -i http://localhost:3000/api/test
```
**Expected Response:**
```
HTTP/1.1 404 Not Found
Content-Type: application/json; charset=utf-8

{"error":"Not found"}
```

### Test Case 4: Request Logging
```bash
# Multiple requests
curl http://localhost:3000
curl -X POST http://localhost:3000
curl http://localhost:3000/users
```
**Expected Logs:**
```
2024-01-15T10:30:45.123Z - GET /
2024-01-15T10:30:46.456Z - POST /
2024-01-15T10:30:47.789Z - GET /users
```

### Test Case 5: Server Resilience
```bash
# Server should continue running after 404s
curl http://localhost:3000/invalid1
curl http://localhost:3000/invalid2
curl http://localhost:3000  # Should still work
```

## Code Review Checklist

### Structure
- [ ] Imports at the top of file
- [ ] Constants defined after imports
- [ ] Middleware before routes
- [ ] 404 handler is last middleware
- [ ] Server listen at bottom

### Best Practices
- [ ] No hardcoded values (except PORT for now)
- [ ] Proper error handling
- [ ] Clean, readable code
- [ ] Appropriate comments

### Express Patterns
- [ ] Uses `app.use()` for middleware
- [ ] Uses `app.get()` for GET routes
- [ ] Calls `next()` in non-terminal middleware
- [ ] Returns responses in route handlers

## Common Issues & Solutions

### Issue 1: Middleware Not Logging
**Symptom**: Requests work but no logs appear
**Cause**: Missing `next()` call
**Solution**: Ensure logging middleware calls `next()`

### Issue 2: 404 Handler Not Working
**Symptom**: Express default 404 page shows
**Cause**: 404 handler placed before routes
**Solution**: Move 404 handler after all routes

### Issue 3: Server Crashes on Request
**Symptom**: Server stops after first request
**Cause**: Unhandled error in middleware
**Solution**: Add try-catch blocks, ensure proper response

### Issue 4: Routes Return Cannot GET
**Symptom**: All routes return "Cannot GET /path"
**Cause**: Routes defined after 404 handler
**Solution**: Define routes before 404 handler

## Performance Criteria
- [ ] Server starts in < 2 seconds
- [ ] Requests handled in < 50ms
- [ ] No memory leaks on repeated requests
- [ ] Graceful handling of malformed requests

## Security Considerations
- [ ] No sensitive data in logs
- [ ] No directory traversal vulnerabilities
- [ ] Proper HTTP status codes used
- [ ] JSON responses properly formatted

## Sign-off Requirements
- [ ] All test cases pass
- [ ] Code follows Express.js conventions
- [ ] No console errors or warnings
- [ ] Server stable under basic load