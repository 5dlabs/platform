# Task 5: Add Error Handling and Documentation

## Overview

This final task completes the Hello World API by implementing comprehensive error handling middleware and creating user-facing documentation. This ensures the API is production-ready with proper error management and clear usage instructions.

## Purpose and Objectives

The objectives of this task are to:

- Implement global error handling middleware for unexpected errors
- Ensure 404 handling is properly positioned
- Create comprehensive README documentation
- Provide usage examples and setup instructions
- Complete the API with professional error handling

## Technical Approach

### 1. Error Handling Strategy
- Implement Express error middleware (4-parameter function)
- Catch and handle all unhandled errors
- Log errors for debugging
- Return sanitized error responses to clients

### 2. Middleware Order
- Error handling must be second-to-last middleware
- 404 handler must be the very last middleware
- Proper ordering ensures all errors are caught

### 3. Documentation Approach
- Create clear, concise README
- Include installation and usage instructions
- Provide example requests and responses
- Follow markdown best practices

## Implementation Details

### Error Handling Middleware

Add error handling middleware to `src/index.js` after all routes but before the 404 handler:

```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});
```

### Complete Server Implementation

The final `src/index.js` with all components:

```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Hello endpoint
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
  console.error(err.stack);
  res.status(500).json({ error: 'Something went wrong!' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Server setup
app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
```

### README Documentation

Create `README.md` in the project root:

```markdown
# Hello World API

A simple Express.js API that serves a Hello World message and health check endpoint.

## Installation

```bash
npm install
```

## Usage

Start the server:

```bash
npm start
```

The server will run on http://localhost:3000

## Endpoints

### GET / - Hello World
Returns a greeting message.

**Response:**
```json
{
  "message": "Hello, World!"
}
```

### GET /health - Health Check
Returns the service health status and current timestamp.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T14:32:17.845Z"
}
```

## Example Requests

### Using curl

```bash
# Hello endpoint
curl http://localhost:3000/

# Health check
curl http://localhost:3000/health
```

### Using HTTPie

```bash
# Hello endpoint
http GET localhost:3000

# Health check
http GET localhost:3000/health
```

## Error Handling

The API includes comprehensive error handling:

- **404 Not Found**: Returned for undefined routes
- **500 Internal Server Error**: Returned for unexpected server errors

## Development

### Project Structure
```
hello-world-api/
├── src/
│   └── index.js      # Main server file
├── package.json      # Dependencies
├── README.md         # This file
└── .gitignore       # Git ignore rules
```

### Requirements
- Node.js 20 or higher
- npm (comes with Node.js)

### Running Tests
```bash
# Start the server
npm start

# In another terminal, test endpoints
curl http://localhost:3000/
curl http://localhost:3000/health
```

## License

ISC
```

## Dependencies and Requirements

### Prerequisites
- All previous tasks completed (1-4)
- Express server with both endpoints implemented

### Documentation Requirements
- Clear installation instructions
- Endpoint documentation with examples
- Error response documentation
- Project structure overview

## Testing Strategy

### Error Handling Tests

1. **Test 500 Error Handler**
   
   Create a temporary test route that throws an error:
   ```javascript
   // Temporary test route
   app.get('/test-error', (req, res) => {
     throw new Error('Test error');
   });
   ```
   
   Test it:
   ```bash
   curl http://localhost:3000/test-error
   ```
   
   Expected: `{"error":"Something went wrong!"}`

2. **Test 404 Handler**
   ```bash
   curl http://localhost:3000/nonexistent
   ```
   Expected: `{"error":"Not found"}`

3. **Verify Error Logging**
   - Check server console for error stack traces
   - Ensure errors are logged but not sent to client

### Documentation Validation

1. **Markdown Rendering**
   - View README.md in a markdown viewer
   - Verify formatting is correct
   - Check code blocks render properly

2. **Example Testing**
   - Copy and run each example command
   - Verify outputs match documentation

3. **Completeness Check**
   - Installation steps work for new user
   - All endpoints are documented
   - Error responses are explained

## Success Criteria

The task is complete when:

1. Error handling middleware is implemented correctly
2. Error middleware is placed before 404 handler
3. 500 errors return generic message (not stack trace)
4. README.md exists with all required sections
5. All example commands in README work correctly
6. Project structure is accurately documented
7. Both endpoints are fully documented with examples

## Common Issues and Solutions

### Issue 1: Error Handler Not Working
**Problem**: Errors still crash the server
**Solution**: Ensure error handler has exactly 4 parameters: `(err, req, res, next)`

### Issue 2: Wrong Middleware Order
**Problem**: 404s handled before errors
**Solution**: Place error handler before 404 handler

### Issue 3: Stack Trace Exposed
**Problem**: Error details sent to client
**Solution**: Only send generic message, log details to console

### Issue 4: README Examples Fail
**Problem**: Commands in README don't work
**Solution**: Test each command and update README accordingly

## Best Practices Implemented

### Error Handling
1. **Security**: Never expose internal error details to clients
2. **Logging**: Always log full error for debugging
3. **Consistency**: All errors return JSON format
4. **Status Codes**: Appropriate HTTP status for each error type

### Documentation
1. **Structure**: Logical flow from installation to usage
2. **Examples**: Concrete, runnable examples
3. **Formatting**: Proper markdown with code highlighting
4. **Completeness**: All features documented

## Middleware Order Importance

The final middleware order is critical:

1. **Request Logger**: First, to log all requests
2. **Routes**: Handle specific endpoints
3. **Error Handler**: Catch errors from routes
4. **404 Handler**: Last resort for unmatched paths

```javascript
// 1. Logger (first)
app.use(logger);

// 2. Routes
app.get('/', helloHandler);
app.get('/health', healthHandler);

// 3. Error handler (second to last)
app.use(errorHandler);

// 4. 404 handler (last)
app.use(notFoundHandler);
```

## Testing the Complete API

### Comprehensive Test Script

Create `test-complete-api.js`:

```javascript
const http = require('http');

const tests = [
  { path: '/', expected: 'message' },
  { path: '/health', expected: 'status' },
  { path: '/invalid', expected: 'error', statusCode: 404 }
];

tests.forEach(test => {
  http.get(`http://localhost:3000${test.path}`, (res) => {
    let data = '';
    res.on('data', chunk => data += chunk);
    res.on('end', () => {
      const response = JSON.parse(data);
      const passed = response.hasOwnProperty(test.expected) && 
                    (!test.statusCode || res.statusCode === test.statusCode);
      console.log(`${test.path}: ${passed ? '✓ PASS' : '✗ FAIL'}`);
    });
  });
});
```

## Production Considerations

While this is a simple API, the patterns implemented prepare it for production:

1. **Error Handling**: Prevents crashes and information leakage
2. **Health Checks**: Enables monitoring and orchestration
3. **Logging**: Provides operational visibility
4. **Documentation**: Reduces onboarding time
5. **Structure**: Follows Express best practices

## Next Steps

After completing this task, consider:

1. Adding request validation middleware
2. Implementing rate limiting
3. Adding CORS support
4. Creating automated tests
5. Setting up CI/CD pipeline
6. Containerizing with Docker
7. Adding API versioning
8. Implementing authentication