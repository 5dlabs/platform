# Task 5: Add Request Validation and Error Handling - Autonomous AI Agent Prompt

You are tasked with implementing comprehensive input validation and standardized error handling across the Express application. This includes setting up express-validator for declarative validation, implementing rate limiting to prevent abuse, and creating a centralized error handling system.

## Your Mission

Enhance the application's security and robustness by adding input validation to all endpoints, implementing rate limiting on sensitive routes, creating custom error classes for consistent error handling, and establishing a centralized error management system.

## Prerequisites

Ensure Tasks 1-4 are complete:
- Express server with middleware
- Database with models
- JWT authentication system
- Task management API endpoints

## Step-by-Step Instructions

### 1. Install Required Dependencies

```bash
npm install express-validator@^7.0.1 express-rate-limit@^7.2.0
```

### 2. Create Validation Middleware

Create `src/middleware/validation.js`:

```javascript
const { body, param, query, validationResult } = require('express-validator');

// Helper to check validation results
const handleValidationErrors = (req, res, next) => {
  const errors = validationResult(req);
  
  if (!errors.isEmpty()) {
    const error = errors.array()[0];
    
    return res.status(400).json({
      error: {
        message: error.msg,
        field: error.path,
        code: 'VALIDATION_ERROR',
        value: error.value
      }
    });
  }
  
  next();
};

// Common validation rules
const validationRules = {
  // Auth validations
  email: body('email')
    .trim()
    .notEmpty().withMessage('Email is required')
    .isEmail().withMessage('Invalid email format')
    .normalizeEmail()
    .isLength({ max: 255 }).withMessage('Email must be less than 255 characters'),
  
  password: body('password')
    .notEmpty().withMessage('Password is required')
    .isLength({ min: 8 }).withMessage('Password must be at least 8 characters')
    .isLength({ max: 128 }).withMessage('Password must be less than 128 characters'),
  
  // Optional: Add password complexity
  // .matches(/\d/).withMessage('Password must contain at least one number')
  // .matches(/[A-Z]/).withMessage('Password must contain at least one uppercase letter'),
  
  // Task validations
  taskTitle: body('title')
    .trim()
    .notEmpty().withMessage('Title is required')
    .isLength({ min: 1, max: 255 }).withMessage('Title must be between 1 and 255 characters'),
  
  taskDescription: body('description')
    .optional({ nullable: true })
    .trim()
    .isLength({ max: 1000 }).withMessage('Description must be less than 1000 characters'),
  
  taskCompleted: body('completed')
    .optional()
    .isBoolean().withMessage('Completed must be a boolean value'),
  
  // ID validations
  idParam: param('id')
    .isInt({ min: 1 }).withMessage('Invalid ID format')
    .toInt(),
  
  // Query validations
  completedQuery: query('completed')
    .optional()
    .isIn(['true', 'false']).withMessage('Completed must be true or false'),
  
  limitQuery: query('limit')
    .optional()
    .isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1 and 100')
    .toInt(),
  
  offsetQuery: query('offset')
    .optional()
    .isInt({ min: 0 }).withMessage('Offset must be 0 or greater')
    .toInt(),
  
  // Refresh token validation
  refreshToken: body('refreshToken')
    .notEmpty().withMessage('Refresh token is required')
    .isString().withMessage('Refresh token must be a string')
};

// Validation chains for each endpoint
const validate = {
  // Auth endpoints
  register: [
    validationRules.email,
    validationRules.password,
    handleValidationErrors
  ],
  
  login: [
    validationRules.email,
    validationRules.password,
    handleValidationErrors
  ],
  
  refreshToken: [
    validationRules.refreshToken,
    handleValidationErrors
  ],
  
  // Task endpoints
  createTask: [
    validationRules.taskTitle,
    validationRules.taskDescription,
    handleValidationErrors
  ],
  
  updateTask: [
    validationRules.idParam,
    body('title')
      .optional()
      .trim()
      .notEmpty().withMessage('Title cannot be empty if provided')
      .isLength({ max: 255 }).withMessage('Title must be less than 255 characters'),
    validationRules.taskDescription,
    validationRules.taskCompleted,
    handleValidationErrors
  ],
  
  getTask: [
    validationRules.idParam,
    handleValidationErrors
  ],
  
  deleteTask: [
    validationRules.idParam,
    handleValidationErrors
  ],
  
  listTasks: [
    validationRules.completedQuery,
    validationRules.limitQuery,
    validationRules.offsetQuery,
    handleValidationErrors
  ]
};

module.exports = {
  validate,
  handleValidationErrors,
  validationRules
};
```

