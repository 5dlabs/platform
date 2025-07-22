# Task 5: Implement Global Error Handling Middleware

## Overview
Add centralized error handling and 404 middleware for consistent error responses across the API. This ensures all errors are handled gracefully with a uniform response format.

## Task Details
- **Priority**: High
- **Dependencies**: Task 4 (Develop User Management Endpoints)
- **Status**: Pending

## Implementation Guide

### 1. Create Error Handler Middleware
`/src/middleware/errorHandler.js`:
```javascript
export const errorHandler = (err, req, res, next) => {
  // Log error for debugging (in production, use proper logger)
  console.error(`[${new Date().toISOString()}] Error:`, err.stack);
  
  // Default to 500 if no status code set
  const status = err.status || err.statusCode || 500;
  
  // Send error response
  res.status(status).json({
    error: err.name || 'Internal Server Error',
    message: err.message || 'An unexpected error occurred',
    ...(process.env.NODE_ENV === 'development' && { stack: err.stack })
  });
};

export const notFoundHandler = (req, res, next) => {
  res.status(404).json({
    error: 'Not Found',
    message: `Cannot ${req.method} ${req.path}`
  });
};
```

### 2. Create Custom Error Classes
`/src/utils/errors.js`:
```javascript
export class AppError extends Error {
  constructor(message, status) {
    super(message);
    this.name = 'AppError';
    this.status = status;
    Error.captureStackTrace(this, this.constructor);
  }
}

export class ValidationError extends AppError {
  constructor(message) {
    super(message, 400);
    this.name = 'ValidationError';
  }
}

export class NotFoundError extends AppError {
  constructor(message = 'Resource not found') {
    super(message, 404);
    this.name = 'NotFoundError';
  }
}
```

### 3. Update Server Integration
In `/src/index.js`, add after all routes:
```javascript
import { errorHandler, notFoundHandler } from './middleware/errorHandler.js';

// ... existing routes ...

// 404 handler - must be after all routes
app.use(notFoundHandler);

// Global error handler - must be last
app.use(errorHandler);
```

### 4. Update Controllers to Use Error Classes
Example in user controller:
```javascript
import { ValidationError } from '../utils/errors.js';

export const addUser = (req, res, next) => {
  try {
    const { name, email } = req.body;
    
    if (!name || !email) {
      throw new ValidationError('Name and email are required');
    }
    
    // ... rest of logic
  } catch (error) {
    next(error); // Pass to error handler
  }
};
```

## Error Response Format
```json
{
  "error": "Error Type",
  "message": "Descriptive error message",
  "stack": "..." // Only in development
}
```

## Acceptance Criteria
- [ ] Global error handler catches all errors
- [ ] 404 handler for unknown routes
- [ ] Consistent JSON error format
- [ ] Stack traces only in development
- [ ] Custom error classes work correctly
- [ ] Errors logged with timestamps
- [ ] No unhandled promise rejections

## Test Strategy
1. Test 404 errors:
   ```bash
   curl http://localhost:3000/api/unknown
   ```

2. Test validation errors with proper format
3. Test internal server errors (500)
4. Verify stack traces only in development
5. Check error logging output
6. Test async error handling