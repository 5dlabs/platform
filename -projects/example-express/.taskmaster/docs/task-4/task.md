# Task 4: Create Task Management API Endpoints

## Overview

This task implements the core functionality of the application by creating RESTful API endpoints for task management. Building upon the authentication system from Task 3 and the Task model from Task 2, this task provides a complete CRUD (Create, Read, Update, Delete) API with proper authorization checks to ensure users can only access their own tasks.

## Objectives

- Implement RESTful endpoints for task management
- Ensure all endpoints require authentication
- Add authorization checks so users can only access their own tasks
- Support filtering and pagination for task lists
- Implement proper HTTP status codes and response formats
- Add input validation for task creation and updates

## Technical Requirements

### API Endpoints
- `GET /api/tasks` - List all tasks for authenticated user
- `POST /api/tasks` - Create a new task
- `GET /api/tasks/:id` - Get a specific task
- `PUT /api/tasks/:id` - Update a task
- `DELETE /api/tasks/:id` - Delete a task

### Request/Response Formats
- JSON request and response bodies
- Consistent error response structure
- Proper HTTP status codes
- Pagination metadata for list endpoints

## Implementation Steps

### 1. Create Tasks Router (Subtask 4.1-4.4)

Create `src/routes/tasks.js`:
```javascript
const express = require('express');
const router = express.Router();
const Task = require('../models/Task');
const { authenticateToken } = require('../middleware/auth');

// Apply authentication to all routes
router.use(authenticateToken);

// GET /api/tasks - Get all tasks for authenticated user
router.get('/', (req, res) => {
  try {
    const { completed, limit = 20, offset = 0 } = req.query;
    
    // Build filters
    const filters = {};
    if (completed !== undefined) {
      filters.completed = completed === 'true';
    }
    
    // Parse pagination params
    const limitNum = parseInt(limit) || 20;
    const offsetNum = parseInt(offset) || 0;
    
    // Ensure reasonable limits
    const finalLimit = Math.min(Math.max(limitNum, 1), 100);
    
    filters.limit = finalLimit;
    filters.offset = offsetNum;
    
    // Get tasks and count
    const tasks = Task.findByUserId(req.user.id, filters);
    const totalCount = Task.countByUserId(req.user.id, { completed: filters.completed });
    
    // Calculate pagination metadata
    const hasNext = offsetNum + tasks.length < totalCount;
    const hasPrev = offsetNum > 0;
    
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
    
    // Create task
    const task = Task.create(
      req.user.id,
      title.trim(),
      description ? description.trim() : null
    );
    
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
    
    if (isNaN(taskId)) {
      return res.status(400).json({
        error: {
          message: 'Invalid task ID',
          code: 'INVALID_ID'
        }
      });
    }
    
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
    
    // Validate and prepare updates
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

### 2. Update Task Model for Pagination (Already included in Task 2)

The Task model from Task 2 already includes the necessary methods:
- `findByUserId` with limit/offset support
- `countByUserId` for total count

### 3. Integrate Task Routes

Update `src/app.js` to include task routes:
```javascript
// Add after auth routes
const taskRoutes = require('./routes/tasks');

// Add the route
app.use('/api/tasks', taskRoutes);
```

### 4. Create API Documentation

Document the API endpoints for users:

#### GET /api/tasks
```
Description: Get all tasks for the authenticated user
Authentication: Required (Bearer token)
Query Parameters:
  - completed (optional): Filter by completion status (true/false)
  - limit (optional): Number of tasks to return (default: 20, max: 100)
  - offset (optional): Number of tasks to skip (default: 0)
Response: 200 OK
{
  "tasks": [
    {
      "id": 1,
      "title": "Task title",
      "description": "Task description",
      "completed": false,
      "createdAt": "2025-07-20T12:00:00.000Z",
      "updatedAt": "2025-07-20T12:00:00.000Z"
    }
  ],
  "pagination": {
    "total": 50,
    "limit": 20,
    "offset": 0,
    "hasNext": true,
    "hasPrev": false
  }
}
```

#### POST /api/tasks
```
Description: Create a new task
Authentication: Required (Bearer token)
Request Body:
{
  "title": "Task title (required, max 255 chars)",
  "description": "Task description (optional, max 1000 chars)"
}
Response: 201 Created
{
  "id": 1,
  "title": "Task title",
  "description": "Task description",
  "completed": false,
  "createdAt": "2025-07-20T12:00:00.000Z",
  "updatedAt": "2025-07-20T12:00:00.000Z"
}
```

#### GET /api/tasks/:id
```
Description: Get a specific task
Authentication: Required (Bearer token)
Response: 200 OK
{
  "id": 1,
  "title": "Task title",
  "description": "Task description",
  "completed": false,
  "createdAt": "2025-07-20T12:00:00.000Z",
  "updatedAt": "2025-07-20T12:00:00.000Z"
}
Errors:
  - 404: Task not found
  - 403: Access denied (not owner)
