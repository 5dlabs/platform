# Autonomous Agent Prompt for Task 12: Research Express.js Best Practices

## Task Context
You need to research current Express.js best practices and create comprehensive documentation that can guide future improvements to the Hello World API.

## Your Mission
Research Express.js best practices in four key areas (error handling, middleware organization, request logging, and security) and compile your findings into a well-structured documentation file.

## Step-by-Step Instructions

### 1. Create Documentation Directory
```bash
cd hello-world-api
mkdir -p docs
```

### 2. Research and Document Best Practices
Create `docs/best-practices.md` with comprehensive research findings:

```markdown
# Express.js Best Practices Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Middleware Organization](#middleware-organization)
4. [Request Logging](#request-logging)
5. [Security Best Practices](#security-best-practices)
6. [Performance Optimization](#performance-optimization)
7. [References](#references)

## Introduction
This document outlines current best practices for Express.js applications based on official documentation, community standards, and industry recommendations as of 2024.

## Error Handling Patterns

### 1. Centralized Error Handling
```javascript
// middleware/errorHandler.js
class AppError extends Error {
  constructor(message, statusCode) {
    super(message);
    this.statusCode = statusCode;
    this.isOperational = true;
    Error.captureStackTrace(this, this.constructor);
  }
}

const errorHandler = (err, req, res, next) => {
  let { statusCode = 500, message } = err;
  
  // Log error
  logger.error({
    error: err,
    request: req.url,
    method: req.method,
    ip: req.ip
  });
  
  res.status(statusCode).json({
    status: 'error',
    statusCode,
    message: process.env.NODE_ENV === 'production' 
      ? 'Something went wrong!' 
      : message
  });
};
```

### 2. Async Error Handling
```javascript
// Use express-async-errors
require('express-async-errors');

// Or create async wrapper
const asyncHandler = (fn) => (req, res, next) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

// Usage
app.get('/users/:id', asyncHandler(async (req, res) => {
  const user = await User.findById(req.params.id);
  if (!user) throw new AppError('User not found', 404);
  res.json(user);
}));
```

### 3. Error Types
- Operational errors (predictable, should be handled)
- Programming errors (bugs, should crash gracefully)
- Third-party errors (external service failures)

## Middleware Organization

### 1. Recommended Order
```javascript
const app = express();

// 1. Security middleware
app.use(helmet());
app.use(cors(corsOptions));

// 2. Request parsing
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true }));

// 3. Request logging
app.use(morgan('combined'));

// 4. Session handling (if needed)
app.use(session(sessionConfig));

// 5. Rate limiting
app.use('/api', rateLimiter);

// 6. Routes
app.use('/api/v1', routes);

// 7. Error handling (last)
app.use(errorHandler);
app.use(notFoundHandler);
```

### 2. Middleware Composition
```javascript
// Compose multiple middleware
const authenticate = compose([
  validateToken,
  loadUser,
  checkPermissions
]);

app.get('/admin', authenticate, adminHandler);
```

## Request Logging

### 1. Morgan Configuration
```javascript
// Development
app.use(morgan('dev'));

// Production with custom tokens
morgan.token('user-id', (req) => req.user?.id || 'anonymous');
morgan.token('response-time-ms', (req, res) => {
  return Date.now() - req._startTime;
});

const format = ':user-id :method :url :status :response-time-ms ms';
app.use(morgan(format, { stream: logger.stream }));
```

### 2. Structured Logging with Winston
```javascript
const winston = require('winston');

const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'hello-world-api' },
  transports: [
    new winston.transports.File({ filename: 'error.log', level: 'error' }),
    new winston.transports.File({ filename: 'combined.log' })
  ]
});

// Add console transport in development
if (process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.simple()
  }));
}
```

### 3. Request Correlation
```javascript
const { v4: uuidv4 } = require('uuid');

app.use((req, res, next) => {
  req.id = req.headers['x-request-id'] || uuidv4();
  res.setHeader('X-Request-ID', req.id);
  next();
});
```

## Security Best Practices

### 1. Helmet.js Configuration
```javascript
const helmet = require('helmet');

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
    },
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true
  }
}));
```

### 2. Rate Limiting
```javascript
const rateLimit = require('express-rate-limit');

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP',
  standardHeaders: true,
  legacyHeaders: false,
});

// Apply to all requests
app.use('/api', limiter);

// Stricter limits for auth endpoints
const authLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 5,
  skipSuccessfulRequests: true,
});

app.use('/api/auth', authLimiter);
```

### 3. Input Validation
```javascript
const { body, validationResult } = require('express-validator');

const validateUser = [
  body('email').isEmail().normalizeEmail(),
  body('password').isLength({ min: 8 }).trim(),
  body('name').notEmpty().trim().escape(),
  
  (req, res, next) => {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ errors: errors.array() });
    }
    next();
  }
];

app.post('/users', validateUser, createUser);
```

### 4. Security Headers
```javascript
// Additional security headers
app.use((req, res, next) => {
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('X-XSS-Protection', '1; mode=block');
  res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
  res.removeHeader('X-Powered-By');
  next();
});
```

## Performance Optimization

### 1. Compression
```javascript
const compression = require('compression');
app.use(compression());
```

### 2. Caching
```javascript
// Static files
app.use(express.static('public', {
  maxAge: '1d',
  etag: true
}));

// API responses
app.get('/api/data', (req, res) => {
  res.set('Cache-Control', 'public, max-age=300'); // 5 minutes
  res.json(data);
});
```

### 3. Database Connection Pooling
```javascript
// Example with PostgreSQL
const { Pool } = require('pg');
const pool = new Pool({
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
});
```

## References
- [Express.js Official Documentation](https://expressjs.com/)
- [Express.js Security Best Practices](https://expressjs.com/en/advanced/best-practice-security.html)
- [Node.js Best Practices](https://github.com/goldbergyoni/nodebestpractices)
- [OWASP Security Guidelines](https://owasp.org/www-project-top-ten/)
- [Express.js Performance Best Practices](https://expressjs.com/en/advanced/best-practice-performance.html)
```

### 3. Add Implementation Examples
Include practical examples that can be applied to the current project:

```javascript
// Example: Applying best practices to hello-world-api

// 1. Enhanced error handling
const createError = require('http-errors');

app.get('/', (req, res, next) => {
  try {
    res.json({ message: 'Hello, World!' });
  } catch (error) {
    next(createError(500, 'Failed to process request'));
  }
});

// 2. Environment-based configuration
const config = {
  development: {
    morgan: 'dev',
    errorStack: true
  },
  production: {
    morgan: 'combined',
    errorStack: false
  }
};

const env = process.env.NODE_ENV || 'development';
const currentConfig = config[env];
```

## Validation Steps

### 1. Verify Documentation Creation
```bash
ls -la docs/best-practices.md
# File should exist
```

### 2. Check Documentation Structure
```bash
# Check for all required sections
grep -E "^##" docs/best-practices.md
# Should show all main sections
```

### 3. Validate Code Examples
Ensure all code examples in the documentation are syntactically correct and relevant.

## Expected Result
- Comprehensive best practices document created
- All four research areas covered
- Practical code examples included
- References to authoritative sources
- Actionable recommendations for the project

## Important Notes
- Focus on current practices (2024)
- Include code examples that work with Express 4.x
- Consider both development and production scenarios
- Emphasize security and performance
- Make recommendations specific to the Hello World API project