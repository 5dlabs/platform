# Autonomous Prompt: Add Error Handling Middleware

## Task Context
You are an AI assistant tasked with implementing comprehensive error handling middleware for an Express.js TypeScript application. This middleware will provide consistent error responses, proper logging, and centralized error management.

## Objective
Create a robust error handling system that catches all errors, formats them consistently, provides appropriate HTTP status codes, and logs errors for debugging purposes.

## Required Actions

### 1. Create Error Types and Interfaces
Create `src/types/error.ts` with error definitions:

```typescript
export interface ErrorResponse {
  error: string;
  code?: string;
  details?: string | object;
  timestamp: string;
  path: string;
  method: string;
  requestId?: string;
}

export interface CustomError extends Error {
  statusCode?: number;
  code?: string;
  details?: any;
}

export class ValidationError extends Error {
  statusCode = 400;
  code = 'VALIDATION_ERROR';
  
  constructor(message: string, public details?: any) {
    super(message);
    this.name = 'ValidationError';
    Object.setPrototypeOf(this, ValidationError.prototype);
  }
}

export class NotFoundError extends Error {
  statusCode = 404;
  code = 'NOT_FOUND';
  
  constructor(message: string = 'Resource not found', public details?: any) {
    super(message);
    this.name = 'NotFoundError';
    Object.setPrototypeOf(this, NotFoundError.prototype);
  }
}

export class ConflictError extends Error {
  statusCode = 409;
  code = 'CONFLICT';
  
  constructor(message: string, public details?: any) {
    super(message);
    this.name = 'ConflictError';
    Object.setPrototypeOf(this, ConflictError.prototype);
  }
}

export class UnauthorizedError extends Error {
  statusCode = 401;
  code = 'UNAUTHORIZED';
  
  constructor(message: string = 'Unauthorized', public details?: any) {
    super(message);
    this.name = 'UnauthorizedError';
    Object.setPrototypeOf(this, UnauthorizedError.prototype);
  }
}

export class ForbiddenError extends Error {
  statusCode = 403;
  code = 'FORBIDDEN';
  
  constructor(message: string = 'Forbidden', public details?: any) {
    super(message);
    this.name = 'ForbiddenError';
    Object.setPrototypeOf(this, ForbiddenError.prototype);
  }
}
```

### 2. Create Error Handling Middleware
Create `src/middleware/error.ts`:

```typescript
import { Request, Response, NextFunction } from 'express';
import { ErrorResponse, CustomError } from '../types/error';

// Generate unique request ID
function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// Add request ID to all requests
export const requestIdMiddleware = (req: Request, res: Response, next: NextFunction) => {
  (req as any).requestId = generateRequestId();
  next();
};

// Main error handling middleware
export const errorHandler = (
  error: CustomError,
  req: Request,
  res: Response,
  next: NextFunction
) => {
  const timestamp = new Date().toISOString();
  const requestId = (req as any).requestId;
  
  // Default error values
  let statusCode = error.statusCode || 500;
  let message = error.message || 'Internal server error';
  let code = error.code || 'INTERNAL_ERROR';
  let details = error.details;
  
  // Handle specific error types
  switch (error.name) {
    case 'ValidationError':
      statusCode = 400;
      code = 'VALIDATION_ERROR';
      message = 'Validation failed';
      break;
      
    case 'CastError':
      statusCode = 400;
      code = 'INVALID_FORMAT';
      message = 'Invalid data format';
      break;
      
    case 'MongoError':
      if (error.code === 11000) {
        statusCode = 409;
        code = 'DUPLICATE_ENTRY';
        message = 'Duplicate entry detected';
      }
      break;
      
    case 'JsonWebTokenError':
      statusCode = 401;
      code = 'INVALID_TOKEN';
      message = 'Invalid authentication token';
      break;
      
    case 'TokenExpiredError':
      statusCode = 401;
      code = 'TOKEN_EXPIRED';
      message = 'Authentication token expired';
      break;
      
    case 'SyntaxError':
      if (error.message.includes('JSON')) {
        statusCode = 400;
        code = 'INVALID_JSON';
        message = 'Invalid JSON format';
      }
      break;
  }
  
  // Log error for debugging (exclude sensitive information)
  const logError = {
    timestamp,
    requestId,
    method: req.method,
    path: req.path,
    statusCode,
    code,
    message,
    stack: process.env.NODE_ENV === 'development' ? error.stack : undefined,
    userAgent: req.get('User-Agent'),
    ip: req.ip
  };
  
  // Use different log levels based on error severity
  if (statusCode >= 500) {
    console.error('Server Error:', logError);
  } else if (statusCode >= 400) {
    console.warn('Client Error:', logError);
  } else {
    console.info('Request Error:', logError);
  }
  
  // Prepare error response
  const errorResponse: ErrorResponse = {
    error: message,
    code,
    timestamp,
    path: req.path,
    method: req.method,
    requestId
  };
  
  // Add details only in development or for specific error types
  if (details && (process.env.NODE_ENV === 'development' || statusCode < 500)) {
    errorResponse.details = details;
  }
  
  // Don't expose stack traces in production
  if (process.env.NODE_ENV === 'development' && error.stack) {
    errorResponse.details = {
      ...errorResponse.details,
      stack: error.stack
    };
  }
  
  res.status(statusCode).json(errorResponse);
};

// 404 handler for unmatched routes
export const notFoundHandler = (req: Request, res: Response, next: NextFunction) => {
  const error = new Error(`Route ${req.method} ${req.path} not found`);
  (error as CustomError).statusCode = 404;
  (error as CustomError).code = 'ROUTE_NOT_FOUND';
  next(error);
};

// Async error wrapper
export const asyncHandler = (fn: Function) => (req: Request, res: Response, next: NextFunction) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};
```

