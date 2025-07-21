# Task 5: Implement API Routes

## Overview

This task connects the controllers to HTTP endpoints by implementing Express routes. Routes define the API's URL structure, apply validation middleware, and delegate request handling to the appropriate controllers.

## Context

Building upon the validation middleware from Task 3 and controllers from Task 4, this task creates the RESTful API endpoints as specified in the [architecture document](../architecture.md). Routes act as the glue between HTTP requests and business logic, ensuring requests are validated before reaching controllers.

## Implementation Guide

### 1. Create Todo Routes (src/routes/todoRoutes.js)

Implement all CRUD routes with validation:

```javascript
const express = require('express');
const todoController = require('../controllers/todoController');
const { todoValidation } = require('../middleware/validation');

const router = express.Router();

/**
 * @route   GET /api/todos
 * @desc    Get all todos with optional filters
 * @query   {boolean} completed - Filter by completion status
 * @query   {number} limit - Maximum number of results (1-100)
 * @query   {number} offset - Number of results to skip
 * @access  Public
 */
router.get(
  '/', 
  todoValidation.list, 
  todoController.getAllTodos
);

/**
 * @route   POST /api/todos
 * @desc    Create a new todo
 * @body    {string} title - Todo title (required)
 * @body    {string} description - Todo description (optional)
 * @access  Public
 */
router.post(
  '/', 
  todoValidation.create, 
  todoController.createTodo
);

/**
 * @route   GET /api/todos/:id
 * @desc    Get a specific todo by ID
 * @param   {number} id - Todo ID
 * @access  Public
 */
router.get(
  '/:id', 
  todoValidation.getOne, 
  todoController.getTodoById
);

/**
 * @route   PUT /api/todos/:id
 * @desc    Update a todo
 * @param   {number} id - Todo ID
 * @body    {string} title - New title (optional)
 * @body    {string} description - New description (optional)
 * @body    {boolean} completed - Completion status (optional)
 * @access  Public
 */
router.put(
  '/:id', 
  todoValidation.update, 
  todoController.updateTodo
);

/**
 * @route   DELETE /api/todos/:id
 * @desc    Delete a todo
 * @param   {number} id - Todo ID
 * @access  Public
 */
router.delete(
  '/:id', 
  todoValidation.delete, 
  todoController.deleteTodo
);

/**
 * @route   GET /api/todos/stats
 * @desc    Get todo statistics
 * @access  Public
 */
router.get(
  '/stats/summary',
  todoController.getTodoStats
);

module.exports = router;
```

### 2. Create Health Routes (src/routes/healthRoutes.js)

Implement health check endpoint:

```javascript
const express = require('express');
const healthController = require('../controllers/healthController');

const router = express.Router();

/**
 * @route   GET /api/health
 * @desc    Health check endpoint
 * @access  Public
 * @returns {Object} Service health status
 */
router.get('/', healthController.checkHealth);

/**
 * @route   GET /api/health/ready
 * @desc    Readiness check for load balancers
 * @access  Public
 */
router.get('/ready', (req, res) => {
  // Check if all services are ready
  const isReady = true; // Add actual checks here
  
  if (isReady) {
    res.json({ ready: true });
  } else {
    res.status(503).json({ ready: false });
  }
});

/**
 * @route   GET /api/health/live
 * @desc    Liveness check for container orchestration
 * @access  Public
 */
router.get('/live', (req, res) => {
  res.json({ alive: true });
});

module.exports = router;
```

### 3. Create Root Router (src/routes/index.js)

Create a main router to organize all routes:

```javascript
const express = require('express');
const todoRoutes = require('./todoRoutes');
const healthRoutes = require('./healthRoutes');

const router = express.Router();

// API version and info
router.get('/', (req, res) => {
  res.json({
    message: 'Simple Todo REST API',
    version: '1.0.0',
    endpoints: {
      todos: '/api/todos',
      health: '/api/health',
      documentation: '/api-docs'
    }
  });
});

// Mount route modules
router.use('/todos', todoRoutes);
router.use('/health', healthRoutes);

module.exports = router;
```

### 4. Update Express App (src/app.js)

Integrate routes into the Express application:

```javascript
const express = require('express');
const routes = require('./routes');
const { requestId, responseTime } = require('./middleware/common');

// Initialize Express app
const app = express();

// Body parsing middleware
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Common middleware
app.use(requestId);
app.use(responseTime);

// Request logging in development
if (process.env.NODE_ENV === 'development') {
  app.use((req, res, next) => {
    console.log(`[${req.id}] ${req.method} ${req.path}`);
    next();
  });
}

// API Routes
app.use('/api', routes);

// Root endpoint
app.get('/', (req, res) => {
  res.json({
    message: 'Welcome to Simple Todo API',
    documentation: `${req.protocol}://${req.get('host')}/api-docs`,
    api: `${req.protocol}://${req.get('host')}/api`
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({ 
    error: 'Not Found',
    message: `Cannot ${req.method} ${req.path}`,
    requestId: req.id
  });
});

