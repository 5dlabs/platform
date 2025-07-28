# Task 2: Implement Core Application Structure - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. File Structure ✓
- [ ] File `src/app.js` exists
- [ ] File `src/server.js` exists
- [ ] File `src/utils/response.js` exists
- [ ] File `src/routes/index.js` exists (can be minimal)

### 2. Express Application Setup (src/app.js) ✓
- [ ] Express app is created and exported
- [ ] Correlation ID middleware is implemented
  - [ ] Checks for existing `x-correlation-id` header
  - [ ] Generates new ID if not present
  - [ ] Attaches ID to `req.correlationId`
  - [ ] Sets `x-correlation-id` response header
- [ ] Helmet middleware is configured
- [ ] CORS middleware is configured
- [ ] JSON body parser is configured
- [ ] Pino logger is configured with:
  - [ ] Correlation ID as request ID
  - [ ] Authorization header redaction
- [ ] Routes are mounted at root path
- [ ] Error handling middleware is implemented
- [ ] 404 handler is implemented

### 3. Server Configuration (src/server.js) ✓
- [ ] Environment variables are loaded with dotenv
- [ ] Express app is imported from './app'
- [ ] Server listens on PORT from env or 3000
- [ ] Console log shows server start message
- [ ] Graceful shutdown is implemented:
  - [ ] Handles SIGTERM signal
  - [ ] Handles SIGINT signal
  - [ ] Closes server connections
  - [ ] Has 10-second timeout
  - [ ] Exits with appropriate codes
- [ ] Server instance is exported

### 4. Response Utilities (src/utils/response.js) ✓
- [ ] Module exports object with functions
- [ ] `success` function exists with:
  - [ ] message parameter
  - [ ] data parameter with null default
  - [ ] Returns proper success object
- [ ] `error` function exists with:
  - [ ] message parameter
  - [ ] status parameter with 400 default
  - [ ] data parameter with null default
  - [ ] Returns Error instance with properties

### 5. Response Format Compliance ✓
- [ ] Success responses have required fields:
  - [ ] status: "success"
  - [ ] message: string
  - [ ] data: any
  - [ ] timestamp: ISO string
- [ ] Error responses have required fields:
  - [ ] status: "error"
  - [ ] message: string
  - [ ] data: any
  - [ ] timestamp: ISO string

## Test Cases

### Test Case 1: Server Startup
```bash
# Start the server
node src/server.js
# Expected output: "Server running on port 3000" (or configured PORT)
```

### Test Case 2: Correlation ID Header
```bash
# Make a request and check headers
curl -I http://localhost:3000
# Expected: Response includes "x-correlation-id" header
```

### Test Case 3: 404 Response Format
```bash
# Request undefined route
curl http://localhost:3000/undefined-route
# Expected JSON response:
# {
#   "status": "error",
#   "message": "Not Found",
#   "data": null,
#   "timestamp": "2024-..."
# }
```

### Test Case 4: Graceful Shutdown
```bash
# Start server
node src/server.js
# Press Ctrl+C
# Expected output:
# "Shutting down gracefully..."
# "Server closed"
# Process exits with code 0
```

### Test Case 5: Response Utility Functions
```javascript
// Test in Node REPL
const response = require('./src/utils/response');

// Test success function
console.log(response.success('Test message', { id: 1 }));
// Expected: { status: 'success', message: 'Test message', data: { id: 1 }, timestamp: '...' }

// Test error function
const err = response.error('Test error', 404);
console.log(err.message); // 'Test error'
console.log(err.status);  // 404
```

### Test Case 6: Middleware Order Verification
```javascript
// Add this temporary test to app.js to verify middleware order
app._router.stack.forEach((middleware, index) => {
  console.log(`${index}: ${middleware.name || 'anonymous'}`);
});
// Verify order matches specification
```

## Validation Commands

### Basic Functionality Check
```bash
# 1. Check syntax errors
node -c src/app.js
node -c src/server.js
node -c src/utils/response.js

# 2. Start server and verify it runs
PORT=3001 node src/server.js
# Should see: "Server running on port 3001"

# 3. Test correlation ID
curl -H "x-correlation-id: test-123" -I http://localhost:3000
# Response should include: x-correlation-id: test-123

# 4. Test 404 handling
curl http://localhost:3000/nonexistent | jq
# Should return formatted 404 error
```

### Environment Variable Test
```bash
# Create test .env file
echo "PORT=4000" > .env
node src/server.js
# Should see: "Server running on port 4000"
```

### Error Handling Test
```javascript
// Add temporary test route to verify error handling
// In src/routes/index.js:
router.get('/test-error', (req, res, next) => {
  next(new Error('Test error'));
});

// Then test:
// curl http://localhost:3000/test-error
// Should return error response with proper format
```

## Success Indicators
- ✅ Server starts without errors
- ✅ All middleware is properly configured
- ✅ Correlation IDs are generated and returned
- ✅ 404 responses use standard format
- ✅ Error responses use standard format
- ✅ Graceful shutdown works correctly
- ✅ Response utilities generate correct formats
- ✅ Logs show request details with correlation IDs

## Common Issues and Solutions

### Issue 1: "Cannot find module './routes'"
**Solution:** Ensure src/routes/index.js exists and exports a router

### Issue 2: No correlation ID in response
**Solution:** Verify correlation ID middleware is placed before other middleware

### Issue 3: Server doesn't shut down gracefully
**Solution:** Check that SIGTERM/SIGINT handlers are registered and server.close() is called

### Issue 4: Error handler not catching errors
**Solution:** Ensure error handler has exactly 4 parameters: (err, req, res, next)

### Issue 5: Pino logger not showing correlation ID
**Solution:** Verify genReqId function returns req.correlationId

## Performance Checks
- Server startup time < 2 seconds
- Graceful shutdown completes < 10 seconds
- Memory usage < 50MB on startup
- Response time for 404 < 10ms