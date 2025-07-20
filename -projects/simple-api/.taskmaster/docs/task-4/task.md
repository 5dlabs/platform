# Task 4: Implement Todo Controller

## Overview
This task implements the controller layer that handles business logic for todo operations. The controller acts as an intermediary between the routes and the model, processing requests, handling errors, and formatting responses according to the API specifications.

## Task Details

### Priority
Medium

### Dependencies
- Task 2: Database Setup and Model Implementation (must be completed)
- Task 3: Implement Express Application and Middleware (must be completed)

### Status
Pending

## Implementation Guide

### 1. Create Todo Controller

**File: `src/controllers/todoController.js`**
```javascript
const { Todo } = require('../models');

/**
 * Todo Controller
 * Handles all todo-related business logic
 */
const todoController = {
  /**
   * Get all todos with optional filtering and pagination
   * @route GET /api/todos
   * @query {boolean} completed - Filter by completion status
   * @query {number} limit - Maximum number of results (1-100, default 100)
   * @query {number} offset - Number of results to skip (default 0)
   */
  getAllTodos(req, res, next) {
    try {
      // Extract and parse query parameters
      const { completed, limit = 100, offset = 0 } = req.query;
      
      // Build filter object
      const filters = {
        limit: parseInt(limit, 10) || 100,
        offset: parseInt(offset, 10) || 0
      };
      
      // Add completed filter if provided
      if (completed !== undefined) {
        filters.completed = completed === 'true' || completed === true;
      }
      
      // Fetch todos from database
      const todos = Todo.findAll(filters);
      
      // Return successful response
      res.status(200).json(todos);
    } catch (error) {
      // Pass errors to error handling middleware
      next(error);
    }
  },
  
  /**
   * Get a single todo by ID
   * @route GET /api/todos/:id
   * @param {number} id - Todo ID
   */
  getTodoById(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const todo = Todo.findById(id);
      
      if (!todo) {
        const error = new Error('Todo not found');
        error.status = 404;
        error.code = 'TODO_NOT_FOUND';
        return next(error);
      }
      
      res.status(200).json(todo);
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Create a new todo
   * @route POST /api/todos
   * @body {string} title - Todo title (required)
   * @body {string} description - Todo description (optional)
   */
  createTodo(req, res, next) {
    try {
      const { title, description } = req.body;
      
      // Create todo in database
      const todo = Todo.create({
        title,
        description: description || null
      });
      
      // Return created resource
      res.status(201).json(todo);
    } catch (error) {
      // Handle database constraint errors
      if (error.code === 'SQLITE_CONSTRAINT') {
        error.status = 400;
        error.message = 'Invalid todo data';
      }
      next(error);
    }
  },
  
  /**
   * Update an existing todo
   * @route PUT /api/todos/:id
   * @param {number} id - Todo ID
   * @body {string} title - New title (optional)
   * @body {string} description - New description (optional)
   * @body {boolean} completed - New completion status (optional)
   */
  updateTodo(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const updates = {};
      
      // Only include provided fields in update
      if (req.body.title !== undefined) {
        updates.title = req.body.title;
      }
      if (req.body.description !== undefined) {
        updates.description = req.body.description;
      }
      if (req.body.completed !== undefined) {
        updates.completed = req.body.completed;
      }
      
      // Perform update
      const todo = Todo.update(id, updates);
      
      if (!todo) {
        const error = new Error('Todo not found');
        error.status = 404;
        error.code = 'TODO_NOT_FOUND';
        return next(error);
      }
      
      res.status(200).json(todo);
    } catch (error) {
      if (error.code === 'SQLITE_CONSTRAINT') {
        error.status = 400;
        error.message = 'Invalid todo data';
      }
      next(error);
    }
  },
  
  /**
   * Delete a todo
   * @route DELETE /api/todos/:id
   * @param {number} id - Todo ID
   */
  deleteTodo(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const deleted = Todo.delete(id);
      
      if (!deleted) {
        const error = new Error('Todo not found');
        error.status = 404;
        error.code = 'TODO_NOT_FOUND';
        return next(error);
      }
      
      // Return 204 No Content on successful deletion
      res.status(204).end();
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Get todo statistics
   * @route GET /api/todos/stats
   */
  getTodoStats(req, res, next) {
    try {
      const total = Todo.count();
      const completed = Todo.count({ completed: true });
      const pending = total - completed;
      
      res.status(200).json({
        total,
        completed,
        pending,
        completionRate: total > 0 ? (completed / total) : 0
      });
    } catch (error) {
      next(error);
    }
  }
};

module.exports = todoController;
```