// Global error handling middleware
app.use((err, req, res, next) => {
  console.error(`[${req.id}] Error:`, err.stack);
  
  // Handle validation errors from express-validator
  if (err.name === 'ValidationError' || err.type === 'entity.parse.failed') {
    return res.status(400).json({
      error: 'Validation Error',
      message: err.message,
      details: err.details || [],
      requestId: req.id
    });
  }
  
  // Handle custom errors with status codes
  if (err.status) {
    return res.status(err.status).json({
      error: err.name || 'Error',
      message: err.message,
      requestId: req.id
    });
  }
  
  // Default error response
  res.status(500).json({
    error: 'Internal Server Error',
    message: process.env.NODE_ENV === 'production' 
      ? 'An error occurred processing your request' 
      : err.message,
    requestId: req.id
  });
});

module.exports = app;
```

### 5. Route Testing with cURL

Create a test script for manual route testing:

```bash
#!/bin/bash
# test-routes.sh

BASE_URL="http://localhost:3000/api"

echo "Testing Todo API Routes"
echo "======================"

# Test GET all todos
echo -e "\n1. GET /api/todos"
curl -X GET "$BASE_URL/todos"

# Test POST create todo
echo -e "\n\n2. POST /api/todos"
curl -X POST "$BASE_URL/todos" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Todo","description":"Created via API"}'

# Test GET single todo
echo -e "\n\n3. GET /api/todos/1"
curl -X GET "$BASE_URL/todos/1"

# Test PUT update todo
echo -e "\n\n4. PUT /api/todos/1"
curl -X PUT "$BASE_URL/todos/1" \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Test DELETE todo
echo -e "\n\n5. DELETE /api/todos/1"
curl -X DELETE "$BASE_URL/todos/1"

# Test health check
echo -e "\n\n6. GET /api/health"
curl -X GET "$BASE_URL/health"

# Test with filters
echo -e "\n\n7. GET /api/todos?completed=true&limit=5"
curl -X GET "$BASE_URL/todos?completed=true&limit=5"

# Test validation error
echo -e "\n\n8. POST /api/todos (validation error)"
curl -X POST "$BASE_URL/todos" \
  -H "Content-Type: application/json" \
  -d '{"description":"Missing title"}'

echo -e "\n\nTests complete!"
```

## Dependencies and Relationships

- **Depends on**: 
  - Task 3 (Express Application and Middleware) - Uses validation
  - Task 4 (Implement Todo Controller) - Uses controllers
- **Required by**: 
  - Task 6 (API Documentation) - Documents these routes
  - Task 7 (Comprehensive Tests) - Tests these routes

## Success Criteria

1. ✅ All CRUD routes implemented and functional
2. ✅ Routes use appropriate HTTP methods (GET, POST, PUT, DELETE)
3. ✅ Validation middleware applied to all routes
4. ✅ Controllers properly connected to routes
5. ✅ Health check endpoints functional
6. ✅ Routes mounted at correct paths (/api/todos, /api/health)
7. ✅ 404 errors for undefined routes
8. ✅ Request IDs included in responses
9. ✅ Proper route parameter handling

## Testing

Manual testing with the provided script:

```bash
chmod +x test-routes.sh
./test-routes.sh
```

Or individual route testing:

```bash
# Start server
npm run dev

# Create a todo
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"My First Todo"}'

# List todos
curl http://localhost:3000/api/todos

# Get specific todo
curl http://localhost:3000/api/todos/1

# Update todo
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Delete todo
curl -X DELETE http://localhost:3000/api/todos/1
```

## Route Organization

```
/                       # API welcome message
/api                    # API root
/api/todos              # Todo resources
  GET    /              # List todos
  POST   /              # Create todo
  GET    /:id           # Get single todo
  PUT    /:id           # Update todo
  DELETE /:id           # Delete todo
  GET    /stats/summary # Todo statistics
/api/health             # Health checks
  GET    /              # Main health check
  GET    /ready         # Readiness probe
  GET    /live          # Liveness probe
/api-docs               # Swagger documentation (Task 6)
```

## Common Issues and Solutions

1. **Route order matters**: Place specific routes before generic ones
2. **Validation errors**: Ensure validation middleware is before controller
3. **404 on valid routes**: Check route mounting and paths
4. **CORS issues**: Add CORS middleware if needed for browser access

## Security Considerations

- All inputs validated before reaching controllers
- SQL injection prevented by parameterized queries in model
- No sensitive data exposed in error messages
- Request IDs help with debugging and log correlation

## Next Steps

After completing this task:
- Task 6: Add Swagger documentation for all routes
- Task 7: Write integration tests for all endpoints

The API is now fully functional with all CRUD operations available via RESTful endpoints.