### 3. Create Custom Error Classes

Create `src/utils/errors.js`:

```javascript
// Base error class for operational errors
class AppError extends Error {
  constructor(message, statusCode, code) {
    super(message);
    this.statusCode = statusCode;
    this.code = code;
    this.isOperational = true;
    
    Error.captureStackTrace(this, this.constructor);
  }
}

// Specific error classes
class ValidationError extends AppError {
  constructor(message, field = null) {
    super(message, 400, 'VALIDATION_ERROR');
    this.field = field;
  }
}

class AuthenticationError extends AppError {
  constructor(message = 'Authentication failed') {
    super(message, 401, 'AUTHENTICATION_ERROR');
  }
}

class AuthorizationError extends AppError {
  constructor(message = 'Access denied') {
    super(message, 403, 'AUTHORIZATION_ERROR');
  }
}

class NotFoundError extends AppError {
  constructor(resource = 'Resource') {
    super(`${resource} not found`, 404, 'NOT_FOUND');
  }
}

class ConflictError extends AppError {
  constructor(message = 'Resource conflict') {
    super(message, 409, 'CONFLICT_ERROR');
  }
}

class RateLimitError extends AppError {
  constructor(message = 'Too many requests, please try again later') {
    super(message, 429, 'RATE_LIMIT_ERROR');
  }
}

// Global error handler middleware
const errorHandler = (err, req, res, next) => {
  // Log error details
  console.error('Error:', {
    message: err.message,
    stack: err.stack,
    statusCode: err.statusCode || 500,
    code: err.code || 'UNKNOWN_ERROR',
    path: req.path,
    method: req.method,
    ip: req.ip,
    userId: req.user?.id
  });
  
  // Default to 500 server error
  let statusCode = err.statusCode || 500;
  let message = err.message || 'Internal server error';
  let code = err.code || 'INTERNAL_ERROR';
  
  // Handle specific error types
  if (err.name === 'ValidationError' && !err.isOperational) {
    // Mongoose/Database validation error
    statusCode = 400;
    message = 'Validation failed';
    code = 'VALIDATION_ERROR';
  } else if (err.name === 'CastError') {
    // Invalid ObjectId
    statusCode = 400;
    message = 'Invalid ID format';
    code = 'INVALID_ID';
  } else if (err.code === 'SQLITE_CONSTRAINT') {
    // Database constraint error
    statusCode = 409;
    message = 'Database constraint violation';
    code = 'CONSTRAINT_ERROR';
  }
  
  // Don't leak error details in production
  if (!err.isOperational && process.env.NODE_ENV === 'production') {
    message = 'Something went wrong';
  }
  
  // Send error response
  res.status(statusCode).json({
    error: {
      message,
      code,
      ...(err.field && { field: err.field }),
      ...(process.env.NODE_ENV === 'development' && !err.isOperational && { 
        stack: err.stack,
        details: err.message 
      })
    }
  });
};

// Async handler to catch errors in async route handlers
const asyncHandler = (fn) => (req, res, next) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

module.exports = {
  AppError,
  ValidationError,
  AuthenticationError,
  AuthorizationError,
  NotFoundError,
  ConflictError,
  RateLimitError,
  errorHandler,
  asyncHandler
};
```

### 4. Implement Rate Limiting

Create `src/middleware/rateLimiter.js`:

