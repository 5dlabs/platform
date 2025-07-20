# Task 4: Create Task Management API - Autonomous AI Agent Prompt

You are tasked with implementing a complete RESTful API for task management. This API will allow authenticated users to create, read, update, and delete their tasks with proper authorization checks ensuring users can only access their own data.

## Your Mission

Build a comprehensive task management API with full CRUD functionality, pagination support, filtering capabilities, and proper authorization. All endpoints must require authentication and enforce ownership checks where applicable.

## Prerequisites

Ensure Tasks 1-3 are complete:
- Express server is running
- Database with User and Task models is functional
- JWT authentication is implemented

## Step-by-Step Instructions

### 1. Create Tasks Router

Create `src/routes/tasks.js`:

```javascript
const express = require('express');
const router = express.Router();
const Task = require('../models/Task');
const { authenticateToken } = require('../middleware/auth');

// Apply authentication to all task routes
router.use(authenticateToken);

// GET /api/tasks - Get all tasks for authenticated user
router.get('/', (req, res) => {
  try {
    const { completed, limit = 20, offset = 0 } = req.query;
    
    // Build filters
    const filters = {};
    
    // Handle completed filter
    if (completed !== undefined) {
      filters.completed = completed === 'true';
    }
    
    // Parse and validate pagination params
    const limitNum = parseInt(limit) || 20;
    const offsetNum = parseInt(offset) || 0;
    
    // Ensure reasonable limits (1-100)
    const finalLimit = Math.min(Math.max(limitNum, 1), 100);
    
    filters.limit = finalLimit;
    filters.offset = offsetNum;
    
    // Get tasks for the authenticated user
    const tasks = Task.findByUserId(req.user.id, filters);
    const totalCount = Task.countByUserId(req.user.id, { 
      completed: filters.completed 
    });
    
    // Calculate pagination metadata
    const hasNext = offsetNum + tasks.length < totalCount;
    const hasPrev = offsetNum > 0;
    
    // Format response
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
        limit: finalLimit,
        offset: offsetNum,
        hasNext,
        hasPrev
      }
    });
  } catch (error) {
    console.error('Error fetching tasks:', error);
    res.status(500).json({
      error: {
        message: 'Failed to fetch tasks',
        code: 'FETCH_ERROR'
      }
    });
  }
});

// POST /api/tasks - Create new task
router.post('/', (req, res) => {
  try {
    const { title, description } = req.body;
    
    // Validate required fields
    if (!title || title.trim().length === 0) {
      return res.status(400).json({
        error: {
          message: 'Title is required',
          field: 'title',
          code: 'MISSING_TITLE'
        }
      });
    }
    
    // Validate title length
    if (title.length > 255) {
      return res.status(400).json({
        error: {
          message: 'Title must be 255 characters or less',
          field: 'title',
          code: 'TITLE_TOO_LONG'
        }
      });
    }
    
    // Validate description length if provided
    if (description && description.length > 1000) {
      return res.status(400).json({
        error: {
          message: 'Description must be 1000 characters or less',
          field: 'description',
          code: 'DESCRIPTION_TOO_LONG'
        }
      });
    }
    
    // Create task for authenticated user
    const task = Task.create(
      req.user.id,
      title.trim(),
      description ? description.trim() : null
    );
    
    // Return created task
    res.status(201).json({
      id: task.id,
      title: task.title,
      description: task.description,
      completed: task.completed,
      createdAt: task.created_at,
      updatedAt: task.updated_at
    });
  } catch (error) {
    console.error('Error creating task:', error);
    res.status(500).json({
      error: {
        message: 'Failed to create task',
        code: 'CREATE_ERROR'
      }
    });
  }
});

// GET /api/tasks/:id - Get specific task
router.get('/:id', (req, res) => {
  try {
    const taskId = parseInt(req.params.id);
    
    // Validate ID
    if (isNaN(taskId)) {
      return res.status(400).json({
        error: {
          message: 'Invalid task ID',
          code: 'INVALID_ID'
        }
      });
    }
    
    // Find task
    const task = Task.findById(taskId);
    
    // Check if task exists
    if (!task) {
      return res.status(404).json({
        error: {
          message: 'Task not found',
          code: 'NOT_FOUND'
        }
      });
    }
    
    // Check ownership
    if (task.user_id !== req.user.id) {
      return res.status(403).json({
        error: {
          message: 'Access denied',
          code: 'FORBIDDEN'
        }
      });
    }
    
    // Return task
    res.json({
      id: task.id,
      title: task.title,
      description: task.description,
      completed: Boolean(task.completed),
      createdAt: task.created_at,
      updatedAt: task.updated_at
    });
  } catch (error) {
    console.error('Error fetching task:', error);
    res.status(500).json({
      error: {
        message: 'Failed to fetch task',
        code: 'FETCH_ERROR'
      }
    });
  }
});

// PUT /api/tasks/:id - Update task
router.put('/:id', (req, res) => {
  try {
    const taskId = parseInt(req.params.id);
    
    // Validate ID
    if (isNaN(taskId)) {
      return res.status(400).json({
        error: {
          message: 'Invalid task ID',
          code: 'INVALID_ID'
        }
      });
    }
    
    const { title, description, completed } = req.body;
    const updates = {};
    
    // Validate and prepare title update
    if (title !== undefined) {
      if (!title || title.trim().length === 0) {
        return res.status(400).json({
          error: {
            message: 'Title cannot be empty',
            field: 'title',
            code: 'EMPTY_TITLE'
          }
        });
      }
      if (title.length > 255) {
        return res.status(400).json({
          error: {
            message: 'Title must be 255 characters or less',
            field: 'title',
            code: 'TITLE_TOO_LONG'
          }
        });
      }
      updates.title = title.trim();
    }
    
    // Validate and prepare description update
    if (description !== undefined) {
      if (description && description.length > 1000) {
        return res.status(400).json({
          error: {
            message: 'Description must be 1000 characters or less',
            field: 'description',
            code: 'DESCRIPTION_TOO_LONG'
          }
        });
      }
      updates.description = description ? description.trim() : null;
    }
    
    // Handle completed status update
    if (completed !== undefined) {
      updates.completed = Boolean(completed);
    }
    
    // Check if there are any updates
    if (Object.keys(updates).length === 0) {
      return res.status(400).json({
        error: {
          message: 'No valid fields to update',
          code: 'NO_UPDATES'
        }
      });
    }
    
    // Perform update (includes ownership check)
    const success = Task.update(taskId, req.user.id, updates);
    
    if (!success) {
      return res.status(404).json({
        error: {
          message: 'Task not found or access denied',
          code: 'NOT_FOUND'
        }
      });
    }
    
    // Fetch and return updated task
    const updatedTask = Task.findById(taskId);
    
    res.json({
      id: updatedTask.id,
      title: updatedTask.title,
      description: updatedTask.description,
      completed: Boolean(updatedTask.completed),
      createdAt: updatedTask.created_at,
      updatedAt: updatedTask.updated_at
    });
  } catch (error) {
    console.error('Error updating task:', error);
    res.status(500).json({
      error: {
        message: 'Failed to update task',
        code: 'UPDATE_ERROR'
      }
    });
  }
});

// DELETE /api/tasks/:id - Delete task
router.delete('/:id', (req, res) => {
  try {
    const taskId = parseInt(req.params.id);
    
    // Validate ID
    if (isNaN(taskId)) {
      return res.status(400).json({
        error: {
          message: 'Invalid task ID',
          code: 'INVALID_ID'
        }
      });
    }
    
    // Delete task (includes ownership check)
    const success = Task.delete(taskId, req.user.id);
    
    if (!success) {
      return res.status(404).json({
        error: {
          message: 'Task not found or access denied',
          code: 'NOT_FOUND'
        }
      });
    }
    
    // Return 204 No Content on successful deletion
    res.status(204).send();
  } catch (error) {
    console.error('Error deleting task:', error);
    res.status(500).json({
      error: {
        message: 'Failed to delete task',
        code: 'DELETE_ERROR'
      }
    });
  }
});

module.exports = router;
```

