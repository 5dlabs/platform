# Task 11: Add Error Handling

## Overview
This task implements comprehensive error handling middleware for the Express.js API, including both 500 Internal Server Error handling for unexpected errors and 404 Not Found handling for undefined routes. This ensures the API responds gracefully to error conditions with appropriate HTTP status codes.

## Purpose and Objectives
- Implement error handling middleware for server errors (500)
- Add 404 handler for undefined routes
- Provide consistent error response format
- Log errors for debugging purposes
- Prevent error stack traces from leaking to clients
- Follow Express.js error handling best practices

## Technical Approach

### Error Handling Strategy
1. **Error Middleware**: Four-parameter function to catch errors
2. **404 Handler**: Catch-all for unmatched routes
3. **Middleware Order**: Critical for proper error handling
4. **Response Format**: Consistent JSON error responses
5. **Logging**: Detailed server-side error logging

### Key Technical Decisions
- Use Express's standard error handling pattern
- Return generic error messages to clients (security)
- Log detailed errors server-side only
- Place error handlers after all routes
- Use consistent JSON response format

## Implementation Details

### Error Handling Middleware Implementation
```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});
```

### Enhanced Implementation with Logging
```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  // Log error details for debugging
  console.error(`${new Date().toISOString()} - ERROR:`);
  console.error(`  Message: ${err.message}`);
  console.error(`  Method: ${req.method}`);
  console.error(`  Path: ${req.path}`);
  console.error(`  Stack: ${err.stack}`);
  
  // Send generic error response
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  console.log(`${new Date().toISOString()} - 404 Not Found: ${req.method} ${req.path}`);
  res.status(404).json({ error: 'Not Found' });
});
```

### Complete Integration in src/index.js
```javascript
// ... existing imports and setup ...

// Middleware
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Routes
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling middleware (must be after routes)
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler (must be last)
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// ... server startup code ...
```

### Error Response Formats

**500 Internal Server Error:**
```json
{
  "error": "Internal Server Error"
}
```

**404 Not Found:**
```json
{
  "error": "Not Found"
}
```

## Dependencies and Requirements

### Prerequisites
- Completed Tasks 9 & 10: Routes are implemented
- Express.js error handling knowledge
- Understanding of middleware order

### Technical Requirements
- Express.js 4.x error handling patterns
- Four-parameter error handler function
- Proper middleware ordering

## Testing Strategy

### Manual Testing

1. **Test 404 Handler**
   ```bash
   curl -i http://localhost:3000/nonexistent
   
   # Expected:
   HTTP/1.1 404 Not Found
   {"error":"Not Found"}
   ```

2. **Test Error Handler** (with test route)
   ```javascript
   // Temporary test route
   app.get('/test-error', (req, res, next) => {
     next(new Error('Test error'));
   });
   ```
   
   ```bash
   curl -i http://localhost:3000/test-error
   
   # Expected:
   HTTP/1.1 500 Internal Server Error
   {"error":"Internal Server Error"}
   ```

3. **Test Various HTTP Methods**
   ```bash
   curl -X POST http://localhost:3000/
   curl -X PUT http://localhost:3000/health
   curl -X DELETE http://localhost:3000/api
   # All should return 404
   ```

### Automated Testing
```javascript
// test-errors.js
const http = require('http');

function testEndpoint(path, method = 'GET', expectedStatus) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'localhost',
      port: 3000,
      path: path,
      method: method
    };
    
    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        const success = res.statusCode === expectedStatus;
        console.log(`${method} ${path}: ${success ? '✓' : '✗'} ${res.statusCode}`);
        if (!success) console.log('  Response:', data);
        resolve(success);
      });
    });
    
    req.on('error', reject);
    req.end();
  });
}

// Run tests
async function runTests() {
  const tests = [
    { path: '/', method: 'GET', expected: 200 },
    { path: '/health', method: 'GET', expected: 200 },
    { path: '/nonexistent', method: 'GET', expected: 404 },
    { path: '/', method: 'POST', expected: 404 },
    { path: '/api/v1/users', method: 'GET', expected: 404 },
  ];
  
  for (const test of tests) {
    await testEndpoint(test.path, test.method, test.expected);
  }
}

runTests().catch(console.error);
```

### Success Criteria
- ✅ 404 returned for all undefined routes
- ✅ 500 returned for server errors
- ✅ Error responses are JSON format
- ✅ Error details logged to console
- ✅ Client doesn't see stack traces
- ✅ All errors are handled gracefully
- ✅ Server doesn't crash on errors

## Related Tasks
- **Previous**: Tasks 9 & 10 - Implement endpoints
- **Next**: Task 12 - Create README
- **Related**: Task 8 - Main server file

## Notes and Considerations
- Error middleware must have exactly 4 parameters
- Order matters: error handlers must be last
- Never expose stack traces to clients in production
- Consider adding request ID for error tracking
- In production, use proper logging libraries
- Could add custom error classes for different scenarios
- Rate limiting would prevent error-based DoS attacks
- Consider different error codes for different scenarios