```javascript
const rateLimit = require('express-rate-limit');
const { RateLimitError } = require('../utils/errors');

// Helper to create rate limiter with custom options
const createRateLimiter = (options) => {
  return rateLimit({
    windowMs: options.windowMs,
    max: options.max,
    message: options.message,
    standardHeaders: true, // Return rate limit info in headers
    legacyHeaders: false, // Disable X-RateLimit headers
    skipSuccessfulRequests: false,
    skipFailedRequests: false,
    
    // Custom handler for rate limit errors
    handler: (req, res, next) => {
      next(new RateLimitError(options.message));
    },
    
    // Skip rate limiting in test environment
    skip: (req) => {
      return process.env.NODE_ENV === 'test';
    },
    
    // Key generator (default uses IP)
    keyGenerator: (req) => {
      // Use user ID if authenticated, otherwise use IP
      return req.user?.id || req.ip;
    }
  });
};

// Different rate limiters for different use cases
const limiters = {
  // General API rate limiter
  general: createRateLimiter({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100, // 100 requests per window
    message: 'Too many requests from this IP, please try again later'
  }),
  
  // Strict rate limiter for auth endpoints
  auth: createRateLimiter({
    windowMs: 60 * 60 * 1000, // 1 hour
    max: 5, // 5 attempts per hour
    message: 'Too many authentication attempts, please try again later'
  }),
  
  // Medium rate limiter for resource creation
  create: createRateLimiter({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 30, // 30 creations per window
    message: 'Too many creation requests, please slow down'
  }),
  
  // Lenient rate limiter for read operations
  read: createRateLimiter({
    windowMs: 1 * 60 * 1000, // 1 minute
    max: 60, // 60 requests per minute
    message: 'Too many requests, please slow down'
  })
};

module.exports = limiters;
```

### 5. Update Authentication Routes

Update `src/routes/auth.js` to use validation and error handling:

```javascript
const express = require('express');
const router = express.Router();
const User = require('../models/User');
const { hashPassword, comparePassword } = require('../utils/password');
const { generateToken, generateRefreshToken, verifyToken } = require('../utils/jwt');
const { validate } = require('../middleware/validation');
const limiters = require('../middleware/rateLimiter');
const { 
  ConflictError, 
  AuthenticationError, 
  NotFoundError,
  asyncHandler 
} = require('../utils/errors');

// POST /auth/register
router.post('/register', 
  limiters.auth,
  validate.register,
  asyncHandler(async (req, res) => {
    const { email, password } = req.body;
    
    // Check if user already exists
    const existingUser = User.findByEmail(email);
    if (existingUser) {
      throw new ConflictError('Email already registered');
    }
    
    // Hash password and create user
    const hashedPassword = await hashPassword(password);
    const user = User.create(email, hashedPassword);
    
    // Generate tokens
    const tokenPayload = {
      userId: user.id,
      email: user.email
    };
    
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    console.log(`New user registered: ${user.email}`);
    
    res.status(201).json({
      message: 'User registered successfully',
      user: {
        id: user.id,
        email: user.email
      },
      tokens: {
        accessToken,
        refreshToken,
        expiresIn: '24h'
      }
    });
  })
);

// POST /auth/login
router.post('/login',
  limiters.auth,
  validate.login,
  asyncHandler(async (req, res) => {
    const { email, password } = req.body;
    
    // Find user by email
    const user = User.findByEmail(email);
    if (!user) {
      throw new AuthenticationError('Invalid credentials');
    }
    
    // Verify password
    const isValidPassword = await comparePassword(password, user.password);
    if (!isValidPassword) {
      throw new AuthenticationError('Invalid credentials');
    }
    
    // Generate tokens
    const tokenPayload = {
      userId: user.id,
      email: user.email
    };
    
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    console.log(`User logged in: ${user.email}`);
    
    res.json({
      message: 'Login successful',
      user: {
        id: user.id,
        email: user.email
      },
      tokens: {
        accessToken,
        refreshToken,
        expiresIn: '24h'
      }
    });
  })
);

// POST /auth/refresh
router.post('/refresh',
  limiters.general,
  validate.refreshToken,
  asyncHandler(async (req, res) => {
    const { refreshToken } = req.body;
    
    // Verify refresh token
    let decoded;
    try {
      decoded = verifyToken(refreshToken);
    } catch (error) {
      throw new AuthenticationError('Invalid refresh token');
    }
    
    // Verify user still exists
    const user = User.findById(decoded.userId);
    if (!user) {
      throw new NotFoundError('User');
    }
    
    // Generate new access token
    const tokenPayload = {
      userId: decoded.userId,
      email: decoded.email
    };
    
    const accessToken = generateToken(tokenPayload);
    
    res.json({
      accessToken,
      expiresIn: '24h'
    });
  })
);

// GET /auth/me
router.get('/me', 
  limiters.general,
  require('../middleware/auth').authenticateToken,
  asyncHandler(async (req, res) => {
    const user = User.findById(req.user.id);
    
    if (!user) {
      throw new NotFoundError('User');
    }
    
    res.json({
      user: {
        id: user.id,
        email: user.email,
        createdAt: user.created_at
      }
    });
  })
);

module.exports = router;
```