```

#### PUT /api/tasks/:id
```
Description: Update a task
Authentication: Required (Bearer token)
Request Body (all fields optional):
{
  "title": "Updated title",
  "description": "Updated description",
  "completed": true
}
Response: 200 OK
{
  "id": 1,
  "title": "Updated title",
  "description": "Updated description",
  "completed": true,
  "createdAt": "2025-07-20T12:00:00.000Z",
  "updatedAt": "2025-07-20T13:00:00.000Z"
}
```

#### DELETE /api/tasks/:id
```
Description: Delete a task
Authentication: Required (Bearer token)
Response: 204 No Content
Errors:
  - 404: Task not found or access denied
```

## Testing

### Manual Testing with cURL

1. **Create a task**:
```bash
TOKEN="your-auth-token"
curl -X POST http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Task","description":"This is a test"}'
```

2. **List tasks**:
```bash
curl http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN"
```

3. **List completed tasks**:
```bash
curl "http://localhost:3000/api/tasks?completed=true" \
  -H "Authorization: Bearer $TOKEN"
```

4. **Get specific task**:
```bash
curl http://localhost:3000/api/tasks/1 \
  -H "Authorization: Bearer $TOKEN"
```

5. **Update task**:
```bash
curl -X PUT http://localhost:3000/api/tasks/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'
```

6. **Delete task**:
```bash
curl -X DELETE http://localhost:3000/api/tasks/1 \
  -H "Authorization: Bearer $TOKEN"
```

### Integration Tests

Create `tests/tasks.test.js`:
```javascript
const request = require('supertest');
const { app } = require('../src/app');
const { resetDatabase } = require('../src/db/init');

let authToken;
let userId;

beforeEach(async () => {
  resetDatabase();
  
  // Create test user and get token
  const response = await request(app)
    .post('/auth/register')
    .send({
      email: 'testuser@example.com',
      password: 'password123'
    });
  
  authToken = response.body.tokens.accessToken;
  userId = response.body.user.id;
});

describe('Task API', () => {
  describe('POST /api/tasks', () => {
    test('creates new task', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${authToken}`)
        .send({
          title: 'Test Task',
          description: 'Test Description'
        });
      
      expect(response.status).toBe(201);
      expect(response.body.title).toBe('Test Task');
      expect(response.body.completed).toBe(false);
    });
    
    test('requires authentication', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .send({
          title: 'Test Task'
        });
      
      expect(response.status).toBe(401);
    });
    
    test('requires title', async () => {
      const response = await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${authToken}`)
        .send({
          description: 'No title'
        });
      
      expect(response.status).toBe(400);
      expect(response.body.error.code).toBe('MISSING_TITLE');
    });
  });
  
  describe('GET /api/tasks', () => {
    beforeEach(async () => {
      // Create some test tasks
      await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${authToken}`)
        .send({ title: 'Task 1' });
      
      await request(app)
        .post('/api/tasks')
        .set('Authorization', `Bearer ${authToken}`)
        .send({ title: 'Task 2', completed: true });
    });
    
    test('returns user tasks', async () => {
      const response = await request(app)
        .get('/api/tasks')
        .set('Authorization', `Bearer ${authToken}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(2);
      expect(response.body.pagination.total).toBe(2);
    });
    
    test('filters by completed status', async () => {
      const response = await request(app)
        .get('/api/tasks?completed=true')
        .set('Authorization', `Bearer ${authToken}`);
      
      expect(response.status).toBe(200);
      expect(response.body.tasks).toHaveLength(1);
      expect(response.body.tasks[0].completed).toBe(true);
    });
  });
});
```

## Common Issues and Solutions

### Issue: Tasks from other users visible
**Solution**: Always filter by req.user.id in queries

### Issue: Pagination not working correctly
**Solution**: Ensure limit and offset are properly parsed as integers

### Issue: Updated_at not changing
**Solution**: The trigger in SQLite should handle this automatically

## Security Considerations

1. **Always verify task ownership** before updates/deletes
2. **Validate all input** to prevent SQL injection
3. **Use authentication middleware** on all routes
4. **Return generic errors** for unauthorized access
5. **Limit query results** to prevent large data dumps

## Next Steps

After completing this task:
- Full CRUD API for tasks is operational
- Users can manage their tasks via API
- Proper authorization ensures data isolation
- Pagination supports large task lists
- Ready for Task 5: Add Request Validation and Error Handling

The API now provides all the core functionality needed for task management, with security and scalability considerations built in.