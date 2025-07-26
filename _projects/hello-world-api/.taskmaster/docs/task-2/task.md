# Task 2: Create Express.js Server

## Overview

This task focuses on implementing the core Express.js server that will power the Hello World API. Building upon the initialized Node.js project from Task 1, this task establishes the server infrastructure with proper request logging and error handling foundations.

## Purpose and Objectives

The primary objective is to create a functional Express.js server that:

- Listens on port 3000 for incoming HTTP requests
- Implements request logging middleware for monitoring and debugging
- Provides a foundation for adding API endpoints
- Handles basic server lifecycle (startup and shutdown)

## Technical Approach

### 1. Server Architecture
- Utilize Express.js as the web application framework
- Implement middleware pattern for cross-cutting concerns
- Use console logging for request tracking in development

### 2. Middleware Implementation
- Create custom logging middleware that captures:
  - Timestamp in ISO format
  - HTTP method
  - Request URL
- Ensure middleware processes all requests before routing

### 3. Server Configuration
- Configure server to listen on port 3000
- Support environment variable override for port configuration
- Implement graceful startup with success messaging

## Implementation Details

### Complete Server Implementation

The complete `src/index.js` file should contain:

```javascript
const express = require('express');
const app = express();
const PORT = 3000;

// Middleware for logging requests
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
  next();
});

// Basic route placeholder
app.get('/', (req, res) => {
  res.status(200).send('Server is running');
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

### Key Components Explained

1. **Express Application Setup**
   - Import Express framework
   - Create application instance
   - Define port constant

2. **Request Logging Middleware**
   - Captures all incoming requests
   - Logs timestamp, method, and URL
   - Calls `next()` to continue request processing

3. **Placeholder Route**
   - Provides immediate feedback that server is operational
   - Returns 200 status with simple message

4. **404 Handler**
   - Catches all unmatched routes
   - Returns standardized JSON error response

5. **Server Startup**
   - Binds to specified port
   - Logs startup confirmation

## Dependencies and Requirements

### Prerequisites
- Completed Task 1 (Node.js project initialization)
- Node.js 20+ installed
- Express.js installed via npm

### Runtime Dependencies
- Express.js ^4.18.2

### Development Dependencies
- None required for basic server implementation

## Testing Strategy

### Manual Testing Steps

1. **Server Startup Test**
   ```bash
   npm start
   ```
   Expected output:
   ```
   Server running on http://localhost:3000
   ```

2. **Request Logging Test**
   ```bash
   # In another terminal
   curl http://localhost:3000
   ```
   Expected server log:
   ```
   2024-01-15T10:30:45.123Z - GET /
   ```

3. **404 Handler Test**
   ```bash
   curl http://localhost:3000/nonexistent
   ```
   Expected response:
   ```json
   {"error":"Not found"}
   ```
   Expected status code: 404

4. **Multiple Request Types Test**
   ```bash
   # Test different HTTP methods
   curl -X POST http://localhost:3000
   curl -X PUT http://localhost:3000
   curl -X DELETE http://localhost:3000
   ```
   Verify each request is logged with correct method

### Automated Test Approach

Create a test script `test-server.js`:

```javascript
const http = require('http');

// Test server is running
http.get('http://localhost:3000', (res) => {
  console.log('Status:', res.statusCode);
  console.log('Server test:', res.statusCode === 200 ? 'PASS' : 'FAIL');
});

// Test 404 handler
http.get('http://localhost:3000/404test', (res) => {
  console.log('404 test:', res.statusCode === 404 ? 'PASS' : 'FAIL');
});
```

## Success Criteria

The task is complete when:

1. Server starts successfully on port 3000
2. All HTTP requests are logged with timestamp, method, and URL
3. Server responds to requests (even if with placeholder response)
4. Undefined routes return 404 status with JSON error message
5. Console displays clear startup message
6. Server can be stopped gracefully with Ctrl+C

## Common Issues and Solutions

### Issue 1: Port 3000 already in use
**Error**: `Error: listen EADDRINUSE: address already in use :::3000`
**Solution**: 
- Find process using port: `lsof -i :3000` (Mac/Linux) or `netstat -ano | findstr :3000` (Windows)
- Kill the process or use a different port

### Issue 2: Express not found
**Error**: `Error: Cannot find module 'express'`
**Solution**: 
- Ensure you're in the project directory
- Run `npm install` to install dependencies

### Issue 3: Middleware not logging
**Cause**: Middleware placed after routes
**Solution**: Ensure logging middleware is added before any routes

## Best Practices Implemented

1. **Middleware Order**: Logging middleware placed first to capture all requests
2. **Error Handling**: 404 handler as last middleware to catch unmatched routes
3. **Logging Format**: ISO timestamp for easy parsing and sorting
4. **Status Codes**: Proper HTTP status codes (200 for success, 404 for not found)
5. **Console Feedback**: Clear server startup message for developer experience

## Next Steps

After completing this task:
- Task 3: Implement Hello Endpoint - Add the main API endpoint
- Task 4: Implement Health Check Endpoint - Add monitoring capabilities
- Task 5: Add Error Handling and Documentation - Complete error handling middleware