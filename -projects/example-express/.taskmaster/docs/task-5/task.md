# Task 5: Add Request Validation and Error Handling

## Overview

This task enhances the application's robustness by implementing comprehensive input validation across all endpoints and establishing standardized error handling. It includes setting up express-validator for declarative validation, implementing rate limiting to prevent abuse, and creating a centralized error handling system with custom error classes.

## Objectives

- Install and configure express-validator for input validation
- Create reusable validation middleware for common patterns
- Implement rate limiting on sensitive endpoints
- Build custom error classes for different error types
- Establish centralized error handling and logging
- Ensure consistent error response format across the API
- Protect against common attack vectors (SQL injection, XSS)

## Technical Requirements

### Dependencies
- **express-validator** (^7.0.1): Input validation and sanitization
- **express-rate-limit** (^7.2.0): Rate limiting middleware

### Validation Rules
- Email: Valid format, normalized to lowercase
- Password: Minimum 8 characters, optional complexity rules
- Task title: Required, 1-255 characters
- Task description: Optional, max 1000 characters
- IDs: Must be valid integers

### Rate Limiting
- General endpoints: 100 requests per 15 minutes
- Auth endpoints: 5 requests per hour
- Configurable per endpoint

## Implementation Steps

### 1. Install Dependencies (Subtask 5.1)

```bash
npm install express-validator@^7.0.1 express-rate-limit@^7.2.0
```

### 2. Create Validation Rules (Subtask 5.1)

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
    .isLength({ max: 255 }).withMessage('Email too long'),
  
  password: body('password')
    .notEmpty().withMessage('Password is required')
    .isLength({ min: 8 }).withMessage('Password must be at least 8 characters')
    .isLength({ max: 128 }).withMessage('Password too long'),
  
  // Task validations
  taskTitle: body('title')
    .trim()
    .notEmpty().withMessage('Title is required')
    .isLength({ min: 1, max: 255 }).withMessage('Title must be 1-255 characters'),
  
  taskDescription: body('description')
    .optional()
    .trim()
    .isLength({ max: 1000 }).withMessage('Description must be 1000 characters or less'),
  
  taskCompleted: body('completed')
    .optional()
    .isBoolean().withMessage('Completed must be a boolean'),
  
  // ID validations
  idParam: param('id')
    .isInt({ min: 1 }).withMessage('Invalid ID format'),
  
  // Query validations
  completedQuery: query('completed')
    .optional()
    .isIn(['true', 'false']).withMessage('Completed must be true or false'),
  
  limitQuery: query('limit')
    .optional()
    .isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1 and 100'),
  
  offsetQuery: query('offset')
    .optional()
    .isInt({ min: 0 }).withMessage('Offset must be non-negative')
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
    body('refreshToken')
      .notEmpty().withMessage('Refresh token is required'),
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
      .notEmpty().withMessage('Title cannot be empty')
      .isLength({ max: 255 }).withMessage('Title must be 255 characters or less'),
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
  handleValidationErrors
};
```

### 3. Create Custom Error Classes (Subtask 5.3)

Create `src/utils/errors.js`:
```javascript
// Base error class
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
  constructor(message, field) {
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
  constructor(message = 'Too many requests') {
    super(message, 429, 'RATE_LIMIT_ERROR');
  }
}

// Error handler middleware
const errorHandler = (err, req, res, next) => {
  let error = err;
  
  // Log error
  console.error('Error:', {
    message: err.message,
    stack: err.stack,
    statusCode: err.statusCode,
    code: err.code,
    path: req.path,
    method: req.method,
    ip: req.ip,
    user: req.user?.id
  });
  
  // Default error
  if (!error.isOperational) {
    error = new AppError(
      process.env.NODE_ENV === 'production' 
        ? 'Internal server error' 
        : err.message,
      500,
      'INTERNAL_ERROR'
    );
  }
  
  // Send error response
  res.status(error.statusCode).json({
    error: {
      message: error.message,
      code: error.code,
      ...(error.field && { field: error.field }),
      ...(process.env.NODE_ENV === 'development' && { stack: err.stack })
    }
  });
};

// Async error wrapper
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

### 4. Implement Rate Limiting (Subtask 5.4)

Create `src/middleware/rateLimiter.js`:
```javascript
const rateLimit = require('express-rate-limit');
const { RateLimitError } = require('../utils/errors');

// Rate limit configurations
const createRateLimiter = (options) => {
  return rateLimit({
    windowMs: options.windowMs,
    max: options.max,
    message: options.message,
    standardHeaders: true,
    legacyHeaders: false,
    handler: (req, res) => {
      throw new RateLimitError(options.message);
    },
    skip: (req) => {
      // Skip rate limiting in test environment
      return process.env.NODE_ENV === 'test';
    }
  });
};

// Different limiters for different endpoints
const limiters = {
  // General API limiter
  general: createRateLimiter({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100,
    message: 'Too many requests, please try again later'
  }),
  
  // Strict limiter for auth endpoints
  auth: createRateLimiter({
    windowMs: 60 * 60 * 1000, // 1 hour
    max: 5,
    message: 'Too many authentication attempts, please try again later'
  }),
  
  // Medium limiter for creation endpoints
  create: createRateLimiter({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 30,
    message: 'Too many creation requests, please slow down'
  })
};

module.exports = limiters;
```

