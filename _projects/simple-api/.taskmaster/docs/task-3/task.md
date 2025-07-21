# Task 3: Implement Express Application and Middleware

## Overview

This task focuses on setting up the Express application with necessary middleware, error handling, and validation utilities. This creates the foundational web server infrastructure that will handle HTTP requests and responses for the Todo REST API.

## Context

Building upon the project setup from Task 1, this task implements the core Express application structure as defined in the [architecture document](../architecture.md). The application will follow Express best practices with proper middleware ordering, centralized error handling, and request validation.

## Implementation Guide

### 1. Create Express Application (src/app.js)

Create the main Express application file that configures all middleware:

```javascript
const express = require('express');
const { validationResult } = require('express-validator');

// Initialize Express app
const app = express();

// Body parsing middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Request logging in development
if (process.env.NODE_ENV === 'development') {
  app.use((req, res, next) => {
    console.log(`${req.method} ${req.path}`);
    next();
  });
}

// Routes will be added here in Task 5

// 404 handler
app.use((req, res) => {
  res.status(404).json({ 
    error: 'Not Found',
    message: `Cannot ${req.method} ${req.path}`
  });
});

// Global error handling middleware
app.use((err, req, res, next) => {
  console.error(err.stack);
  
  // Handle validation errors
  if (err.name === 'ValidationError') {
    return res.status(400).json({
      error: 'Validation Error',
      details: err.details
    });
  }
  
  // Handle database errors
  if (err.code === 'SQLITE_ERROR') {
    return res.status(500).json({
      error: 'Database Error',
      message: process.env.NODE_ENV === 'production' 
        ? 'A database error occurred' 
        : err.message
    });
  }
  
  // Default error response
  res.status(err.status || 500).json({
    error: err.name || 'Internal Server Error',
    message: process.env.NODE_ENV === 'production' 
      ? 'An error occurred processing your request' 
      : err.message
  });
});

module.exports = app;
```

### 2. Create Validation Middleware (src/middleware/validation.js)

Implement reusable validation rules for todo operations:

```javascript
const { body, param, query, validationResult } = require('express-validator');

// Validation middleware to check for errors
const handleValidationErrors = (req, res, next) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    return res.status(400).json({ 
      error: 'Validation Error', 
      details: errors.array().map(err => ({
        field: err.path,
        message: err.msg,
        value: err.value
      }))
    });
  }
  next();
};

// Validation rules for todo operations
const todoValidation = {
  create: [
    body('title')
      .notEmpty().withMessage('Title is required')
      .isString().withMessage('Title must be a string')
      .trim()
      .isLength({ min: 1, max: 200 }).withMessage('Title must be between 1 and 200 characters'),
    body('description')
      .optional()
      .isString().withMessage('Description must be a string')
      .trim()
      .isLength({ max: 1000 }).withMessage('Description cannot exceed 1000 characters'),
    handleValidationErrors
  ],
  
  update: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    body('title')
      .optional()
      .isString().withMessage('Title must be a string')
      .trim()
      .notEmpty().withMessage('Title cannot be empty')
      .isLength({ max: 200 }).withMessage('Title cannot exceed 200 characters'),
    body('description')
      .optional()
      .isString().withMessage('Description must be a string')
      .trim()
      .isLength({ max: 1000 }).withMessage('Description cannot exceed 1000 characters'),
    body('completed')
      .optional()
      .isBoolean().withMessage('Completed must be a boolean'),
    handleValidationErrors
  ],
  
  getOne: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    handleValidationErrors
  ],
  
  delete: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    handleValidationErrors
  ],
  
  list: [
    query('completed')
      .optional()
      .isIn(['true', 'false']).withMessage('Completed must be true or false'),
    query('limit')
      .optional()
      .isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1 and 100'),
    query('offset')
      .optional()
      .isInt({ min: 0 }).withMessage('Offset must be a non-negative integer'),
    handleValidationErrors
  ]
};

module.exports = { todoValidation, handleValidationErrors };
```

### 3. Create Additional Middleware (src/middleware/common.js)

Create common middleware utilities:

```javascript
// CORS middleware (if needed in future)
const cors = (req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization');
  
  if (req.method === 'OPTIONS') {
    return res.sendStatus(200);
  }
  
  next();
};

// Request ID middleware for tracking
const requestId = (req, res, next) => {
  req.id = Date.now().toString(36) + Math.random().toString(36).substr(2);
  res.setHeader('X-Request-ID', req.id);
  next();
};

// Response time middleware
const responseTime = (req, res, next) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(`${req.method} ${req.path} - ${res.statusCode} - ${duration}ms`);
  });
  
  next();
};

module.exports = {
  cors,
  requestId,
  responseTime
};
```

### 4. Create Server Entry Point (server.js)

Create the main server file in the root directory:

```javascript
require('dotenv').config();
const app = require('./src/app');

const PORT = process.env.PORT || 3000;
const NODE_ENV = process.env.NODE_ENV || 'development';

// Start server
const server = app.listen(PORT, () => {
  console.log(`Server running in ${NODE_ENV} mode on port ${PORT}`);
  console.log(`API documentation available at http://localhost:${PORT}/api-docs`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM signal received: closing HTTP server');
  server.close(() => {
    console.log('HTTP server closed');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  console.log('SIGINT signal received: closing HTTP server');
  server.close(() => {
    console.log('HTTP server closed');
    process.exit(0);
  });
});

module.exports = server;
```

### 5. Update package.json

Ensure the main entry point is correctly set:

```json
{
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest --coverage",
    "format": "prettier --write \"**/*.js\""
  }
}
```

## Dependencies and Relationships

- **Depends on**: Task 1 (Project Setup and Configuration)
- **Required by**: 
  - Task 4 (Implement Todo Controller) - needs error handling
  - Task 5 (Implement API Routes) - needs app and validation
  - Task 6 (API Documentation) - needs middleware setup

## Success Criteria

1. ✅ Express application configured with proper middleware ordering
2. ✅ JSON body parsing enabled for API requests
3. ✅ Comprehensive validation rules for all todo operations
4. ✅ Centralized error handling with appropriate status codes
5. ✅ Environment-specific error messages (detailed in dev, generic in prod)
6. ✅ 404 handler for undefined routes
7. ✅ Server starts successfully on configured port
8. ✅ Graceful shutdown handling implemented

## Testing

To verify the implementation:

1. **Start the server**:
   ```bash
   npm run dev
   ```
   Should see: "Server running in development mode on port 3000"

2. **Test 404 handling**:
   ```bash
   curl http://localhost:3000/undefined
   ```
   Should return: `{"error":"Not Found","message":"Cannot GET /undefined"}`

3. **Test JSON parsing**:
   ```bash
   curl -X POST http://localhost:3000/test \
     -H "Content-Type: application/json" \
     -d '{"test":"data"}'
   ```

4. **Test validation** (will be fully tested with routes in Task 5)

## Common Issues and Solutions

1. **Port already in use**: 
   - Change PORT in .env file
   - Or kill the process using the port

2. **Middleware order issues**: 
   - Ensure body parser comes before routes
   - Error handler must be last

3. **Validation not working**: 
   - Check that handleValidationErrors is included in route chain
   - Verify validation rules match expected input format

## Code Quality Considerations

- Use consistent error formats throughout the application
- Keep middleware functions small and focused
- Maintain proper middleware ordering
- Use environment variables for configuration
- Implement proper logging for debugging

## Next Steps

After completing this task:
- Task 4: Implement Todo Controller (can start immediately)
- Task 5: Implement API Routes (requires both Task 3 and 4)

The Express application structure is now ready to handle HTTP requests with proper validation and error handling.