### 3. Create Error Utilities
Create `src/utils/errors.ts`:

```typescript
import { ValidationError, NotFoundError, ConflictError } from '../types/error';

export const throwValidationError = (message: string, details?: any): never => {
  throw new ValidationError(message, details);
};

export const throwNotFoundError = (message?: string, details?: any): never => {
  throw new NotFoundError(message, details);
};

export const throwConflictError = (message: string, details?: any): never => {
  throw new ConflictError(message, details);
};

export const handleValidationErrors = (errors: any[]): never => {
  const details = errors.map(error => ({
    field: error.path,
    message: error.message,
    value: error.value
  }));
  
  throwValidationError('Validation failed', details);
};
```

### 4. Update Main Application
Update `src/index.ts` to include error handling:

```typescript
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';

import healthRoutes from './routes/health';
import userRoutes from './routes/users';
import { errorHandler, notFoundHandler, requestIdMiddleware } from './middleware/error';

const app = express();
const port = process.env.PORT || 3000;

// Security middleware
app.use(helmet());
app.use(cors());

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: {
    error: 'Too many requests',
    code: 'RATE_LIMIT_EXCEEDED',
    timestamp: new Date().toISOString()
  }
});
app.use('/api/', limiter);

// Request parsing middleware
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Request ID middleware (for error tracking)
app.use(requestIdMiddleware);

// API routes
app.use('/api', healthRoutes);
app.use('/api', userRoutes);

// Default route
app.get('/', (req, res) => {
  res.json({ 
    message: 'Express TypeScript API Server',
    timestamp: new Date().toISOString(),
    environment: process.env.NODE_ENV || 'development',
    endpoints: {
      health: '/api/health',
      users: '/api/users'
    }
  });
});

// 404 handler for unmatched routes
app.use(notFoundHandler);

// Global error handler (must be last)
app.use(errorHandler);

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully');
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log('SIGINT received, shutting down gracefully');
  process.exit(0);
});

// Unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
  process.exit(1);
});

// Uncaught exceptions
process.on('uncaughtException', (error) => {
  console.error('Uncaught Exception:', error);
  process.exit(1);
});

app.listen(port, () => {
  console.log(`⚡️ Server is running at http://localhost:${port}`);
  console.log(`📊 Health check: http://localhost:${port}/api/health`);
  console.log(`👥 Users API: http://localhost:${port}/api/users`);
});

export default app;
```

### 5. Update User Routes with Error Handling
Update `src/routes/users.ts` to use the new error handling:

```typescript
import { Router, Request, Response, NextFunction } from 'express';
import { User, CreateUserRequest, UpdateUserRequest, UserResponse, userToResponse, isValidCreateUserRequest } from '../types/user';
import { ValidationError, NotFoundError, ConflictError } from '../types/error';
import { asyncHandler } from '../middleware/error';
import { v4 as uuidv4 } from 'uuid';

const router = Router();

// In-memory storage
let users: User[] = [];

// GET /users
router.get('/users', asyncHandler(async (req: Request, res: Response) => {
  const userResponses: UserResponse[] = users.map(userToResponse);
  res.json(userResponses);
}));

