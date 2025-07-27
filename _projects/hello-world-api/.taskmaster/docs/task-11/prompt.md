# Autonomous Agent Prompt: Add Error Handling

## Context
You are implementing error handling for a Hello World API. The Express.js server currently has two endpoints (root and health check) but lacks proper error handling middleware. You need to add middleware to handle server errors (500) and undefined routes (404).

## Objective
Add error handling middleware to the Express application that:
1. Catches and handles server errors with 500 status codes
2. Handles undefined routes with 404 status codes
3. Returns consistent JSON error responses
4. Logs error details for debugging

## Task Requirements

### 1. Add Error Handling Middleware
Add error handling middleware to `src/index.js`:
- Must have 4 parameters: (err, req, res, next)
- Log error details to console
- Return 500 status with JSON error response
- Place after all route definitions

### 2. Add 404 Handler
Add a catch-all middleware for undefined routes:
- Catches any unmatched routes
- Returns 404 status with JSON error response
- Must be the very last middleware

### 3. Middleware Placement
Ensure correct middleware order:
1. Request logging (existing)
2. Routes (existing)
3. Error handling middleware (new)
4. 404 handler (new)

## Complete Implementation

Add these middlewares to src/index.js after all routes:

```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  console.error(`Stack: ${err.stack}`);
  console.error(`Request path: ${req.path}`);
  console.error(`Request method: ${req.method}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});
```

## Integration Example

Complete src/index.js structure with error handling:

```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Root endpoint
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  console.error(`Stack: ${err.stack}`);
  console.error(`Request path: ${req.path}`);
  console.error(`Request method: ${req.method}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// Server startup code...
```

## Step-by-Step Execution

1. **Locate insertion point**:
   - Find the end of route definitions
   - Before server.listen() call
   - After all app.get() routes

2. **Add error handling middleware**:
   - Copy the 4-parameter error handler
   - Ensure all 4 parameters are present
   - Include detailed error logging

3. **Add 404 handler**:
   - Place after error handling middleware
   - This must be the last app.use()
   - Simple 2-parameter handler

4. **Save and test**:
   - Save the file
   - Restart server
   - Test both error handlers

## Validation Criteria

### Success Indicators
- [ ] Error middleware has exactly 4 parameters
- [ ] 404 handler is the last middleware
- [ ] Both return JSON responses
- [ ] Error details logged to console
- [ ] 500 status for errors
- [ ] 404 status for undefined routes
- [ ] Existing endpoints still work

### Testing Commands

1. **Test 404 Handler**:
   ```bash
   curl http://localhost:3000/undefined
   # Expected: {"error":"Not Found"}
   # Status: 404
   ```

2. **Test 404 with Different Methods**:
   ```bash
   curl -X POST http://localhost:3000/
   curl -X PUT http://localhost:3000/health
   # Both should return 404
   ```

3. **Verify Existing Routes Work**:
   ```bash
   curl http://localhost:3000/
   curl http://localhost:3000/health
   # Should work normally
   ```

## Testing Error Handler

To test the 500 error handler, you can temporarily add a test route:

```javascript
// Temporary test route (add before error middleware)
app.get('/test-error', (req, res, next) => {
  next(new Error('Test server error'));
});
```

Then test:
```bash
curl http://localhost:3000/test-error
# Expected: {"error":"Internal Server Error"}
# Status: 500
# Check server console for error logs
```

## Expected Behavior

### For 404 Errors
**Request**: GET /undefined
**Response**: 
```json
{"error":"Not Found"}
```
**Status**: 404

### For 500 Errors
**Console Output**:
```
Error: Test server error
Stack: Error: Test server error
    at /path/to/index.js:line:col
    ...
Request path: /test-error
Request method: GET
```
**Response**:
```json
{"error":"Internal Server Error"}
```
**Status**: 500

## Common Mistakes to Avoid

1. **Wrong parameter count**: Error middleware needs exactly 4 parameters
2. **Wrong order**: 404 handler must be last
3. **Missing parameters**: Even if unused, include all 4 parameters
4. **Exposing stack traces**: Don't send stack traces to client
5. **Forgetting next()**: Not needed in these handlers as they send responses

## Important Notes

- The error middleware signature `(err, req, res, next)` is how Express identifies it
- The 404 handler has no error parameter - just `(req, res)`
- Order matters: routes → error handler → 404 handler
- In production, avoid exposing detailed error information
- The 404 handler catches ALL unmatched routes

## Security Considerations

- Log detailed errors server-side only
- Return generic error messages to clients
- Never expose stack traces in responses
- Consider environment-based error messages

## Tools Required
- File system access to modify src/index.js
- Text editing capability for JavaScript code
- Command execution for testing

Proceed with implementing the error handling middleware, ensuring proper placement and testing that both error scenarios return appropriate JSON responses.