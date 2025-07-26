# Acceptance Criteria for Task 8: Create Main Server File

## Required Outcomes

### 1. Server File Creation
- [ ] File exists at `src/index.js`
- [ ] File imports Express.js correctly
- [ ] Express app instance is created
- [ ] Server exports app module (for testing)

### 2. Port Configuration
- [ ] PORT constant is defined
- [ ] Supports environment variable: `process.env.PORT`
- [ ] Falls back to port 3000 if env var not set
- [ ] Port value is used in app.listen()

### 3. Request Logging Middleware
- [ ] Middleware function is implemented
- [ ] Logs timestamp in ISO format
- [ ] Logs HTTP method (GET, POST, etc.)
- [ ] Logs request URL
- [ ] Calls next() to continue pipeline
- [ ] Middleware is registered with app.use()

### 4. Server Startup
- [ ] app.listen() is called with PORT
- [ ] Startup message is logged to console
- [ ] Message includes port number
- [ ] Server actually listens on specified port

### 5. Error Handling
- [ ] Basic error handling is implemented
- [ ] Handles port already in use error
- [ ] Logs meaningful error messages
- [ ] Exits gracefully on critical errors

## Test Cases

### Test 1: Basic Server Startup
```bash
cd hello-world-api
npm start
# Expected output includes:
# "Server running on port 3000"
```

### Test 2: Custom Port Configuration
```bash
PORT=5000 npm start
# Expected output includes:
# "Server running on port 5000"
```

### Test 3: Request Logging Verification
```bash
# Terminal 1:
npm start

# Terminal 2:
curl http://localhost:3000
# Expected in Terminal 1:
# [ISO timestamp] - GET /
```

### Test 4: Port Conflict Handling
```bash
# Terminal 1:
npm start

# Terminal 2:
npm start
# Expected in Terminal 2:
# Error message about port being in use
```

### Test 5: Server Response
```bash
# Start server
npm start

# Test server is responding
curl -I http://localhost:3000
# Expected: HTTP response headers (even if 404)
```

### Test 6: Graceful Shutdown
```bash
# Start server
npm start

# Press Ctrl+C
# Expected: Server shuts down without errors
```

## Code Quality Checks

### Structure Requirements
- [ ] Proper module imports at top
- [ ] Clear code organization
- [ ] Meaningful variable names
- [ ] Appropriate comments

### Middleware Requirements
- [ ] Middleware defined before routes
- [ ] Proper middleware signature (req, res, next)
- [ ] No blocking operations in middleware
- [ ] next() called appropriately

### Best Practices
- [ ] No hardcoded values (use constants/config)
- [ ] Consistent code formatting
- [ ] Error messages are informative
- [ ] No console errors on startup

## Definition of Done
- Server starts successfully on default port 3000
- Environment variable PORT is respected
- All HTTP requests are logged with timestamp
- Basic error handling is in place
- Code follows Express.js conventions
- Ready for endpoint implementation (Tasks 9-10)

## Common Issues to Avoid
1. Forgetting to call next() in middleware
2. Hardcoding port instead of using environment variable
3. Not handling EADDRINUSE error
4. Middleware defined after routes
5. Missing error handling for server startup
6. Not exporting app for potential testing