// GET /users/:id
router.get('/users/:id', asyncHandler(async (req: Request, res: Response) => {
  const { id } = req.params;
  const user = users.find(u => u.id === id);
  
  if (!user) {
    throw new NotFoundError('User not found');
  }
  
  res.json(userToResponse(user));
}));

// POST /users
router.post('/users', asyncHandler(async (req: Request, res: Response) => {
  const createRequest: CreateUserRequest = req.body;
  
  if (!isValidCreateUserRequest(createRequest)) {
    throw new ValidationError('Invalid user data', {
      name: 'Name is required and must be non-empty',
      email: 'Email is required and must be valid'
    });
  }
  
  const { name, email } = createRequest;
  
  // Check for duplicate email
  const existingUser = users.find(user => user.email.toLowerCase() === email.toLowerCase());
  if (existingUser) {
    throw new ConflictError('Email already exists', {
      email: 'A user with this email address already exists'
    });
  }
  
  // Create new user
  const newUser: User = {
    id: uuidv4(),
    name: name.trim(),
    email: email.toLowerCase().trim(),
    createdAt: new Date()
  };
  
  users.push(newUser);
  res.status(201).json(userToResponse(newUser));
}));

// PUT /users/:id
router.put('/users/:id', asyncHandler(async (req: Request, res: Response) => {
  const { id } = req.params;
  const updateRequest: UpdateUserRequest = req.body;
  
  const userIndex = users.findIndex(u => u.id === id);
  if (userIndex === -1) {
    throw new NotFoundError('User not found');
  }
  
  const user = users[userIndex];
  
  if (updateRequest.name !== undefined) {
    if (!updateRequest.name || updateRequest.name.trim().length === 0) {
      throw new ValidationError('Invalid name', {
        name: 'Name must be non-empty'
      });
    }
    user.name = updateRequest.name.trim();
  }
  
  if (updateRequest.email !== undefined) {
    // Check for duplicate email
    const existingUser = users.find(u => 
      u.id !== id && u.email.toLowerCase() === updateRequest.email!.toLowerCase()
    );
    if (existingUser) {
      throw new ConflictError('Email already exists', {
        email: 'Another user already has this email address'
      });
    }
    
    user.email = updateRequest.email.toLowerCase().trim();
  }
  
  users[userIndex] = user;
  res.json(userToResponse(user));
}));

// DELETE /users/:id
router.delete('/users/:id', asyncHandler(async (req: Request, res: Response) => {
  const { id } = req.params;
  const userIndex = users.findIndex(u => u.id === id);
  
  if (userIndex === -1) {
    throw new NotFoundError('User not found');
  }
  
  users.splice(userIndex, 1);
  res.status(204).send();
}));

export default router;
```

### 6. Install Additional Dependencies
Install security and utility packages:

```bash
npm install helmet cors express-rate-limit
npm install --save-dev @types/cors
```

## Validation Steps
1. **Build Test**: Run `npm run build` to ensure TypeScript compiles
2. **Error Response Test**: Test error responses for consistency
3. **Status Code Test**: Verify correct HTTP status codes
4. **Logging Test**: Check error logging functionality
5. **Security Test**: Verify no sensitive data in error responses

## Testing Commands

### Error Handling Tests
```bash
# Test 404 for non-existent route
curl -X GET http://localhost:3000/api/nonexistent

# Test validation error
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"","email":"invalid"}'

# Test not found error
curl -X GET http://localhost:3000/api/users/nonexistent-id

# Test conflict error
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","email":"duplicate@example.com"}'
  
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test2","email":"duplicate@example.com"}'

# Test JSON syntax error
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","email"}'
```

## Success Criteria
- [ ] Consistent error response format across all endpoints
- [ ] Proper HTTP status codes for different error types
- [ ] Request ID tracking for error correlation
- [ ] Appropriate error logging for debugging
- [ ] No sensitive information in error responses
- [ ] Custom error classes work correctly
- [ ] Async error handling works properly
- [ ] 404 handling for unmatched routes
- [ ] Rate limiting integration
- [ ] Graceful shutdown handling

## Final Deliverables
- [ ] `src/types/error.ts` with error definitions
- [ ] `src/middleware/error.ts` with error handling middleware
- [ ] `src/utils/errors.ts` with error utilities
- [ ] Updated `src/index.ts` with error middleware integration
- [ ] Updated user routes with proper error handling
- [ ] Security middleware integration (helmet, cors, rate limiting)
- [ ] Request ID generation and tracking
- [ ] Comprehensive error logging system