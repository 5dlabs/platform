# Autonomous Task Prompt: Request Logging Enhancement

You need to verify and potentially enhance the request logging middleware in the Express.js server. Basic logging was implemented in Task 2, but this task ensures it meets all requirements.

## Prerequisites
- Task 2 completed (server with basic logging)
- Server file exists at src/index.js
- Basic logging middleware already present

## Current State
The server should already have logging middleware like:
```javascript
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});
```

## Task Requirements

### 1. Verify Existing Logging
Check that the current implementation:
- Exists in src/index.js
- Is placed before all route handlers
- Logs timestamp, method, and URL
- Uses ISO 8601 timestamp format
- Calls next() to continue processing

### 2. Test Current Logging
Run these tests to verify logging works:
```bash
# Start server
npm start

# In another terminal, run:
curl http://localhost:3000/
curl http://localhost:3000/health
curl http://localhost:3000/invalid
curl -X POST http://localhost:3000/
```

Expected console output:
```
2024-01-15T14:32:17.123Z - GET /
2024-01-15T14:32:18.456Z - GET /health
2024-01-15T14:32:19.789Z - GET /invalid
2024-01-15T14:32:20.012Z - POST /
```

### 3. Enhancement Decision
If the basic logging is working correctly, no changes are needed. 

If you want to enhance it, consider this improved version:
```javascript
// Enhanced logging middleware
app.use((req, res, next) => {
  const start = Date.now();
  const timestamp = new Date().toISOString();
  
  // Log incoming request
  console.log(`[${timestamp}] --> ${req.method} ${req.url}`);
  
  // Log response when finished
  res.on('finish', () => {
    const duration = Date.now() - start;
    const responseTime = new Date().toISOString();
    console.log(`[${responseTime}] <-- ${req.method} ${req.url} ${res.statusCode} (${duration}ms)`);
  });
  
  next();
});
```

## Implementation Steps

### Option 1: Verify Only (Recommended)
1. Open src/index.js
2. Locate the logging middleware
3. Verify it meets all requirements
4. Test with various requests
5. Document findings

### Option 2: Enhance Logging
1. Open src/index.js
2. Replace existing logging middleware
3. Add response logging
4. Add response time tracking
5. Test thoroughly

## Verification Checklist

### Basic Requirements
- [ ] Logging middleware exists
- [ ] Placed as first middleware
- [ ] Logs all requests
- [ ] Includes timestamp (ISO 8601)
- [ ] Includes HTTP method
- [ ] Includes URL path
- [ ] Calls next()

### Test Cases
- [ ] GET / is logged
- [ ] GET /health is logged
- [ ] GET /invalid is logged
- [ ] POST requests are logged
- [ ] PUT requests are logged
- [ ] DELETE requests are logged

### Console Output Format
- [ ] Timestamp is readable
- [ ] Format is consistent
- [ ] No errors in console
- [ ] Logs appear immediately

## Common Issues

### Issue 1: No Logs Appearing
**Cause**: Middleware not called or missing next()
**Fix**: Ensure middleware is first and calls next()

### Issue 2: Logs After Errors
**Cause**: Logging middleware placed after routes
**Fix**: Move logging to top of middleware stack

### Issue 3: Duplicate Logs
**Cause**: Multiple logging middlewares
**Fix**: Remove duplicate implementations

### Issue 4: Performance Impact
**Cause**: Synchronous console.log in high traffic
**Fix**: Consider async logging or buffering

## Testing Script
Create a test script to verify logging:
```bash
#!/bin/bash
echo "Testing request logging..."

# Test different methods and paths
curl -s http://localhost:3000/ > /dev/null
echo "✓ Tested GET /"

curl -s http://localhost:3000/health > /dev/null
echo "✓ Tested GET /health"

curl -s -X POST http://localhost:3000/ > /dev/null
echo "✓ Tested POST /"

curl -s http://localhost:3000/not-found > /dev/null
echo "✓ Tested 404 route"

echo "Check server console for logged requests"
```

## Success Criteria
- All requests are logged
- Format is consistent
- No performance degradation
- Logs help with debugging
- No sensitive data logged

## Notes
- Basic console.log is sufficient for development
- Production systems need proper logging libraries
- Consider structured logging (JSON) for parsing
- Never log sensitive information (passwords, tokens)
- Response logging is optional but helpful

Complete verification and any necessary enhancements.