### 2. Create Health Check Controller

**File: `src/controllers/healthController.js`**
```javascript
const { db } = require('../models');

/**
 * Health Check Controller
 * Provides system health status
 */
const healthController = {
  /**
   * Basic health check
   * @route GET /api/health
   */
  getHealth(req, res) {
    res.status(200).json({
      status: 'ok',
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      environment: process.env.NODE_ENV || 'development'
    });
  },
  
  /**
   * Detailed health check including database
   * @route GET /api/health/detailed
   */
  getDetailedHealth(req, res, next) {
    try {
      // Check database connection
      const dbCheck = db.prepare('SELECT 1 as result').get();
      const dbStatus = dbCheck && dbCheck.result === 1 ? 'healthy' : 'unhealthy';
      
      res.status(200).json({
        status: dbStatus === 'healthy' ? 'ok' : 'degraded',
        timestamp: new Date().toISOString(),
        uptime: process.uptime(),
        environment: process.env.NODE_ENV || 'development',
        checks: {
          database: {
            status: dbStatus,
            message: dbStatus === 'healthy' ? 'Connected' : 'Connection failed'
          }
        }
      });
    } catch (error) {
      res.status(503).json({
        status: 'error',
        timestamp: new Date().toISOString(),
        checks: {
          database: {
            status: 'unhealthy',
            message: error.message
          }
        }
      });
    }
  }
};

module.exports = healthController;
```

### 3. Create Controller Index

**File: `src/controllers/index.js`**
```javascript
const todoController = require('./todoController');
const healthController = require('./healthController');

module.exports = {
  todoController,
  healthController
};
```

## Key Implementation Considerations

### Architecture Alignment
- Controllers handle business logic and response formatting
- Clear separation between controllers and models
- Error handling delegates to Express error middleware
- Consistent response formats across all endpoints

### Business Logic Implementation
- Input validation is handled by middleware (Task 3)
- Controllers focus on orchestrating model operations
- Proper HTTP status codes for each operation
- Clear error messages with appropriate error codes

### Error Handling Strategy
- Controllers use try-catch blocks consistently
- Errors are enhanced with status and code properties
- Database constraint errors are transformed to user-friendly messages
- All errors passed to next() for centralized handling

### Response Standards
- 200 OK for successful GET/PUT operations
- 201 Created for successful POST with created resource
- 204 No Content for successful DELETE
- 404 Not Found when resource doesn't exist
- 400 Bad Request for validation/constraint errors

## Testing Considerations

Controllers should be tested for:
1. Successful operations with valid data
2. Error handling for invalid IDs
3. Proper status codes for all scenarios
4. Response format consistency
5. Query parameter handling

Example test:
```javascript
// Mock the Todo model
jest.mock('../src/models', () => ({
  Todo: {
    findAll: jest.fn(),
    findById: jest.fn(),
    create: jest.fn(),
    update: jest.fn(),
    delete: jest.fn(),
    count: jest.fn()
  }
}));

const { todoController } = require('../src/controllers');
const { Todo } = require('../src/models');

describe('TodoController', () => {
  test('getAllTodos returns todos', () => {
    const mockTodos = [{ id: 1, title: 'Test' }];
    Todo.findAll.mockReturnValue(mockTodos);
    
    const req = { query: {} };
    const res = {
      status: jest.fn().mockReturnThis(),
      json: jest.fn()
    };
    const next = jest.fn();
    
    todoController.getAllTodos(req, res, next);
    
    expect(res.status).toHaveBeenCalledWith(200);
    expect(res.json).toHaveBeenCalledWith(mockTodos);
  });
});
```

## Common Issues and Solutions

### Issue: Parsing Query Parameters
**Solution**: Always parse numeric values from strings and provide defaults

### Issue: Inconsistent Error Responses
**Solution**: Always enhance errors with status and code before passing to next()

### Issue: Missing Resources
**Solution**: Check for null returns from model and return 404 appropriately

## Next Steps
After completing this task:
1. Test controllers in isolation with mock data
2. Proceed to Task 5: Implement API Routes
3. Routes will connect these controllers to HTTP endpoints
4. Integration testing will verify end-to-end functionality

## References
- [Express.js Error Handling](https://expressjs.com/en/guide/error-handling.html)
- [HTTP Status Codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
- [Architecture Document - Controller Layer](../architecture.md#controller-layer)