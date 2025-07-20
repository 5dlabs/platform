# Task 5: Implement API Routes

## Overview
Create route handlers for all API endpoints, connecting the Express application to the controllers with proper validation middleware. This task establishes the routes layer that defines the API's public interface.

## Task Details
**ID**: 5  
**Title**: Implement API Routes  
**Priority**: Medium  
**Dependencies**: 
- [Task 3: Implement Express Application and Middleware](../task-3/task.md)
- [Task 4: Implement Todo Controller](../task-4/task.md)  
**Status**: Pending

## Architecture Context
This task implements the Routes Layer as defined in the [architecture document](../../architecture.md):
- API route definitions and URL patterns
- Request routing to appropriate controllers
- Integration of validation middleware
- Route-level error handling
- RESTful endpoint structure

Key architectural patterns:
- RESTful resource naming conventions
- Middleware composition for validation
- Route organization and modularity
- Clear separation between routing and business logic

## Product Requirements Alignment
Implements all API endpoints specified in the PRD:
- GET /api/todos - List all todos with filtering
- POST /api/todos - Create new todo
- GET /api/todos/:id - Get specific todo
- PUT /api/todos/:id - Update existing todo
- DELETE /api/todos/:id - Delete todo
- GET /api/health - Health check endpoint

## Implementation Steps

### 1. Create Todo Routes
Create `src/routes/todoRoutes.js`:
```javascript
const express = require('express');
const router = express.Router();
const { todoController } = require('../controllers');
const todoValidation = require('../middleware/validation');
const { asyncHandler } = require('../middleware/errorHandler');

/**
 * @route   GET /api/todos
 * @desc    Get all todos with optional filtering
 * @query   completed (boolean), limit (number), offset (number)
 * @access  Public
 */
router.get(
  '/',
  todoValidation.list,
  asyncHandler(todoController.getAllTodos)
);

/**
 * @route   POST /api/todos
 * @desc    Create a new todo
 * @body    { title: string, description?: string }
 * @access  Public
 */
router.post(
  '/',
  todoValidation.create,
  asyncHandler(todoController.createTodo)
);

/**
 * @route   GET /api/todos/stats
 * @desc    Get todo statistics
 * @access  Public
 */
router.get(
  '/stats',
  asyncHandler(todoController.getTodoStats)
);

/**
 * @route   GET /api/todos/:id
 * @desc    Get a specific todo by ID
 * @param   id - Todo ID
 * @access  Public
 */
router.get(
  '/:id',
  todoValidation.getOne,
  asyncHandler(todoController.getTodoById)
);

/**
 * @route   PUT /api/todos/:id
 * @desc    Update an existing todo
 * @param   id - Todo ID
 * @body    { title?: string, description?: string, completed?: boolean }
 * @access  Public
 */
router.put(
  '/:id',
  todoValidation.update,
  asyncHandler(todoController.updateTodo)
);

/**
 * @route   DELETE /api/todos/:id
 * @desc    Delete a todo
 * @param   id - Todo ID
 * @access  Public
 */
router.delete(
  '/:id',
  todoValidation.delete,
  asyncHandler(todoController.deleteTodo)
);

module.exports = router;
```

### 2. Create Health Routes
Create `src/routes/healthRoutes.js`:
```javascript
const express = require('express');
const router = express.Router();
const db = require('../models/db');

/**
 * @route   GET /api/health
 * @desc    Health check endpoint
 * @access  Public
 */
router.get('/', (req, res) => {
  try {
    // Check database connectivity
    const dbCheck = db.prepare('SELECT 1').get();
    const dbStatus = dbCheck ? 'connected' : 'disconnected';
    
    res.json({
      status: 'ok',
      timestamp: new Date().toISOString(),
      environment: process.env.NODE_ENV || 'development',
      version: process.env.npm_package_version || '1.0.0',
      uptime: process.uptime(),
      database: dbStatus
    });
  } catch (error) {
    res.status(503).json({
      status: 'error',
      timestamp: new Date().toISOString(),
      environment: process.env.NODE_ENV || 'development',
      error: 'Service temporarily unavailable'
    });
  }
});

/**
 * @route   GET /api/health/detailed
 * @desc    Detailed health check with system info
 * @access  Public
 */
router.get('/detailed', (req, res) => {
  try {
    const memoryUsage = process.memoryUsage();
    const todoCount = require('../models/todo').count();
    
    res.json({
      status: 'ok',
      timestamp: new Date().toISOString(),
      system: {
        nodeVersion: process.version,
        platform: process.platform,
        uptime: process.uptime(),
        memory: {
          rss: `${Math.round(memoryUsage.rss / 1024 / 1024)} MB`,
          heapTotal: `${Math.round(memoryUsage.heapTotal / 1024 / 1024)} MB`,
          heapUsed: `${Math.round(memoryUsage.heapUsed / 1024 / 1024)} MB`
        }
      },
      application: {
        environment: process.env.NODE_ENV || 'development',
        version: process.env.npm_package_version || '1.0.0',
        todoCount
      }
    });
  } catch (error) {
    res.status(503).json({
      status: 'error',
      timestamp: new Date().toISOString(),
      error: 'Service temporarily unavailable'
    });
  }
});

module.exports = router;
```

