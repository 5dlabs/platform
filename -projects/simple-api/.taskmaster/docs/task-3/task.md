# Task 3: Implement Express Application and Middleware

## Overview
This task establishes the Express.js application foundation with essential middleware configuration, error handling, and validation setup. It creates the core application structure that will handle HTTP requests, process them through middleware layers, and return appropriate responses.

## Task Details

### Priority
High

### Dependencies
- Task 1: Project Setup and Configuration (must be completed)
- Database setup is helpful but not strictly required for this task

### Status
Pending

## Implementation Guide

### 1. Create the Main Express Application

**File: `src/app.js`**
```javascript
const express = require('express');
const path = require('path');

// Create Express application
const app = express();

// Trust proxy headers in production
if (process.env.NODE_ENV === 'production') {
  app.set('trust proxy', 1);
}

// Core middleware
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Request logging middleware (development only)
if (process.env.NODE_ENV === 'development') {
  app.use((req, res, next) => {
    console.log(`${new Date().toISOString()} ${req.method} ${req.path}`);
    next();
  });
}

// CORS headers for API access
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Origin, X-Requested-With, Content-Type, Accept, Authorization');
  
  if (req.method === 'OPTIONS') {
    return res.sendStatus(200);
  }
  
  next();
});

// Security headers
app.use((req, res, next) => {
  res.header('X-Content-Type-Options', 'nosniff');
  res.header('X-Frame-Options', 'DENY');
  res.header('X-XSS-Protection', '1; mode=block');
  next();
});

// API response time header
app.use((req, res, next) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    res.set('X-Response-Time', `${duration}ms`);
  });
  
  next();
});

module.exports = app;
```

### 2. Create Validation Middleware

**File: `src/middleware/validation.js`**
```javascript
const { body, param, query, validationResult } = require('express-validator');

// Validation rules for todo operations
const todoValidation = {
  // Validation for creating a todo
  create: [
    body('title')
      .notEmpty().withMessage('Title is required')
      .isString().withMessage('Title must be a string')
      .trim()
      .isLength({ min: 1, max: 200 }).withMessage('Title must be between 1 and 200 characters'),
    body('description')
      .optional({ nullable: true })
      .isString().withMessage('Description must be a string')
      .trim()
      .isLength({ max: 1000 }).withMessage('Description cannot exceed 1000 characters')
  ],
  
  // Validation for updating a todo
  update: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID'),
    body('title')
      .optional()
      .isString().withMessage('Title must be a string')
      .trim()
      .isLength({ min: 1, max: 200 }).withMessage('Title must be between 1 and 200 characters'),
    body('description')
      .optional({ nullable: true })
      .isString().withMessage('Description must be a string')
      .trim()
      .isLength({ max: 1000 }).withMessage('Description cannot exceed 1000 characters'),
    body('completed')
      .optional()
      .isBoolean().withMessage('Completed must be a boolean')
      .toBoolean()
  ],
  
  // Validation for getting a single todo
  getOne: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID')
      .toInt()
  ],
  
  // Validation for deleting a todo
  delete: [
    param('id')
      .isInt({ min: 1 }).withMessage('Invalid todo ID')
      .toInt()
  ],
  
  // Validation for listing todos
  list: [
    query('completed')
      .optional()
      .isBoolean().withMessage('Completed must be true or false')
      .toBoolean(),
    query('limit')
      .optional()
      .isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1 and 100')
      .toInt(),
    query('offset')
      .optional()
      .isInt({ min: 0 }).withMessage('Offset must be a non-negative integer')
      .toInt()
  ]
};

// Middleware to handle validation errors
const handleValidationErrors = (req, res, next) => {
  const errors = validationResult(req);
  
  if (!errors.isEmpty()) {
    return res.status(400).json({
      error: {
        message: 'Validation failed',
        code: 'VALIDATION_ERROR',
        details: errors.array().map(err => ({
          field: err.path,
          message: err.msg,
          value: err.value
        }))
      }
    });
  }
  
  next();
};

module.exports = {
  todoValidation,
  handleValidationErrors
};
```

### 3. Create Error Handling Middleware

