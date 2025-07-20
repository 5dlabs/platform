# Task 3: Implement Express Application and Middleware

## Overview
Set up the Express application with necessary middleware components and error handling. This task establishes the application layer and middleware infrastructure following the architecture's separation of concerns.

## Task Details
**ID**: 3  
**Title**: Implement Express Application and Middleware  
**Priority**: High  
**Dependencies**: [Task 1: Project Setup and Configuration](../task-1/task.md)  
**Status**: Pending

## Architecture Context
This task implements the Application Layer and Middleware components as defined in the [architecture document](../../architecture.md):
- Express application setup and configuration
- Middleware configuration for request processing
- Error handling middleware with consistent response formats
- Validation middleware using express-validator

Key architectural patterns:
- Centralized error handling
- Request validation pipeline
- Consistent error response format
- Middleware composition for cross-cutting concerns

## Product Requirements Alignment
Implements PRD technical requirements:
- Express.js 4.x framework setup
- Consistent JSON error responses
- HTTP status codes: 200, 201, 204, 400, 404, 500
- Request validation with descriptive error messages
- Environment configuration support

## Implementation Steps

### 1. Create Main Application File
Create `src/app.js`:
```javascript
const express = require('express');
const { validationResult } = require('express-validator');

// Initialize Express app
const app = express();

// Basic middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Request logging middleware (development only)
if (process.env.NODE_ENV !== 'production') {
  app.use((req, res, next) => {
    console.log(`${new Date().toISOString()} ${req.method} ${req.path}`);
    next();
  });
}

// CORS headers (if needed for frontend integration)
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization');
  if (req.method === 'OPTIONS') {
    return res.sendStatus(200);
  }
  next();
});

// Routes will be added here

// 404 handler - must be after all routes
app.use((req, res) => {
  res.status(404).json({
    error: {
      message: 'Resource not found',
      code: 'NOT_FOUND',
      path: req.path
    }
  });
});

// Global error handler - must be last
app.use((err, req, res, next) => {
  console.error('Error:', err);
  
  // Handle validation errors
  if (err.type === 'validation') {
    return res.status(400).json({
      error: {
        message: 'Validation failed',
        code: 'VALIDATION_ERROR',
        details: err.errors
      }
    });
  }
  
  // Handle other errors
  const status = err.status || 500;
  const message = process.env.NODE_ENV === 'production' 
    ? 'Internal server error' 
    : err.message;
    
  res.status(status).json({
    error: {
      message,
      code: err.code || 'INTERNAL_ERROR',
      ...(process.env.NODE_ENV !== 'production' && { stack: err.stack })
    }
  });
});

module.exports = app;
```

### 2. Create Validation Middleware
Create `src/middleware/validation.js`:
```javascript
const { body, param, query, validationResult } = require('express-validator');

// Validation middleware wrapper
const validate = (req, res, next) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    const error = new Error('Validation failed');
    error.type = 'validation';
    error.errors = errors.array().map(err => ({
      field: err.path,
      message: err.msg,
      value: err.value
    }));
    return next(error);
  }
  next();
};

// Todo validation rules
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
    validate
  ],
  
  update: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    body('title')
      .optional()
      .isString().withMessage('Title must be a string')
      .trim()
      .isLength({ min: 1, max: 200 }).withMessage('Title must be between 1 and 200 characters'),
    body('description')
      .optional()
      .isString().withMessage('Description must be a string')
      .trim()
      .isLength({ max: 1000 }).withMessage('Description cannot exceed 1000 characters'),
    body('completed')
      .optional()
      .isBoolean().withMessage('Completed must be a boolean'),
    validate
  ],
  
  getOne: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    validate
  ],
  
  delete: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    validate
  ],
  
  list: [
    query('completed')
      .optional()
      .isBoolean().withMessage('Completed must be a boolean'),
    query('limit')
      .optional()
      .isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1 and 100'),
    query('offset')
      .optional()
      .isInt({ min: 0 }).withMessage('Offset must be a non-negative integer'),
    validate
  ]
};

module.exports = todoValidation;
```

### 3. Create Error Handler Utilities
Create `src/middleware/errorHandler.js`:
```javascript
// Custom error classes
class AppError extends Error {
  constructor(message, status = 500, code = 'INTERNAL_ERROR') {
    super(message);
    this.status = status;
    this.code = code;
    this.name = this.constructor.name;
    Error.captureStackTrace(this, this.constructor);
  }
}

class NotFoundError extends AppError {
  constructor(resource = 'Resource') {
    super(`${resource} not found`, 404, 'NOT_FOUND');
  }
}

class ValidationError extends AppError {
  constructor(errors) {
    super('Validation failed', 400, 'VALIDATION_ERROR');
    this.errors = errors;
  }
}

// Async error wrapper
const asyncHandler = (fn) => (req, res, next) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

module.exports = {
  AppError,
  NotFoundError,
  ValidationError,
  asyncHandler
};
```

### 4. Create Server Entry Point
Create `server.js` in the root directory:
```javascript
require('dotenv').config();
const app = require('./src/app');

const PORT = process.env.PORT || 3000;

// Start server
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
  console.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});

module.exports = server;
```

### 5. Update App.js with Route Placeholder
Add to `src/app.js` after middleware setup:
```javascript
// API routes (to be implemented)
app.get('/api/health', (req, res) => {
  res.json({
    status: 'ok',
    timestamp: new Date().toISOString(),
    environment: process.env.NODE_ENV || 'development'
  });
});

// Todo routes will be mounted here
// app.use('/api/todos', todoRoutes);
```

## Success Criteria
- Express application starts without errors
- Health endpoint returns proper response
- Error handling middleware catches and formats errors correctly
- Validation middleware properly validates requests
- CORS headers are set for cross-origin requests
- Request logging works in development mode
- 404 errors return consistent format
- Server handles graceful shutdown

## Testing Considerations
- Test error handling with various error types
- Verify validation middleware with valid/invalid inputs
- Test CORS functionality
- Ensure health endpoint is accessible
- Test graceful shutdown handling

## Related Tasks
- **Previous**: [Task 1: Project Setup and Configuration](../task-1/task.md)
- **Next**: [Task 4: Implement Todo Controller](../task-4/task.md)
- **Related**: [Task 5: Implement API Routes](../task-5/task.md) will use this application setup

## References
- [Architecture Document](../../architecture.md) - Sections: Application Layer, Middleware, Error Handling Strategy
- [Product Requirements](../../prd.txt) - Section: Technical Requirements, Error Handling