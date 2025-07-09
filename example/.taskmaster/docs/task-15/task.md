# Task 15: Add Error Handling Middleware

## Overview
Implement global error handling middleware for consistent error responses across the Express application, providing proper HTTP status codes and formatted error messages.

## Description
This task involves creating centralized error handling middleware that catches and processes errors from all routes and middleware, ensuring consistent error response format and appropriate HTTP status codes for different error types.

## Priority
Medium

## Dependencies
- Task 11: Initialize Express TypeScript Project (must be completed first)

## Implementation Steps

### 1. Create error middleware file
- Create `src/middleware/error.ts` with Express error handler setup
- Define error response interfaces and types
- Implement different error type handlers

### 2. Implement error handler function
- Create error handling middleware function with proper error formatting
- Handle different error types (validation, not found, server errors)
- Log errors appropriately for debugging
- Return consistent error response format

### 3. Wire error middleware to app
- Import error middleware in main Express app
- Mount error middleware as the last middleware in the stack
- Ensure proper error propagation from routes

## Implementation Details

### Error Middleware Structure
```typescript
import { Request, Response, NextFunction } from 'express';

interface ErrorResponse {
  error: string;
  details?: string | object;
  timestamp: string;
  path: string;
  method: string;
}

interface CustomError extends Error {
  statusCode?: number;
  code?: string;
  details?: any;
}

const errorHandler = (
  error: CustomError,
  req: Request,
  res: Response,
  next: NextFunction
) => {
  const timestamp = new Date().toISOString();
  
  // Log error for debugging
  console.error(`[${timestamp}] ${req.method} ${req.path}:`, error);
  
  // Default error response
  let statusCode = error.statusCode || 500;
  let message = error.message || 'Internal server error';
  let details = error.details;
  
  // Handle specific error types
  if (error.name === 'ValidationError') {
    statusCode = 400;
    message = 'Validation failed';
  } else if (error.name === 'CastError') {
    statusCode = 400;
    message = 'Invalid ID format';
  } else if (error.code === 'ENOENT') {
    statusCode = 404;
    message = 'Resource not found';
  }
  
  // Prepare error response
  const errorResponse: ErrorResponse = {
    error: message,
    timestamp,
    path: req.path,
    method: req.method
  };
  
  if (details) {
    errorResponse.details = details;
  }
  
  res.status(statusCode).json(errorResponse);
};

export default errorHandler;
```

### Integration with Main App
```typescript
// In src/index.ts
import errorHandler from './middleware/error';

// Routes
app.use('/api', userRoutes);
app.use('/api', healthRoutes);

// Error handling middleware (must be last)
app.use(errorHandler);
```

### Custom Error Classes
```typescript
export class ValidationError extends Error {
  statusCode = 400;
  
  constructor(message: string, public details?: any) {
    super(message);
    this.name = 'ValidationError';
  }
}

export class NotFoundError extends Error {
  statusCode = 404;
  
  constructor(message: string = 'Resource not found') {
    super(message);
    this.name = 'NotFoundError';
  }
}

export class ConflictError extends Error {
  statusCode = 409;
  
  constructor(message: string, public details?: any) {
    super(message);
    this.name = 'ConflictError';
  }
}
```

## File Structure
```
src/
├── middleware/
│   └── error.ts
├── routes/
│   ├── users.ts (updated)
│   └── health.ts
└── index.ts (updated)
```

## Test Strategy
- Test with invalid requests to verify error responses
- Test different error types (validation, not found, server errors)
- Verify error response format consistency
- Test error logging functionality
- Ensure proper HTTP status codes

## Expected Outcomes
- Consistent error response format across all endpoints
- Proper HTTP status codes for different error types
- Centralized error logging for debugging
- Improved error handling maintainability
- Better client error handling experience

## Common Issues
- **Middleware order**: Error middleware must be last in the stack
- **Error propagation**: Ensure routes call `next(error)` properly
- **Information disclosure**: Avoid exposing sensitive information in errors
- **Logging**: Balance between debugging info and security

## Enhanced Features (Optional)
- Error code standardization
- Request ID tracking for error correlation
- Rate limiting for error responses
- Custom error pages for web interfaces
- Integration with monitoring systems
- Error reporting to external services

## Integration Points
- Works with user routes for validation errors
- Integrates with health check endpoints
- Supports future middleware additions
- Compatible with authentication middleware
- Suitable for API versioning