### 6. Update Task Routes

Update `src/routes/tasks.js` to use validation and error handling:

```javascript
const express = require('express');
const router = express.Router();
const Task = require('../models/Task');
const { authenticateToken } = require('../middleware/auth');
const { validate } = require('../middleware/validation');
const limiters = require('../middleware/rateLimiter');
const { 
  NotFoundError, 
  AuthorizationError, 
  ValidationError,
  asyncHandler 
} = require('../utils/errors');

// Apply authentication to all routes
router.use(authenticateToken);

// GET /api/tasks
router.get('/', 
  limiters.read,
  validate.listTasks,
  asyncHandler(async (req, res) => {
    const { completed, limit = 20, offset = 0 } = req.query;
    
    // Build filters (values already validated and converted)
    const filters = {
      ...(completed !== undefined && { completed: completed === 'true' }),
      limit: limit,
      offset: offset
    };
    
    // Get tasks and count
    const tasks = Task.findByUserId(req.user.id, filters);
    const totalCount = Task.countByUserId(req.user.id, { 
      completed: filters.completed 
    });
    
    // Calculate pagination metadata
    const hasNext = filters.offset + tasks.length < totalCount;
    const hasPrev = filters.offset > 0;
    
    res.json({
      tasks: tasks.map(task => ({
        id: task.id,
        title: task.title,
        description: task.description,
        completed: Boolean(task.completed),
        createdAt: task.created_at,
        updatedAt: task.updated_at
      })),
      pagination: {
        total: totalCount,
        limit: filters.limit,
        offset: filters.offset,
        hasNext,
        hasPrev
      }
    });
  })
);

// POST /api/tasks
router.post('/',
  limiters.create,
  validate.createTask,
  asyncHandler(async (req, res) => {
    const { title, description } = req.body;
    
    // Create task
    const task = Task.create(
      req.user.id,
      title,
      description || null
    );
    
    res.status(201).json({
      id: task.id,
      title: task.title,
      description: task.description,
      completed: task.completed,
      createdAt: task.created_at,
      updatedAt: task.updated_at
    });
  })
);

// GET /api/tasks/:id
router.get('/:id',
  limiters.read,
  validate.getTask,
  asyncHandler(async (req, res) => {
    const taskId = req.params.id; // Already converted to int by validator
    
    // Find task
    const task = Task.findById(taskId);
    
    if (!task) {
      throw new NotFoundError('Task');
    }
    
    // Check ownership
    if (task.user_id !== req.user.id) {
      throw new AuthorizationError();
    }
    
    res.json({
      id: task.id,
      title: task.title,
      description: task.description,
      completed: Boolean(task.completed),
      createdAt: task.created_at,
      updatedAt: task.updated_at
    });
  })
);

// PUT /api/tasks/:id
router.put('/:id',
  limiters.general,
  validate.updateTask,
  asyncHandler(async (req, res) => {
    const taskId = req.params.id; // Already converted to int
    const { title, description, completed } = req.body;
    
    // Build updates object
    const updates = {};
    if (title !== undefined) updates.title = title;
    if (description !== undefined) updates.description = description;
    if (completed !== undefined) updates.completed = completed;
    
    // Check if there are any updates
    if (Object.keys(updates).length === 0) {
      throw new ValidationError('No valid fields to update');
    }
    
    // Perform update (includes ownership check)
    const success = Task.update(taskId, req.user.id, updates);
    
    if (!success) {
      throw new NotFoundError('Task');
    }
    
    // Fetch updated task
    const updatedTask = Task.findById(taskId);
    
    res.json({
      id: updatedTask.id,
      title: updatedTask.title,
      description: updatedTask.description,
      completed: Boolean(updatedTask.completed),
      createdAt: updatedTask.created_at,
      updatedAt: updatedTask.updated_at
    });
  })
);

// DELETE /api/tasks/:id
router.delete('/:id',
  limiters.general,
  validate.deleteTask,
  asyncHandler(async (req, res) => {
    const taskId = req.params.id; // Already converted to int
    
    // Delete task (includes ownership check)
    const success = Task.delete(taskId, req.user.id);
    
    if (!success) {
      throw new NotFoundError('Task');
    }
    
    res.status(204).send();
  })
);

module.exports = router;
```