**File: `src/middleware/errorHandler.js`**
```javascript
/**
 * Central error handling middleware
 */
const errorHandler = (err, req, res, next) => {
  // Log error details in development
  if (process.env.NODE_ENV === 'development') {
    console.error('Error:', err);
  }
  
  // Default error response
  let status = err.status || 500;
  let message = err.message || 'Internal Server Error';
  let code = err.code || 'INTERNAL_ERROR';
  
  // Handle specific error types
  if (err.name === 'ValidationError') {
    status = 400;
    code = 'VALIDATION_ERROR';
  } else if (err.name === 'CastError') {
    status = 400;
    message = 'Invalid ID format';
    code = 'INVALID_ID';
  } else if (err.code === 'SQLITE_CONSTRAINT') {
    status = 400;
    message = 'Database constraint violation';
    code = 'CONSTRAINT_ERROR';
  }
  
  // Send error response
  res.status(status).json({
    error: {
      message,
      code,
      ...(process.env.NODE_ENV === 'development' && {
        stack: err.stack,
        details: err
      })
    }
  });
};

/**
 * 404 Not Found handler
 */
const notFoundHandler = (req, res) => {
  res.status(404).json({
    error: {
      message: 'Resource not found',
      code: 'NOT_FOUND',
      path: req.path
    }
  });
};

/**
 * Async route handler wrapper to catch promise rejections
 */
const asyncHandler = (fn) => (req, res, next) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

module.exports = {
  errorHandler,
  notFoundHandler,
  asyncHandler
};
```

### 4. Update Main Application with Middleware

**Update `src/app.js`** to include error handling:
```javascript
const express = require('express');
const path = require('path');
const { errorHandler, notFoundHandler } = require('./middleware/errorHandler');

// Create Express application
const app = express();

// [Previous middleware configuration remains the same...]

// Import routes (to be implemented in Task 5)
// const todoRoutes = require('./routes/todos');
// const healthRoutes = require('./routes/health');

// Register routes (to be implemented in Task 5)
// app.use('/api/todos', todoRoutes);
// app.use('/api/health', healthRoutes);

// 404 handler (must be after all routes)
app.use(notFoundHandler);

// Global error handler (must be last)
app.use(errorHandler);

module.exports = app;
```

### 5. Create Server Entry Point

**File: `server.js`** (in project root)
```javascript
require('dotenv').config();

const app = require('./src/app');

const PORT = process.env.PORT || 3000;

// Start server
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT} in ${process.env.NODE_ENV || 'development'} mode`);
});

// Graceful shutdown handling
const gracefulShutdown = () => {
  console.log('Received shutdown signal, closing server...');
  
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
  
  // Force close after 10 seconds
  setTimeout(() => {
    console.error('Could not close connections in time, forcefully shutting down');
    process.exit(1);
  }, 10000);
};

process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);
```

### 6. Create Middleware Index

**File: `src/middleware/index.js`**
```javascript
const { todoValidation, handleValidationErrors } = require('./validation');
const { errorHandler, notFoundHandler, asyncHandler } = require('./errorHandler');

module.exports = {
  todoValidation,
  handleValidationErrors,
  errorHandler,
  notFoundHandler,
  asyncHandler
};
```

## Key Implementation Considerations

### Architecture Alignment
- Follows the middleware layer specified in the architecture document
- Separates concerns between validation, error handling, and request processing
- Provides consistent error response format across the API

### Security Best Practices
- Input validation and sanitization using express-validator
- Security headers to prevent common attacks
- Request size limits to prevent DoS attacks
- CORS configuration for API access control

### Performance Optimizations
- Response time tracking for monitoring
- Conditional middleware based on environment
- Efficient error handling without exposing sensitive data

### Developer Experience
- Detailed error messages in development mode
- Request logging for debugging
- Validation rules are reusable and well-documented
- Async error handling simplified with wrapper

## Testing Considerations

The middleware should be tested for:
1. Validation rules correctly accept/reject inputs
2. Error handler properly formats different error types
3. CORS headers are set correctly
4. Security headers are present
5. 404 handler catches undefined routes

Example test:
```javascript
const request = require('supertest');
const app = require('../src/app');

describe('Middleware', () => {
  test('should return 404 for unknown routes', async () => {
    const res = await request(app).get('/unknown');
    expect(res.status).toBe(404);
    expect(res.body.error.code).toBe('NOT_FOUND');
  });
});
```

## Common Issues and Solutions

### Issue: Validation Not Working
**Solution**: Ensure `handleValidationErrors` is called after validation rules in route definitions

### Issue: CORS Errors in Browser
**Solution**: Check that OPTIONS requests are handled and appropriate headers are set

### Issue: Large Request Bodies Rejected
**Solution**: Adjust the limit in `express.json()` middleware configuration

## Next Steps
After completing this task:
1. Test the Express application starts successfully
2. Verify middleware is working with test requests
3. Proceed to Task 4: Implement Todo Controller
4. Then Task 5 will connect routes to the application

## References
- [Express.js Middleware Documentation](https://expressjs.com/en/guide/using-middleware.html)
- [Express-Validator Documentation](https://express-validator.github.io/docs/)
- [Architecture Document - Middleware Layer](../architecture.md#middleware)