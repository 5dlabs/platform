# Task 4: Implement Todo Controller

## Overview
Create controller functions to handle todo CRUD operations with proper business logic, error handling, and response formatting. This task implements the controller layer that bridges between routes and models.

## Task Details
**ID**: 4  
**Title**: Implement Todo Controller  
**Priority**: Medium  
**Dependencies**: 
- [Task 2: Database Setup and Model Implementation](../task-2/task.md)
- [Task 3: Implement Express Application and Middleware](../task-3/task.md)  
**Status**: Pending

## Architecture Context
This task implements the Controller Layer as defined in the [architecture document](../../architecture.md):
- Business logic implementation
- Request/response handling
- Data validation and sanitization
- Error response formatting
- Bridge between routes and models

Key architectural responsibilities:
- Transform HTTP requests into model operations
- Handle business logic and validation
- Format responses consistently
- Manage error states gracefully

## Product Requirements Alignment
Implements API endpoint requirements from PRD:
- GET /api/todos with filtering (completed, limit, offset)
- POST /api/todos for todo creation
- GET /api/todos/:id for single todo retrieval
- PUT /api/todos/:id for updates
- DELETE /api/todos/:id for deletion
- Consistent JSON response formats

## Implementation Steps

### 1. Create Todo Controller
Create `src/controllers/todoController.js`:
```javascript
const Todo = require('../models/todo');
const { NotFoundError, AppError } = require('../middleware/errorHandler');

const todoController = {
  /**
   * Get all todos with optional filtering
   * Query params: completed (boolean), limit (number), offset (number)
   */
  async getAllTodos(req, res, next) {
    try {
      const { completed, limit, offset } = req.query;
      
      // Convert and validate query parameters
      const filters = {};
      
      if (completed !== undefined) {
        filters.completed = completed === 'true';
      }
      
      if (limit) {
        filters.limit = parseInt(limit, 10);
        if (isNaN(filters.limit) || filters.limit < 1) {
          throw new AppError('Invalid limit parameter', 400, 'INVALID_PARAMETER');
        }
      }
      
      if (offset) {
        filters.offset = parseInt(offset, 10);
        if (isNaN(filters.offset) || filters.offset < 0) {
          throw new AppError('Invalid offset parameter', 400, 'INVALID_PARAMETER');
        }
      }
      
      // Get todos from model
      const todos = Todo.findAll(filters);
      
      // Add metadata to response
      const response = {
        data: todos,
        count: todos.length,
        ...(filters.limit && { limit: filters.limit }),
        ...(filters.offset && { offset: filters.offset })
      };
      
      res.json(response);
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Get a single todo by ID
   */
  async getTodoById(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      
      if (isNaN(id) || id < 1) {
        throw new AppError('Invalid todo ID', 400, 'INVALID_ID');
      }
      
      const todo = Todo.findById(id);
      
      if (!todo) {
        throw new NotFoundError('Todo');
      }
      
      res.json({ data: todo });
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Create a new todo
   */
  async createTodo(req, res, next) {
    try {
      const { title, description } = req.body;
      
      // Additional validation (beyond middleware)
      if (title && title.trim().length === 0) {
        throw new AppError('Title cannot be empty', 400, 'INVALID_TITLE');
      }
      
      // Create todo
      const todoData = {
        title: title.trim(),
        description: description ? description.trim() : null
      };
      
      const todo = Todo.create(todoData);
      
      if (!todo) {
        throw new AppError('Failed to create todo', 500, 'CREATE_FAILED');
      }
      
      res.status(201).json({
        data: todo,
        message: 'Todo created successfully'
      });
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Update an existing todo
   */
  async updateTodo(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      const { title, description, completed } = req.body;
      
      if (isNaN(id) || id < 1) {
        throw new AppError('Invalid todo ID', 400, 'INVALID_ID');
      }
      
      // Check if todo exists
      const existingTodo = Todo.findById(id);
      if (!existingTodo) {
        throw new NotFoundError('Todo');
      }
      
      // Build update object
      const updates = {};
      
      if (title !== undefined) {
        if (title.trim().length === 0) {
          throw new AppError('Title cannot be empty', 400, 'INVALID_TITLE');
        }
        updates.title = title.trim();
      }
      
      if (description !== undefined) {
        updates.description = description ? description.trim() : null;
      }
      
      if (completed !== undefined) {
        updates.completed = Boolean(completed);
      }
      
      // Perform update
      const updatedTodo = Todo.update(id, updates);
      
      if (!updatedTodo) {
        throw new AppError('Failed to update todo', 500, 'UPDATE_FAILED');
      }
      
      res.json({
        data: updatedTodo,
        message: 'Todo updated successfully'
      });
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Delete a todo
   */
  async deleteTodo(req, res, next) {
    try {
      const id = parseInt(req.params.id, 10);
      
      if (isNaN(id) || id < 1) {
        throw new AppError('Invalid todo ID', 400, 'INVALID_ID');
      }
      
      // Check if todo exists
      const existingTodo = Todo.findById(id);
      if (!existingTodo) {
        throw new NotFoundError('Todo');
      }
      
      // Delete todo
      const deleted = Todo.delete(id);
      
      if (!deleted) {
        throw new AppError('Failed to delete todo', 500, 'DELETE_FAILED');
      }
      
      // Return 204 No Content on successful deletion
      res.status(204).end();
    } catch (error) {
      next(error);
    }
  },
  
  /**
   * Get todo statistics (additional utility endpoint)
   */
  async getTodoStats(req, res, next) {
    try {
      const totalCount = Todo.count();
      const completedCount = Todo.count({ completed: true });
      const pendingCount = Todo.count({ completed: false });
      
      res.json({
        data: {
          total: totalCount,
          completed: completedCount,
          pending: pendingCount,
          completionRate: totalCount > 0 
            ? Math.round((completedCount / totalCount) * 100) 
            : 0
        }
      });
    } catch (error) {
      next(error);
    }
  }
};

module.exports = todoController;
```