### 3. Create Main Routes Index
Create `src/routes/index.js`:
```javascript
const express = require('express');
const router = express.Router();
const todoRoutes = require('./todoRoutes');
const healthRoutes = require('./healthRoutes');

// Mount sub-routers
router.use('/todos', todoRoutes);
router.use('/health', healthRoutes);

// API root endpoint
router.get('/', (req, res) => {
  res.json({
    message: 'Simple Todo REST API',
    version: '1.0.0',
    endpoints: {
      todos: {
        list: 'GET /api/todos',
        create: 'POST /api/todos',
        get: 'GET /api/todos/:id',
        update: 'PUT /api/todos/:id',
        delete: 'DELETE /api/todos/:id',
        stats: 'GET /api/todos/stats'
      },
      health: {
        basic: 'GET /api/health',
        detailed: 'GET /api/health/detailed'
      },
      documentation: 'GET /api-docs'
    }
  });
});

module.exports = router;
```

### 4. Update Application to Include Routes
Update `src/app.js` to mount the routes:
```javascript
const express = require('express');
const routes = require('./routes');
const swagger = require('./middleware/swagger'); // If implementing Task 6

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

// CORS headers
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization');
  if (req.method === 'OPTIONS') {
    return res.sendStatus(200);
  }
  next();
});

// API Documentation (if swagger is implemented)
// app.use('/api-docs', swagger.serve, swagger.setup);

// Mount API routes
app.use('/api', routes);

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    error: {
      message: 'Resource not found',
      code: 'NOT_FOUND',
      path: req.path
    }
  });
});

// Global error handler
app.use((err, req, res, next) => {
  console.error('Error:', err);
  
  // Handle validation errors from express-validator
  if (err.type === 'validation') {
    return res.status(400).json({
      error: {
        message: 'Validation failed',
        code: 'VALIDATION_ERROR',
        details: err.errors
      }
    });
  }
  
  // Handle custom application errors
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

## Route Structure

### RESTful Endpoints
```
GET    /api/todos          - List todos
POST   /api/todos          - Create todo
GET    /api/todos/stats    - Get statistics
GET    /api/todos/:id      - Get single todo
PUT    /api/todos/:id      - Update todo
DELETE /api/todos/:id      - Delete todo
```

### System Endpoints
```
GET    /api                - API information
GET    /api/health         - Basic health check
GET    /api/health/detailed - Detailed health info
```

## Success Criteria
- All API endpoints are accessible and functional
- Routes properly delegate to controllers
- Validation middleware is applied correctly
- Error handling works at the route level
- Async errors are properly caught
- Route organization is clean and modular
- Health endpoints provide accurate information

## Testing Considerations
- Integration tests for all endpoints
- Test route parameter handling
- Verify validation middleware integration
- Test error responses for invalid routes
- Ensure proper HTTP methods are enforced
- Test CORS functionality

## Related Tasks
- **Dependencies**: 
  - [Task 3: Express Application and Middleware](../task-3/task.md) - Provides app structure
  - [Task 4: Implement Todo Controller](../task-4/task.md) - Provides controller functions
- **Next**: [Task 6: Implement API Documentation](../task-6/task.md) - Will document these routes
- **Related**: [Task 7: Write Comprehensive Tests](../task-7/task.md) - Will test these routes

## References
- [Architecture Document](../../architecture.md) - Sections: Routes Layer, API Endpoints
- [Product Requirements](../../prd.txt) - Section: API Endpoints