### 7. Update Main Application

Update `src/app.js` to include the error handler:

```javascript
// ... existing requires ...
const { errorHandler } = require('./utils/errors');

// ... existing middleware and routes ...

// 404 handler - must be after all routes
app.use((req, res, next) => {
  const error = new Error('Not Found');
  error.statusCode = 404;
  error.code = 'NOT_FOUND';
  next(error);
});

// Global error handler - must be last middleware
app.use(errorHandler);

// ... rest of app.js ...
```

## Verification Steps

### 1. Test Input Validation

```bash
# Test email validation
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"notanemail","password":"password123"}'
# Expected: 400 "Invalid email format"

# Test password validation
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"short"}'
# Expected: 400 "Password must be at least 8 characters"

# Test task title validation
TOKEN="your-auth-token"
curl -X POST http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"","description":"Empty title"}'
# Expected: 400 "Title is required"

# Test query parameter validation
curl "http://localhost:3000/api/tasks?limit=200" \
  -H "Authorization: Bearer $TOKEN"
# Expected: 400 "Limit must be between 1 and 100"
```

### 2. Test Rate Limiting

```bash
# Test auth endpoint rate limiting (5 requests per hour)
for i in {1..6}; do
  curl -X POST http://localhost:3000/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"wrongpassword"}' &
done
# 6th request should return: 429 "Too many authentication attempts"

# Test general rate limiting (100 requests per 15 minutes)
for i in {1..101}; do
  curl http://localhost:3000/api/tasks \
    -H "Authorization: Bearer $TOKEN" &
done
# 101st request should return: 429 "Too many requests"
```

### 3. Test Error Handling

```bash
# Test 404 error
curl http://localhost:3000/nonexistent \
  -H "Authorization: Bearer $TOKEN"
# Expected: 404 "Not Found"

# Test authentication error
curl http://localhost:3000/api/tasks
# Expected: 401 "Access token required"

# Test authorization error (access another user's task)
curl http://localhost:3000/api/tasks/999 \
  -H "Authorization: Bearer $TOKEN"
# Expected: 403 "Access denied" or 404 "Task not found"
```

## Success Criteria

- All input is validated before processing
- Validation errors return consistent format with field information
- Rate limiting prevents abuse on sensitive endpoints
- Custom error classes provide consistent error handling
- Error responses follow standard format
- No sensitive information leaked in production
- All routes use async error handling
- Rate limits are appropriate for endpoint sensitivity

## Important Notes

- Validation happens before any business logic
- Rate limiting uses IP for anonymous users, user ID for authenticated
- Error messages are generic to avoid information leakage
- Stack traces only shown in development mode
- All async routes wrapped with asyncHandler
- Validation sanitizes input (trim, normalize email)

You have now successfully implemented comprehensive validation and error handling. The application is more secure, robust, and provides consistent, user-friendly error messages.