### 2. Create Controller Index File
Create `src/controllers/index.js`:
```javascript
const todoController = require('./todoController');

module.exports = {
  todoController
};
```

### 3. Add Response Formatter Utility (Optional)
Create `src/utils/responseFormatter.js`:
```javascript
/**
 * Format successful response
 */
const successResponse = (res, data, message = null, statusCode = 200) => {
  const response = {
    success: true,
    data
  };
  
  if (message) {
    response.message = message;
  }
  
  return res.status(statusCode).json(response);
};

/**
 * Format error response
 */
const errorResponse = (res, error, statusCode = 500) => {
  return res.status(statusCode).json({
    success: false,
    error: {
      message: error.message || 'An error occurred',
      code: error.code || 'UNKNOWN_ERROR'
    }
  });
};

module.exports = {
  successResponse,
  errorResponse
};
```

## Response Formats

### Success Response Format
```json
{
  "data": {
    "id": 1,
    "title": "Sample Todo",
    "description": "Description",
    "completed": false,
    "createdAt": "2024-01-01T00:00:00Z",
    "updatedAt": "2024-01-01T00:00:00Z"
  },
  "message": "Todo created successfully"
}
```

### List Response Format
```json
{
  "data": [...],
  "count": 10,
  "limit": 20,
  "offset": 0
}
```

### Error Response Format
```json
{
  "error": {
    "message": "Todo not found",
    "code": "NOT_FOUND"
  }
}
```

## Success Criteria
- All CRUD operations are implemented
- Proper error handling for all edge cases
- Consistent response formats
- Input validation and sanitization
- Proper HTTP status codes returned
- Controller methods are testable
- Business logic is separated from HTTP concerns

## Testing Considerations
- Mock the Todo model for unit testing
- Test all success paths
- Test error scenarios (not found, invalid input)
- Verify response formats
- Test parameter validation and conversion
- Ensure proper error propagation

## Related Tasks
- **Dependencies**: 
  - [Task 2: Database Setup and Model Implementation](../task-2/task.md) - Provides Todo model
  - [Task 3: Implement Express Application and Middleware](../task-3/task.md) - Provides error handling
- **Next**: [Task 5: Implement API Routes](../task-5/task.md) - Will use these controllers
- **Related**: [Task 7: Write Comprehensive Tests](../task-7/task.md) - Will test these controllers

## References
- [Architecture Document](../../architecture.md) - Section: Controller Layer
- [Product Requirements](../../prd.txt) - Section: API Endpoints