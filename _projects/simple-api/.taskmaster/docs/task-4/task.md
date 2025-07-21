# Task 4: Implement Todo Controller

## Overview

This task implements the controller layer of the MVC pattern, creating functions that handle the business logic for todo CRUD operations. Controllers bridge the gap between the data models and the HTTP routes, processing requests and formatting responses.

## Context

Building upon the Todo model from Task 2 and the Express infrastructure from Task 3, this task creates controller functions that will be used by the routes in Task 5. Controllers follow the single responsibility principle, handling one specific operation each while maintaining consistent error handling and response formats.

## Implementation Guide

### 1. Create Todo Controller (src/controllers/todoController.js)

Implement all CRUD operation handlers:

```javascript
const Todo = require('../models/todo');

const todoController = {
  /**
   * Get all todos with optional filtering and pagination
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  getAllTodos(req, res, next) {
    try {
      const { completed, limit, offset } = req.query;
      
      // Build filter object with type conversion
      const filters = {};
      
      // Convert string 'true'/'false' to boolean
      if (completed !== undefined) {
        filters.completed = completed === 'true';
      }
      
      // Parse pagination parameters
      if (limit) {
        filters.limit = parseInt(limit, 10);
      }
      
      if (offset) {
        filters.offset = parseInt(offset, 10);
      }
      
      // Fetch todos from database
      const todos = Todo.findAll(filters);
      
      // Return successful response
      res.json({
        data: todos,
        count: todos.length,
        filters: {
          completed: filters.completed,
          limit: filters.limit || null,
          offset: filters.offset || null
        }
      });
    } catch (err) {
      // Pass errors to error handling middleware
      next(err);
    }
  },
  
  /**
   * Get a single todo by ID
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  getTodoById(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const todo = Todo.findById(id);
      
      if (!todo) {
        const error = new Error('Todo not found');
        error.status = 404;
        return next(error);
      }
      
      res.json({
        data: todo
      });
    } catch (err) {
      next(err);
    }
  },
  
  /**
   * Create a new todo
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  createTodo(req, res, next) {
    try {
      const { title, description } = req.body;
      
      // Create todo with validated data
      const todo = Todo.create({ 
        title: title.trim(), 
        description: description ? description.trim() : null 
      });
      
      // Return created resource with 201 status
      res.status(201).json({
        message: 'Todo created successfully',
        data: todo
      });
    } catch (err) {
      next(err);
    }
  },
  
  /**
   * Update an existing todo
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  updateTodo(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const { title, description, completed } = req.body;
      
      // Build update object with only provided fields
      const updates = {};
      
      if (title !== undefined) {
        updates.title = title.trim();
      }
      
      if (description !== undefined) {
        updates.description = description.trim();
      }
      
      if (completed !== undefined) {
        updates.completed = completed;
      }
      
      // Attempt to update
      const todo = Todo.update(id, updates);
      
      if (!todo) {
        const error = new Error('Todo not found');
        error.status = 404;
        return next(error);
      }
      
      res.json({
        message: 'Todo updated successfully',
        data: todo
      });
    } catch (err) {
      next(err);
    }
  },
  
  /**
   * Delete a todo
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  deleteTodo(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const deleted = Todo.delete(id);
      
      if (!deleted) {
        const error = new Error('Todo not found');
        error.status = 404;
        return next(error);
      }
      
      // Return 204 No Content for successful deletion
      res.status(204).end();
    } catch (err) {
      next(err);
    }
  },
  
  /**
   * Get todos statistics
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  getTodoStats(req, res, next) {
    try {
      const allTodos = Todo.findAll();
      const completedTodos = Todo.findAll({ completed: true });
      const pendingTodos = Todo.findAll({ completed: false });
      
      res.json({
        data: {
          total: allTodos.length,
          completed: completedTodos.length,
          pending: pendingTodos.length,
          completionRate: allTodos.length > 0 
            ? Math.round((completedTodos.length / allTodos.length) * 100) 
            : 0
        }
      });
    } catch (err) {
      next(err);
    }
  }
};

module.exports = todoController;
```

### 2. Create Controller Utilities (src/controllers/utils.js)

Add helper functions for controllers:

