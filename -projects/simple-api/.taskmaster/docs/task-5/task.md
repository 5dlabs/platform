# Task 5: Implement API Routes

## Overview
This task creates the routing layer that connects HTTP endpoints to controller functions. Routes define the API structure, apply validation middleware, and ensure requests are properly handled according to REST conventions.

## Task Details

### Priority
Medium

### Dependencies
- Task 3: Implement Express Application and Middleware (must be completed)
- Task 4: Implement Todo Controller (must be completed)

### Status
Pending

## Implementation Guide

### 1. Create Todo Routes

**File: `src/routes/todos.js`**
```javascript
const express = require('express');
const { todoController } = require('../controllers');
const { todoValidation, handleValidationErrors } = require('../middleware');

const router = express.Router();

/**
 * @route   GET /api/todos
 * @desc    Get all todos with optional filtering and pagination
 * @query   {boolean} completed - Filter by completion status
 * @query   {number} limit - Maximum results (1-100, default 100)
 * @query   {number} offset - Skip results (default 0)
 * @access  Public
 */
router.get(
  '/',
  todoValidation.list,
  handleValidationErrors,
  todoController.getAllTodos
);

/**
 * @route   GET /api/todos/stats
 * @desc    Get todo statistics
 * @access  Public
 */
router.get(
  '/stats',
  todoController.getTodoStats
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
  handleValidationErrors,
  todoController.getTodoById
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
  handleValidationErrors,
  todoController.createTodo
);

/**
 * @route   PUT /api/todos/:id
 * @desc    Update an existing todo
 * @param   {number} id - Todo ID
 * @body    {string} title - New title (optional)
 * @body    {string} description - New description (optional)
 * @body    {boolean} completed - New completion status (optional)
 * @access  Public
 */
router.put(
  '/:id',
  todoValidation.update,
  handleValidationErrors,
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
  handleValidationErrors,
  todoController.deleteTodo
);

module.exports = router;
```

### 2. Create Health Routes

**File: `src/routes/health.js`**
```javascript
const express = require('express');
const { healthController } = require('../controllers');

const router = express.Router();

/**
 * @route   GET /api/health
 * @desc    Basic health check endpoint
 * @access  Public
 */
router.get('/', healthController.getHealth);

/**
 * @route   GET /api/health/detailed
 * @desc    Detailed health check with database status
 * @access  Public
 */
router.get('/detailed', healthController.getDetailedHealth);

module.exports = router;
```

### 3. Create Routes Index

**File: `src/routes/index.js`**
```javascript
const express = require('express');
const todoRoutes = require('./todos');
const healthRoutes = require('./health');

const router = express.Router();

// Mount routes
router.use('/todos', todoRoutes);
router.use('/health', healthRoutes);

// API root endpoint
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

module.exports = router;
```

### 4. Update App.js to Include Routes

**Update `src/app.js`** to register the routes:
```javascript
const express = require('express');
const path = require('path');
const { errorHandler, notFoundHandler } = require('./middleware');
const routes = require('./routes');

// Create Express application
const app = express();

// [Previous middleware configuration remains...]

// API Routes
app.use('/api', routes);

// Swagger documentation (placeholder for Task 6)
// app.use('/api-docs', swagger.serve, swagger.setup);

// 404 handler (must be after all routes)
app.use(notFoundHandler);

// Global error handler (must be last)
app.use(errorHandler);

module.exports = app;
```

### 5. Create Route Documentation

**File: `src/routes/README.md`**
```markdown
# API Routes Documentation

## Base URL
All API endpoints are prefixed with `/api`

## Available Endpoints

### Todo Endpoints
- `GET /api/todos` - List all todos
- `GET /api/todos/stats` - Get todo statistics
- `GET /api/todos/:id` - Get specific todo
- `POST /api/todos` - Create new todo
- `PUT /api/todos/:id` - Update todo
- `DELETE /api/todos/:id` - Delete todo

### System Endpoints
- `GET /api` - API information
- `GET /api/health` - Basic health check
- `GET /api/health/detailed` - Detailed health check

## Route Structure
```
/api
├── /todos
│   ├── GET /
│   ├── GET /stats
│   ├── GET /:id
│   ├── POST /
│   ├── PUT /:id
│   └── DELETE /:id
└── /health
    ├── GET /
    └── GET /detailed
```

## Middleware Order
1. Route-specific validation
2. Validation error handler
3. Controller function
4. Global error handler (if error occurs)
```

## Key Implementation Considerations

### Route Design Principles
- RESTful conventions for resource operations
- Consistent URL patterns and naming
- Proper HTTP methods for each operation
- Clear separation between routes and business logic

### Middleware Application
- Validation middleware applied before controllers
- Error handling middleware catches validation failures
- Route order matters (specific before generic)
- Stats route before /:id to avoid conflicts

### REST Conventions
- GET for reading resources
- POST for creating new resources
- PUT for full updates
- DELETE for removing resources
- Proper status codes from controllers

### Route Organization
- Separate files for different resource types
- Index file for mounting all routes
- Consistent route documentation
- Modular structure for easy expansion

## Testing Considerations

Routes should be tested for:
1. Correct controller methods are called
2. Validation middleware is applied
3. Route parameters are passed correctly
4. 404 returned for undefined routes
5. Middleware execution order

Example integration test:
```javascript
const request = require('supertest');
const app = require('../src/app');

describe('Todo Routes', () => {
  test('GET /api/todos returns todo list', async () => {
    const response = await request(app)
      .get('/api/todos')
      .expect(200);
    
    expect(Array.isArray(response.body)).toBe(true);
  });
  
  test('POST /api/todos validates required fields', async () => {
    const response = await request(app)
      .post('/api/todos')
      .send({}) // Missing title
      .expect(400);
    
    expect(response.body.error.code).toBe('VALIDATION_ERROR');
  });
});
```

## Common Issues and Solutions

### Issue: Route Conflicts
**Solution**: Order routes from most specific to least specific (e.g., /stats before /:id)

### Issue: Missing Validation
**Solution**: Always include validation middleware and error handler for user inputs

### Issue: Undefined Routes
**Solution**: Ensure all routes are exported and mounted correctly in app.js

## API Usage Examples

### Create a Todo
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Complete API implementation", "description": "Finish all endpoints"}'
```

### Get All Todos with Filtering
```bash
curl "http://localhost:3000/api/todos?completed=false&limit=10"
```

### Update a Todo
```bash
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'
```

### Delete a Todo
```bash
curl -X DELETE http://localhost:3000/api/todos/1
```

## Next Steps
After completing this task:
1. Test all endpoints with curl or Postman
2. Verify validation works correctly
3. Ensure proper error responses
4. Proceed to Task 6: API Documentation with Swagger
5. Then comprehensive testing in Task 7

## References
- [Express Router Documentation](https://expressjs.com/en/guide/routing.html)
- [REST API Design Best Practices](https://restfulapi.net/)
- [Architecture Document - Routes Layer](../architecture.md#routes-layer)