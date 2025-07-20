# Task 5: Implement API Routes - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 5: Implement API Routes. All criteria must be met for the task to be considered complete.

## Functional Acceptance Criteria

### 1. Todo Routes Implementation ✓
- [ ] `src/routes/todos.js` file exists
- [ ] Express router created and exported
- [ ] All required imports present:
  - [ ] todoController from controllers
  - [ ] todoValidation from middleware
  - [ ] handleValidationErrors from middleware

### 2. Todo Route Endpoints ✓
- [ ] **GET /** - List todos
  - [ ] Route defined with path '/'
  - [ ] Uses todoValidation.list middleware
  - [ ] Uses handleValidationErrors middleware
  - [ ] Calls todoController.getAllTodos
- [ ] **GET /stats** - Todo statistics
  - [ ] Route defined BEFORE /:id route
  - [ ] No validation required
  - [ ] Calls todoController.getTodoStats
- [ ] **GET /:id** - Get single todo
  - [ ] Route defined with path '/:id'
  - [ ] Uses todoValidation.getOne middleware
  - [ ] Uses handleValidationErrors middleware
  - [ ] Calls todoController.getTodoById
- [ ] **POST /** - Create todo
  - [ ] Route defined with path '/'
  - [ ] Uses todoValidation.create middleware
  - [ ] Uses handleValidationErrors middleware
  - [ ] Calls todoController.createTodo
- [ ] **PUT /:id** - Update todo
  - [ ] Route defined with path '/:id'
  - [ ] Uses todoValidation.update middleware
  - [ ] Uses handleValidationErrors middleware
  - [ ] Calls todoController.updateTodo
- [ ] **DELETE /:id** - Delete todo
  - [ ] Route defined with path '/:id'
  - [ ] Uses todoValidation.delete middleware
  - [ ] Uses handleValidationErrors middleware
  - [ ] Calls todoController.deleteTodo

### 3. Health Routes Implementation ✓
- [ ] `src/routes/health.js` file exists
- [ ] Express router created and exported
- [ ] healthController imported from controllers
- [ ] **GET /** - Basic health check
  - [ ] Route defined with path '/'
  - [ ] No validation required
  - [ ] Calls healthController.getHealth
- [ ] **GET /detailed** - Detailed health check
  - [ ] Route defined with path '/detailed'
  - [ ] No validation required
  - [ ] Calls healthController.getDetailedHealth

### 4. Routes Index ✓
- [ ] `src/routes/index.js` file exists
- [ ] Main router created and exported
- [ ] Todo routes imported and mounted at '/todos'
- [ ] Health routes imported and mounted at '/health'
- [ ] **API Root Endpoint**:
  - [ ] GET '/' route defined
  - [ ] Returns API information JSON
  - [ ] Includes version
  - [ ] Lists available endpoints

### 5. Application Integration ✓
- [ ] `src/app.js` updated to include routes
- [ ] Routes imported from './routes'
- [ ] Routes mounted at '/api' prefix
- [ ] Routes registered BEFORE 404 handler
- [ ] Routes registered BEFORE error handler

## Non-Functional Acceptance Criteria

### Route Design Standards
- [ ] RESTful conventions followed
- [ ] Consistent URL patterns
- [ ] Proper HTTP methods used
- [ ] Clear endpoint naming

### Middleware Order
- [ ] Validation middleware applied first
- [ ] Error handler follows validation
- [ ] Controller called last in chain
- [ ] Middleware properly chained with commas

### Code Organization
- [ ] Routes grouped by resource type
- [ ] Clear file structure
- [ ] Proper module exports
- [ ] No business logic in routes

### Documentation
- [ ] Route comments describe purpose
- [ ] Parameter requirements documented
- [ ] Response types indicated
- [ ] Access level noted (all public)

## Test Cases

### Test Case 1: Route Registration
```javascript
const app = require('../src/app');
const routes = app._router.stack
  .filter(r => r.route)
  .map(r => `${Object.keys(r.route.methods)[0].toUpperCase()} ${r.route.path}`);

console.log(routes);
// Should include all defined routes
```

### Test Case 2: Todo Routes Integration
```bash
# Test each endpoint
curl http://localhost:3000/api/todos
curl http://localhost:3000/api/todos/stats
curl http://localhost:3000/api/todos/1
curl -X POST http://localhost:3000/api/todos -H "Content-Type: application/json" -d '{"title":"Test"}'
curl -X PUT http://localhost:3000/api/todos/1 -H "Content-Type: application/json" -d '{"completed":true}'
curl -X DELETE http://localhost:3000/api/todos/1
```

### Test Case 3: Validation Middleware
```bash
# Missing required field
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{}'

# Expected: 400 Bad Request with validation error
```

### Test Case 4: Route Order (Stats vs ID)
```bash
# This should return statistics, not try to find todo with id "stats"
curl http://localhost:3000/api/todos/stats

# Expected: Statistics object, not 404 error
```

### Test Case 5: Health Endpoints
```bash
# Basic health
curl http://localhost:3000/api/health

# Detailed health
curl http://localhost:3000/api/health/detailed

# Both should return 200 with appropriate data
```

### Test Case 6: API Root
```bash
curl http://localhost:3000/api

# Expected response:
{
  "message": "Simple Todo REST API",
  "version": "1.0.0",
  "endpoints": {
    "todos": "/api/todos",
    "health": "/api/health",
    "documentation": "/api-docs"
  }
}
```

### Test Case 7: 404 Handling
```bash
curl http://localhost:3000/api/nonexistent

# Expected: 404 error response
```

## Definition of Done
- [ ] All functional acceptance criteria are met
- [ ] All non-functional acceptance criteria are met
- [ ] All test cases pass successfully
- [ ] All endpoints accessible via HTTP
- [ ] Validation working on all inputs
- [ ] Proper HTTP methods accepted
- [ ] Routes follow REST conventions
- [ ] Integration with app.js complete
- [ ] Ready for API documentation

## Common Issues Checklist
- [ ] Stats route comes before /:id route
- [ ] Validation middleware includes error handler
- [ ] Routes mounted with correct prefixes
- [ ] No typos in controller method names
- [ ] All routers properly exported

## Notes
- Route order is critical - specific routes before parameters
- Always test validation is working before proceeding
- Use consistent naming throughout the API
- Health endpoints should not require authentication