```javascript
/**
 * Format todo for API response
 * @param {Object} todo - Raw todo from database
 * @returns {Object} Formatted todo
 */
const formatTodo = (todo) => {
  return {
    id: todo.id,
    title: todo.title,
    description: todo.description,
    completed: Boolean(todo.completed),
    createdAt: todo.createdAt,
    updatedAt: todo.updatedAt
  };
};

/**
 * Format multiple todos
 * @param {Array} todos - Array of todos
 * @returns {Array} Formatted todos
 */
const formatTodos = (todos) => {
  return todos.map(formatTodo);
};

/**
 * Create custom error with status code
 * @param {string} message - Error message
 * @param {number} status - HTTP status code
 * @returns {Error} Error object with status
 */
const createError = (message, status) => {
  const error = new Error(message);
  error.status = status;
  return error;
};

/**
 * Parse boolean query parameter
 * @param {string} value - Query parameter value
 * @returns {boolean|undefined} Parsed boolean or undefined
 */
const parseBoolean = (value) => {
  if (value === 'true') return true;
  if (value === 'false') return false;
  return undefined;
};

module.exports = {
  formatTodo,
  formatTodos,
  createError,
  parseBoolean
};
```

### 3. Create Health Check Controller (src/controllers/healthController.js)

Implement a simple health check controller:

```javascript
const db = require('../models/db');

const healthController = {
  /**
   * Health check endpoint
   * @param {Request} req - Express request object
   * @param {Response} res - Express response object
   */
  checkHealth(req, res, next) {
    try {
      // Test database connection
      const dbCheck = db.prepare('SELECT 1').get();
      
      res.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        service: 'todo-api',
        version: process.env.npm_package_version || '1.0.0',
        environment: process.env.NODE_ENV || 'development',
        database: dbCheck ? 'connected' : 'disconnected'
      });
    } catch (err) {
      res.status(503).json({
        status: 'unhealthy',
        timestamp: new Date().toISOString(),
        service: 'todo-api',
        error: 'Database connection failed'
      });
    }
  }
};

module.exports = healthController;
```

## Dependencies and Relationships

- **Depends on**: 
  - Task 2 (Database Setup and Model Implementation) - Uses Todo model
  - Task 3 (Express Application and Middleware) - Uses error handling
- **Required by**: 
  - Task 5 (Implement API Routes) - Routes will use these controllers
  - Task 7 (Write Comprehensive Tests) - Controllers need testing

## Success Criteria

1. ✅ All CRUD operations implemented (Create, Read, Update, Delete)
2. ✅ Proper error handling with appropriate status codes
3. ✅ Consistent response format across all endpoints
4. ✅ Input data properly processed (trimming, type conversion)
5. ✅ 404 errors for non-existent resources
6. ✅ 201 status for successful creation
7. ✅ 204 status for successful deletion
8. ✅ Pagination parameters properly parsed
9. ✅ All errors passed to Express error handler

## Testing

To verify controller logic (full testing will be in Task 7):

```javascript
// Test controller functions exist
const todoController = require('./src/controllers/todoController');
console.log('Controller methods:', Object.keys(todoController));

// Test error handling
const req = { params: { id: '999' } };
const res = { 
  json: (data) => console.log('Response:', data),
  status: (code) => ({ end: () => console.log('Status:', code) })
};
const next = (err) => console.log('Error passed:', err.message);

// This should pass error to next()
todoController.getTodoById(req, res, next);
```

## Response Format Standards

### Success Responses

**List Response**:
```json
{
  "data": [...],
  "count": 10,
  "filters": {
    "completed": true,
    "limit": 10,
    "offset": 0
  }
}
```

**Single Item Response**:
```json
{
  "data": {
    "id": 1,
    "title": "Todo item",
    "description": "Description",
    "completed": false,
    "createdAt": "2023-01-01T00:00:00.000Z",
    "updatedAt": "2023-01-01T00:00:00.000Z"
  }
}
```

**Create/Update Response**:
```json
{
  "message": "Todo created successfully",
  "data": { ... }
}
```

### Error Responses

All errors are passed to the Express error handler via `next(err)` and formatted there.

## Common Issues and Solutions

1. **Type conversion errors**: Always parse numeric IDs with parseInt
2. **Undefined vs null**: Handle both cases for optional fields
3. **Boolean parsing**: Convert string 'true'/'false' to actual booleans
4. **Trimming strings**: Always trim user input to avoid whitespace issues

## Code Quality Considerations

- Use descriptive function names
- Add JSDoc comments for better documentation
- Keep functions focused on single responsibility
- Use consistent error handling pattern
- Pass all errors to Express error handler
- Don't send multiple responses

## Next Steps

After completing this task:
- Task 5: Implement API Routes (requires controllers)
- Task 7: Write tests for all controller functions

Controllers are now ready to be connected to routes for handling HTTP requests.