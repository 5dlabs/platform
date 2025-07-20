# Task 5: Implement API Routes - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 5: Implement API Routes. All criteria must be met for the task to be considered complete.

## Functional Criteria

### 1. Route Structure
All routes must be properly implemented:
- [ ] Todo routes at `/api/todos`
- [ ] Health routes at `/api/health`
- [ ] API info at `/api` root
- [ ] Routes use Express Router
- [ ] Modular route organization

### 2. Todo Routes Implementation
Routes in `src/routes/todoRoutes.js`:
- [ ] `GET /` - List all todos
- [ ] `POST /` - Create new todo
- [ ] `GET /stats` - Get statistics
- [ ] `GET /:id` - Get single todo
- [ ] `PUT /:id` - Update todo
- [ ] `DELETE /:id` - Delete todo

**Important**: Stats route must be defined before `:id` route

### 3. Route Configuration
Each route must have:
- [ ] Correct HTTP method
- [ ] Appropriate validation middleware
- [ ] Controller wrapped with asyncHandler
- [ ] Descriptive comments/documentation

Example pattern:
```javascript
router.get(
  '/',
  todoValidation.list,
  asyncHandler(todoController.getAllTodos)
);
```

### 4. Health Routes Implementation
Routes in `src/routes/healthRoutes.js`:
- [ ] `GET /` - Basic health check
- [ ] `GET /detailed` - Detailed system info
- [ ] Database connectivity check
- [ ] No authentication required

Basic health response:
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T00:00:00Z",
  "environment": "development",
  "version": "1.0.0",
  "database": "connected"
}
```

### 5. API Root Endpoint
Route at `/api`:
- [ ] Lists all available endpoints
- [ ] Includes version information
- [ ] Documents endpoint structure
- [ ] Provides clear API overview

Response format:
```json
{
  "message": "Simple Todo REST API",
  "version": "1.0.0",
  "endpoints": {
    "todos": { ... },
    "health": { ... }
  }
}
```

### 6. Application Integration
In `src/app.js`:
- [ ] Routes mounted at `/api`
- [ ] Routes added before 404 handler
- [ ] Error handler remains last
- [ ] CORS configured before routes

## Technical Criteria

### 1. Route Organization
- [ ] Separate files for different resources
- [ ] Main index.js aggregates routes
- [ ] Clean URL structure
- [ ] RESTful naming conventions

### 2. Middleware Integration
- [ ] Validation middleware applied correctly
- [ ] Async errors caught by asyncHandler
- [ ] Middleware order is correct
- [ ] No middleware conflicts

### 3. Parameter Handling
- [ ] Path parameters work (`:id`)
- [ ] Query parameters passed through
- [ ] Body parsing functions
- [ ] Parameter validation active

### 4. Error Handling
- [ ] Invalid routes return 404
- [ ] Validation errors return 400
- [ ] Async errors are caught
- [ ] Error format is consistent

## Validation Tests

### 1. Todo Routes Test
```bash
# List todos
curl http://localhost:3000/api/todos

# Create todo
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Todo"}'

# Get specific todo
curl http://localhost:3000/api/todos/1

# Update todo
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Delete todo
curl -X DELETE http://localhost:3000/api/todos/1

# Get stats
curl http://localhost:3000/api/todos/stats
```

### 2. Health Routes Test
```bash
# Basic health
curl http://localhost:3000/api/health

# Detailed health
curl http://localhost:3000/api/health/detailed
```

### 3. Validation Test
```bash
# Invalid todo creation (no title)
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{}'
# Should return 400 with validation error

# Invalid ID
curl http://localhost:3000/api/todos/abc
# Should return 400 error
```

### 4. Route Ordering Test
```bash
# Stats route should work
curl http://localhost:3000/api/todos/stats
# Should return stats, not try to find todo with id "stats"
```

## Edge Cases to Verify

1. **Route Conflicts**: `/stats` doesn't conflict with `/:id`
2. **Missing Routes**: 404 errors work correctly
3. **Method Not Allowed**: Wrong HTTP methods rejected
4. **Query Parameters**: Properly passed to controllers
5. **Large IDs**: Handle numeric overflow

## Success Indicators

- [ ] All API endpoints are accessible
- [ ] Validation executes before controllers
- [ ] Routes follow RESTful patterns
- [ ] Error responses are consistent
- [ ] Health monitoring works
- [ ] API is self-documenting

## Performance Criteria

- [ ] Routes load efficiently
- [ ] No circular dependencies
- [ ] Middleware doesn't duplicate work
- [ ] Async operations handled properly

## Security Criteria

- [ ] Input validation on all routes
- [ ] No route exposes internals
- [ ] Parameters are sanitized
- [ ] CORS properly configured

## Integration Checklist

- [ ] Controllers properly imported
- [ ] Validation middleware imported
- [ ] Error handler imported
- [ ] Routes exported correctly
- [ ] App.js updated properly

## API Endpoint Summary

Final route structure:
```
GET    /api                     - API information
GET    /api/todos              - List todos
POST   /api/todos              - Create todo
GET    /api/todos/stats        - Todo statistics
GET    /api/todos/:id          - Get todo
PUT    /api/todos/:id          - Update todo
DELETE /api/todos/:id          - Delete todo
GET    /api/health             - Health check
GET    /api/health/detailed    - Detailed health
```

## Notes for Reviewers

When reviewing this task:
1. Test all endpoints manually
2. Verify route ordering (stats before :id)
3. Check validation middleware integration
4. Ensure async errors are handled
5. Confirm health endpoints work
6. Validate RESTful structure

Task is complete when all checkboxes above can be marked as done.