### 5. Update Authentication Routes with Validation

Update `src/routes/auth.js`:
```javascript
const { validate } = require('../middleware/validation');
const limiters = require('../middleware/rateLimiter');
const { ConflictError, AuthenticationError, asyncHandler } = require('../utils/errors');

// Apply rate limiting to auth routes
router.use(limiters.auth);

// Update register endpoint
router.post('/register', 
  validate.register,
  asyncHandler(async (req, res) => {
    const { email, password } = req.body;
    
    // Check if user exists
    const existingUser = User.findByEmail(email);
    if (existingUser) {
      throw new ConflictError('Email already registered');
    }
    
    // Create user
    const hashedPassword = await hashPassword(password);
    const user = User.create(email, hashedPassword);
    
    // Generate tokens
    const tokenPayload = { userId: user.id, email: user.email };
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    res.status(201).json({
      message: 'User registered successfully',
      user: { id: user.id, email: user.email },
      tokens: { accessToken, refreshToken, expiresIn: '24h' }
    });
  })
);

// Update login endpoint
router.post('/login',
  validate.login,
  asyncHandler(async (req, res) => {
    const { email, password } = req.body;
    
    // Find user
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
    const tokenPayload = { userId: user.id, email: user.email };
    const accessToken = generateToken(tokenPayload);
    const refreshToken = generateRefreshToken(tokenPayload);
    
    res.json({
      message: 'Login successful',
      user: { id: user.id, email: user.email },
      tokens: { accessToken, refreshToken, expiresIn: '24h' }
    });
  })
);
```

### 6. Update Task Routes with Validation

Update `src/routes/tasks.js`:
```javascript
const { validate } = require('../middleware/validation');
const limiters = require('../middleware/rateLimiter');
const { NotFoundError, AuthorizationError, asyncHandler } = require('../utils/errors');

// Apply general rate limiting
router.use(limiters.general);

// Update routes with validation
router.get('/', 
  validate.listTasks,
  asyncHandler(async (req, res) => {
    const { completed, limit = 20, offset = 0 } = req.query;
    
    const filters = {
      ...(completed !== undefined && { completed: completed === 'true' }),
      limit: parseInt(limit),
      offset: parseInt(offset)
    };
    
    const tasks = Task.findByUserId(req.user.id, filters);
    const totalCount = Task.countByUserId(req.user.id, { completed: filters.completed });
    
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
        hasNext: filters.offset + tasks.length < totalCount,
        hasPrev: filters.offset > 0
      }
    });
  })
);

router.post('/',
  limiters.create,
  validate.createTask,
  asyncHandler(async (req, res) => {
    const { title, description } = req.body;
    
    const task = Task.create(req.user.id, title, description);
    
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

router.get('/:id',
  validate.getTask,
  asyncHandler(async (req, res) => {
    const taskId = parseInt(req.params.id);
    const task = Task.findById(taskId);
    
    if (!task) {
      throw new NotFoundError('Task');
    }
    
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

router.put('/:id',
  validate.updateTask,
  asyncHandler(async (req, res) => {
    const taskId = parseInt(req.params.id);
    const updates = {};
    
    // Collect valid updates
    if (req.body.title !== undefined) updates.title = req.body.title;
    if (req.body.description !== undefined) updates.description = req.body.description;
    if (req.body.completed !== undefined) updates.completed = req.body.completed;
    
    if (Object.keys(updates).length === 0) {
      throw new ValidationError('No valid fields to update');
    }
    
    const success = Task.update(taskId, req.user.id, updates);
    if (!success) {
      throw new NotFoundError('Task');
    }
    
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

router.delete('/:id',
  validate.deleteTask,
  asyncHandler(async (req, res) => {
    const taskId = parseInt(req.params.id);
    
    const success = Task.delete(taskId, req.user.id);
    if (!success) {
      throw new NotFoundError('Task');
    }
    
    res.status(204).send();
  })
);
```

### 7. Update Main App with Error Handler

Update `src/app.js`:
```javascript
const { errorHandler } = require('./utils/errors');

// ... existing middleware and routes ...

// Add error handler as the last middleware
app.use(errorHandler);
```

## Testing

### Test Validation
```bash
# Test email validation
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"invalid-email","password":"password123"}'

# Test password validation
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"short"}'

# Test task title validation
curl -X POST http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"","description":"Empty title"}'
```

### Test Rate Limiting
```bash
# Make multiple requests quickly
for i in {1..6}; do
  curl -X POST http://localhost:3000/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"wrong"}' &
done
```

## Common Issues and Solutions

### Issue: Validation not triggering
**Solution**: Ensure validation middleware is before route handler

### Issue: Rate limiting not working
**Solution**: Check Redis connection if using distributed rate limiting

### Issue: Error details leaked in production
**Solution**: Use NODE_ENV to control error detail exposure

## Next Steps

After completing this task:
- All inputs are validated before processing
- Rate limiting protects against abuse
- Errors are handled consistently
- Application is more secure and robust
- Ready for Task 6: Create Basic Frontend Interface

The validation and error handling layer ensures the API is production-ready with proper security measures and user-friendly error messages.