### 2. Integrate Task Routes into Express App

Update `src/app.js` to include the task routes:

```javascript
// Add after auth routes import
const taskRoutes = require('./routes/tasks');

// Add after auth routes (after app.use('/auth', authRoutes))
app.use('/api/tasks', taskRoutes);
```

### 3. Test the Implementation

#### Prerequisites for Testing
First, create a test user and get an auth token:

```bash
# Register a user
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"tasktest@example.com","password":"password123"}'

# Save the access token from the response
TOKEN="<access-token-from-response>"
```

#### Test Each Endpoint

1. **Create a Task**:
```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"My First Task","description":"This is a test task"}'
```

2. **List All Tasks**:
```bash
curl http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN"
```

3. **List Tasks with Pagination**:
```bash
curl "http://localhost:3000/api/tasks?limit=10&offset=0" \
  -H "Authorization: Bearer $TOKEN"
```

4. **Filter Completed Tasks**:
```bash
curl "http://localhost:3000/api/tasks?completed=true" \
  -H "Authorization: Bearer $TOKEN"
```

5. **Get Specific Task**:
```bash
curl http://localhost:3000/api/tasks/1 \
  -H "Authorization: Bearer $TOKEN"
```

6. **Update Task**:
```bash
curl -X PUT http://localhost:3000/api/tasks/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Updated Task","completed":true}'
```

