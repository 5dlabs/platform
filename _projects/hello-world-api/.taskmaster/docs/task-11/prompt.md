# Autonomous Agent Prompt for Task 11: Add Error Handling

## Task Context
You need to add error handling middleware to the Express.js server to handle both server errors (500) and undefined routes (404). This will make the API more robust and production-ready.

## Your Mission
Implement error handling middleware that catches server errors and a 404 handler for undefined routes, ensuring the API responds gracefully to all error conditions.

## Step-by-Step Instructions

### 1. Navigate to Project
```bash
cd hello-world-api
```

### 2. Add Error Handling Middleware
Edit `src/index.js` to add error handling AFTER all route definitions but BEFORE app.listen():

```javascript
// Error handling middleware (must have 4 parameters)
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  console.error(`Stack: ${err.stack}`);
  console.error(`Time: ${new Date().toISOString()}`);
  console.error(`Request: ${req.method} ${req.path}`);
  
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes (must be last)
app.use((req, res) => {
  console.log(`404 Not Found: ${req.method} ${req.path}`);
  res.status(404).json({ error: 'Not Found' });
});
```

### 3. Complete Server Structure
Your complete `src/index.js` should now look like:

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
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

module.exports = app;
```

### 4. Enhanced Implementation (Recommended)
For production-ready error handling:

```javascript
// Environment-aware error handling middleware
app.use((err, req, res, next) => {
  const timestamp = new Date().toISOString();
  
  // Always log errors
  console.error(`[${timestamp}] Error: ${err.message}`);
  console.error(`[${timestamp}] Stack: ${err.stack}`);
  console.error(`[${timestamp}] Request: ${req.method} ${req.originalUrl}`);
  console.error(`[${timestamp}] IP: ${req.ip}`);
  
  // Determine status code
  const statusCode = err.statusCode || 500;
  
  // Build error response
  const errorResponse = {
    error: statusCode === 500 ? 'Internal Server Error' : err.message,
    timestamp: timestamp
  };
  
  // In development, include more details
  if (process.env.NODE_ENV === 'development') {
    errorResponse.details = {
      message: err.message,
      stack: err.stack,
      path: req.path,
      method: req.method
    };
  }
  
  res.status(statusCode).json(errorResponse);
});

// Enhanced 404 handler
app.use((req, res) => {
  const timestamp = new Date().toISOString();
  console.log(`[${timestamp}] 404 Not Found: ${req.method} ${req.originalUrl}`);
  
  res.status(404).json({
    error: 'Not Found',
    message: `Cannot ${req.method} ${req.originalUrl}`,
    timestamp: timestamp
  });
});
```

## Validation Steps

### 1. Test 404 Handler
```bash
# Start server
npm start

# Test undefined route
curl http://localhost:3000/undefined-route
# Expected: {"error":"Not Found"}

curl -I http://localhost:3000/undefined-route
# Expected: HTTP/1.1 404 Not Found
```

### 2. Test Error Handler
To test the 500 error handler, temporarily add a test route:

```javascript
// Temporary test route (add before error handlers)
app.get('/test-error', (req, res, next) => {
  next(new Error('Test error'));
});
```

Then test:
```bash
curl http://localhost:3000/test-error
# Expected: {"error":"Internal Server Error"}

# Check server logs for error details
```

### 3. Test Multiple 404s
```bash
# Test various undefined paths
curl http://localhost:3000/api/users
curl http://localhost:3000/admin
curl -X POST http://localhost:3000/
# All should return 404 with "Not Found" error
```

### 4. Verify Existing Routes Still Work
```bash
# Test that error handlers don't affect normal routes
curl http://localhost:3000/
# Expected: {"message":"Hello, World!"}

curl http://localhost:3000/health
# Expected: {"status":"healthy","timestamp":"..."}
```

## Expected Result
- Error handling middleware catches and logs server errors
- 404 handler catches all undefined routes
- Existing routes continue to work normally
- All errors return consistent JSON format
- Server remains stable even when errors occur

## Important Notes
- Error middleware MUST have 4 parameters (err, req, res, next)
- Error middleware must come AFTER all routes
- 404 handler must be the LAST middleware
- Don't expose sensitive error details in production
- Always log errors for debugging

## Common Issues to Avoid
1. Wrong parameter count in error middleware (must be 4)
2. Placing error handlers before routes (they won't work)
3. Forgetting to call next(err) in routes to trigger error handler
4. Exposing stack traces in production
5. Not logging enough information for debugging
6. 404 handler placed before routes (would catch everything)