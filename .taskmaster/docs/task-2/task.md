# Task 2: Implement Core Application Structure

## Overview
This task establishes the core Express.js application structure, including middleware configuration, error handling, and server initialization. It creates the foundation for all API functionality by setting up the main application entry points and essential utilities.

## Objectives
- Set up the Express.js application with proper middleware stack
- Implement correlation ID tracking for request tracing
- Configure centralized error handling
- Create server entry point with graceful shutdown
- Establish standardized response format utilities

## Technical Approach

### Application Architecture
The application follows a modular architecture with clear separation between:
- **app.js**: Express application configuration and middleware setup
- **server.js**: Server initialization and lifecycle management
- **response.js**: Standardized response format utilities

### Middleware Stack
1. **Correlation ID**: Custom middleware for request tracking
2. **Helmet**: Security headers for protection against common vulnerabilities
3. **CORS**: Cross-Origin Resource Sharing configuration
4. **Body Parser**: JSON request body parsing
5. **Pino Logger**: High-performance request logging with correlation IDs
6. **Error Handler**: Centralized error processing
7. **404 Handler**: Catch-all for undefined routes

### Response Standardization
All API responses follow a consistent format:
```json
{
  "status": "success|error",
  "message": "string",
  "data": "any",
  "timestamp": "ISO 8601 string"
}
```

## Implementation Details

### Step 1: Create Express Application (src/app.js)
```javascript
const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const pino = require('pino-http');
const swaggerJsdoc = require('swagger-jsdoc');
const swaggerUi = require('swagger-ui-express');

// Create Express app
const app = express();

// Add correlation ID middleware
app.use((req, res, next) => {
  req.correlationId = req.headers['x-correlation-id'] || Math.random().toString(36).substring(2, 15);
  res.setHeader('x-correlation-id', req.correlationId);
  next();
});

// Configure middleware
app.use(helmet());
app.use(cors());
app.use(express.json());
app.use(pino({
  genReqId: (req) => req.correlationId,
  redact: ['req.headers.authorization'],
}));

// Configure routes (to be implemented)
app.use('/', require('./routes'));

// Error handling middleware
app.use((err, req, res, next) => {
  req.log.error({ err });
  res.status(err.status || 500).json({
    status: 'error',
    message: err.message || 'Internal Server Error',
    data: null,
    timestamp: new Date().toISOString()
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    status: 'error',
    message: 'Not Found',
    data: null,
    timestamp: new Date().toISOString()
  });
});

module.exports = app;
```

### Step 2: Create Server Entry Point (src/server.js)
```javascript
require('dotenv').config();
const app = require('./app');

const PORT = process.env.PORT || 3000;
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

// Graceful shutdown handling
const shutdown = () => {
  console.log('Shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
  
  // Force close after 10s
  setTimeout(() => {
    console.error('Forcing shutdown after timeout');
    process.exit(1);
  }, 10000);
};

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);

module.exports = server;
```

### Step 3: Create Response Utilities (src/utils/response.js)
```javascript
module.exports = {
  success: (message, data = null) => ({
    status: 'success',
    message,
    data,
    timestamp: new Date().toISOString()
  }),
  error: (message, status = 400, data = null) => {
    const error = new Error(message);
    error.status = status;
    error.data = data;
    return error;
  }
};
```

### Middleware Configuration Details

#### Correlation ID Middleware
- Generates unique ID for each request if not provided
- Passes ID through X-Correlation-ID header
- Enables request tracing across logs

#### Pino Logger Configuration
- Uses correlation ID as request ID
- Redacts sensitive headers (authorization)
- Provides high-performance JSON logging
- Automatically logs request/response details

#### Error Handling Strategy
- Catches all errors thrown in route handlers
- Logs full error details with correlation ID
- Returns sanitized error response to client
- Maintains consistent response format

#### Graceful Shutdown
- Listens for SIGTERM and SIGINT signals
- Stops accepting new connections
- Waits for existing connections to close
- Forces shutdown after 10-second timeout

## Dependencies and Requirements
- Task 1 must be completed (project setup and dependencies)
- All npm packages must be installed
- Environment variables must be configured in .env file
- src/routes directory must exist (even if empty initially)

## Testing Strategy

### Unit Tests
- Test response utility functions for correct format
- Test correlation ID generation and propagation
- Test error transformation in error handler

### Integration Tests
- Test middleware stack initialization
- Test 404 handling for undefined routes
- Test error handling with various error types
- Test graceful shutdown sequence

### Manual Testing
```bash
# Start the server
npm run dev

# Test server is running (will get 404 as routes aren't implemented yet)
curl http://localhost:3000

# Check correlation ID header
curl -I http://localhost:3000

# Test graceful shutdown
# Start server then press Ctrl+C
```

## Success Criteria
- Express app initializes without errors
- Server starts and listens on configured port
- Correlation ID appears in response headers
- 404 responses follow standard format
- Error responses follow standard format
- Graceful shutdown completes within timeout
- All middleware is properly configured
- Logs show request details with correlation IDs

## Related Tasks
- Task 1: Setup Project Structure (prerequisite)
- Task 3: Implement API Endpoints (depends on this task)
- Task 4: Add Health Check Endpoint (depends on this task)
- Task 5: Implement OpenAPI Documentation (depends on this task)