7. **Delete Task**:
```bash
curl -X DELETE http://localhost:3000/api/tasks/1 \
  -H "Authorization: Bearer $TOKEN"
```

## Verification Steps

### 1. Authentication Required
- Try accessing any endpoint without token
- Expected: 401 "Access token required"

### 2. Create Tasks
- Create multiple tasks with different data
- Verify required title validation
- Test length limits (255 for title, 1000 for description)

### 3. List Tasks
- Verify only your tasks are returned
- Test pagination (limit/offset)
- Test filtering by completed status

### 4. Get Specific Task
- Access your own task (should work)
- Try to access another user's task (should get 403)
- Try non-existent task (should get 404)

### 5. Update Tasks
- Update title, description, and completed status
- Try empty title (should fail)
- Verify updated_at timestamp changes

### 6. Delete Tasks
- Delete your own task (should work)
- Try to delete another user's task (should fail)
- Verify task is actually removed

### 7. Authorization Checks
Create another user and verify:
- User A cannot see User B's tasks
- User A cannot update User B's tasks
- User A cannot delete User B's tasks

## Success Criteria

- All CRUD operations work correctly
- Authentication is required for all endpoints
- Users can only access their own tasks
- Pagination works with correct metadata
- Filtering by completed status works
- Input validation prevents invalid data
- Error responses follow consistent format
- Proper HTTP status codes are used
- Task ownership is enforced on all operations

## Common Issues and Solutions

1. **Tasks visible across users**
   - Ensure all queries filter by user_id
   - Check Task model methods include user checks

2. **Pagination not working**
   - Verify limit/offset are parsed as integers
   - Check SQL queries include LIMIT and OFFSET

3. **Updated timestamp not changing**
   - Ensure SQLite trigger is properly created
   - Check that updates actually modify the database

4. **Authorization bypassed**
   - Verify authenticateToken middleware is applied
   - Check ownership validation in update/delete

## Important Notes

- Always validate user input before processing
- Use prepared statements (already in Task model)
- Return consistent error response format
- Log errors for debugging but don't expose internals
- Limit maximum page size to prevent abuse
- Tasks are soft-deleted or hard-deleted based on requirements

You have now successfully implemented a complete task management API. Users can perform all CRUD operations on their tasks with proper authentication and authorization.