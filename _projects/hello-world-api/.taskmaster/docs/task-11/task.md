# Task 11: Add Error Handling

## Overview
**Title**: Add Error Handling  
**Status**: pending  
**Priority**: medium  
**Dependencies**: Task 9 (Root Endpoint), Task 10 (Health Check)  

## Description
Implement basic error handling middleware to return 500 status codes for server errors and 404 status codes for undefined routes. This task adds robustness to the API by ensuring all errors are handled gracefully with appropriate HTTP status codes and error messages.

## Technical Approach

### 1. Error Handling Middleware
- Implement Express error middleware (4 parameters)
- Catch and handle all unhandled errors
- Return standardized error responses
- Log errors for debugging

### 2. 404 Not Found Handler
- Catch requests to undefined routes
- Return consistent 404 responses
- Must be the last middleware

### 3. Error Response Format
- Consistent JSON error structure
- Appropriate HTTP status codes
- Clear error messages
- No sensitive information exposed

## Implementation Details

### Error Handling Middleware
```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  console.error(`Stack: ${err.stack}`);
  console.error(`Request path: ${req.path}`);
  console.error(`Request method: ${req.method}`);
  res.status(500).json({ error: 'Internal Server Error' });
});
```

### 404 Handler
```javascript
// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});
```

### Complete Integration
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

// Error handling middleware (must have 4 parameters)
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  console.error(`Stack: ${err.stack}`);
  console.error(`Request path: ${req.path}`);
  console.error(`Request method: ${req.method}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes (must be last)
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// Server startup
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

// ... server error handling ...
```

### Error Response Formats

#### 500 Internal Server Error
```json
{
  "error": "Internal Server Error"
}
```

#### 404 Not Found
```json
{
  "error": "Not Found"
}
```

## Subtasks Breakdown

### 1. Implement 500 Error Handling Middleware
- **Status**: pending
- **Dependencies**: None
- **Implementation**: Add 4-parameter error middleware
- **Features**: Error logging and standardized response

### 2. Implement 404 Not Found Handler
- **Status**: pending
- **Dependencies**: Subtask 1
- **Implementation**: Add catch-all middleware
- **Placement**: Must be last in middleware chain

### 3. Add Error Logging Functionality
- **Status**: pending
- **Dependencies**: Subtask 1
- **Enhancement**: Detailed error logging
- **Includes**: Stack trace, request details

### 4. Create Error Testing Routes
- **Status**: pending
- **Dependencies**: Subtasks 1, 2
- **Purpose**: Verify error handling works
- **Note**: Temporary, for testing only

### 5. Document Error Handling Implementation
- **Status**: pending
- **Dependencies**: Subtasks 1, 2, 3
- **Implementation**: Add comprehensive comments
- **Documentation**: Explain middleware purpose

## Dependencies
- Express.js error handling features
- Existing routes must be defined first
- Middleware order is critical

## Testing Strategy

### Test 500 Error Handler

#### Option 1: Test Route (Temporary)
```javascript
// Add temporarily for testing
app.get('/test-error', (req, res, next) => {
  next(new Error('Test server error'));
});
```

#### Test Command
```bash
curl http://localhost:3000/test-error
# Expected: {"error":"Internal Server Error"}
# Status: 500
```

### Test 404 Handler

#### Test Various Undefined Routes
```bash
# Test undefined path
curl http://localhost:3000/undefined
# Expected: {"error":"Not Found"}
# Status: 404

# Test undefined API path
curl http://localhost:3000/api/users
# Expected: {"error":"Not Found"}
# Status: 404

# Test with different methods
curl -X POST http://localhost:3000/
# Expected: {"error":"Not Found"}
# Status: 404
```

### Verify Error Logging

#### Server Console Output for 500 Error
```
Error: Test server error
Stack: Error: Test server error
    at /path/to/index.js:line:col
    ...
Request path: /test-error
Request method: GET
```

#### Server Console Output for 404
```
2024-01-15T10:30:45.123Z - GET /undefined
```

## Common Issues and Solutions

### Issue: Error middleware not catching errors
**Solution**: Ensure it has exactly 4 parameters (err, req, res, next)

### Issue: 404 handler catching valid routes
**Solution**: Place 404 handler after all route definitions

### Issue: Errors not being passed to middleware
**Solution**: Use next(err) in route handlers to pass errors

### Issue: Synchronous errors not caught
**Solution**: Express automatically catches synchronous errors in routes

## Security Considerations

### Error Information Exposure
- Never expose stack traces in production
- Don't reveal internal system details
- Use generic error messages for clients
- Log detailed errors server-side only

### Production Configuration
```javascript
// Production error handler
app.use((err, req, res, next) => {
  // Log full error details server-side
  console.error(err.stack);
  
  // Send generic message to client
  const message = process.env.NODE_ENV === 'production' 
    ? 'Internal Server Error' 
    : err.message;
    
  res.status(500).json({ error: message });
});
```

## Best Practices

### 1. Middleware Order
1. Request logging middleware
2. Body parsing middleware (if needed)
3. Routes
4. Error handling middleware
5. 404 handler (last)

### 2. Error Handling Patterns
```javascript
// Async route with error handling
app.get('/async-route', async (req, res, next) => {
  try {
    const result = await someAsyncOperation();
    res.json(result);
  } catch (err) {
    next(err); // Pass to error middleware
  }
});
```

### 3. Custom Error Classes
```javascript
// For future enhancement
class ApiError extends Error {
  constructor(message, statusCode) {
    super(message);
    this.statusCode = statusCode;
  }
}
```

## Performance Considerations

- Error handling adds minimal overhead
- Logging can impact performance if excessive
- Consider async logging in production
- 404 checks are fast (no route matching)

## Next Steps
After completing this task:
- Create README documentation (Task 12)
- Test all endpoints with error scenarios (Task 13)
- Consider adding input validation middleware
- Implement environment-specific error handling

The API now has comprehensive error handling for both server errors and undefined routes.