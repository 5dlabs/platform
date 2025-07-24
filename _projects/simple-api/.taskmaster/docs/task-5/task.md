# Task 5: Implement Global Error Handling Middleware

## Overview
Add centralized error handling and 404 middleware for consistent error responses across the API. This ensures all errors are handled gracefully with a uniform response format.

**NEW FEATURES**: Request ID tracking for better debugging and health check integration for error monitoring.

## Task Details
- **Priority**: High
- **Dependencies**: Task 4 (Develop User Management Endpoints)
- **Status**: Pending

## Implementation Guide

### 1. Create Request ID Middleware
`/src/middleware/requestId.js`:
```javascript
import { randomUUID } from 'crypto';

export const requestIdMiddleware = (req, res, next) => {
  req.requestId = randomUUID();
  res.setHeader('X-Request-ID', req.requestId);
  next();
};
```

### 2. Create Error Handler Middleware
`/src/middleware/errorHandler.js`:
```javascript
// Error statistics tracking
let errorStats = {
  total: 0,
  byType: {},
  lastError: null,
  lastReset: new Date()
};

export const errorHandler = (err, req, res, next) => {
  // Update error statistics
  errorStats.total++;
  errorStats.byType[err.name] = (errorStats.byType[err.name] || 0) + 1;
  errorStats.lastError = new Date().toISOString();

  // Log error with request ID for debugging
  console.error(`[${new Date().toISOString()}] Error (${req.requestId || 'no-id'}):`, err.stack);

  // Default to 500 if no status code set
  const status = err.status || err.statusCode || 500;

  // Send error response with request ID
  res.status(status).json({
    error: err.name || 'Internal Server Error',
    message: err.message || 'An unexpected error occurred',
    requestId: req.requestId,
    ...(process.env.NODE_ENV === 'development' && { stack: err.stack })
  });
};

export const notFoundHandler = (req, res, next) => {
  res.status(404).json({
    error: 'Not Found',
    message: `Cannot ${req.method} ${req.path}`,
    requestId: req.requestId
  });
};

// Export error stats for health endpoint
export const getErrorStats = () => ({
  ...errorStats,
  uptime: Date.now() - errorStats.lastReset.getTime()
});

// Reset stats (called daily)
export const resetErrorStats = () => {
  errorStats = {
    total: 0,
    byType: {},
    lastError: null,
    lastReset: new Date()
  };
};
```

### 3. Create Custom Error Classes
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

export class RateLimitError extends AppError {
  constructor(message = 'Too many requests') {
    super(message, 429);
    this.name = 'RateLimitError';
  }
}
```

### 4. Update Health Endpoint
In `/src/routes/health.js` (or create if doesn't exist):
```javascript
import { Router } from 'express';
import { getErrorStats } from '../middleware/errorHandler.js';

const router = Router();

router.get('/', (req, res) => {
  const errorStats = getErrorStats();

  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    requestId: req.requestId,
    errors: {
      total: errorStats.total,
      byType: errorStats.byType,
      lastError: errorStats.lastError,
      lastReset: errorStats.lastReset
    }
  });
});

export default router;
```

### 5. Update Server Integration
In `/src/index.js`, add middleware in correct order:
```javascript
import { requestIdMiddleware } from './middleware/requestId.js';
import { errorHandler, notFoundHandler, resetErrorStats } from './middleware/errorHandler.js';
import healthRoutes from './routes/health.js';

// Request ID middleware - must be first
app.use(requestIdMiddleware);

// Health endpoint
app.use('/api/health', healthRoutes);

// ... existing routes ...

// 404 handler - must be after all routes
app.use(notFoundHandler);

// Global error handler - must be last
app.use(errorHandler);

// Reset error stats daily
setInterval(resetErrorStats, 24 * 60 * 60 * 1000);
```

### 6. Update Controllers to Use Error Classes
Example in user controller:
```javascript
import { ValidationError, RateLimitError } from '../utils/errors.js';

export const addUser = (req, res, next) => {
  try {
    const { name, email } = req.body;

    if (!name || !email) {
      throw new ValidationError('Name and email are required');
    }

    // Example rate limiting logic
    if (req.rateLimit && req.rateLimit.remaining === 0) {
      throw new RateLimitError('Rate limit exceeded');
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
  "requestId": "req_123456789",
  "stack": "..." // Only in development
}
```

## NEW: Health Endpoint Response
```json
{
  "status": "healthy",
  "timestamp": "2025-01-22T10:00:00.000Z",
  "uptime": 3600,
  "requestId": "req_987654321",
  "errors": {
    "total": 15,
    "byType": {
      "ValidationError": 8,
      "NotFoundError": 6,
      "AppError": 1
    },
    "lastError": "2025-01-22T09:45:00.000Z",
    "lastReset": "2025-01-22T00:00:00.000Z"
  }
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
- [ ] **NEW: Request IDs in all responses**
- [ ] **NEW: Request IDs in error logs**
- [ ] **NEW: Health endpoint shows error statistics**
- [ ] **NEW: Error counters working correctly**

## Test Strategy
1. Test 404 errors:
   ```bash
   curl http://localhost:3000/api/unknown
   ```

2. Test validation errors with proper format and request ID
3. Test internal server errors (500)
4. Verify stack traces only in development
5. Check error logging output includes request IDs
6. Test async error handling
7. **NEW: Test health endpoint for error statistics**
8. **NEW: Verify request ID consistency across logs and responses**
9. **NEW: Test rate limiting error scenarios**