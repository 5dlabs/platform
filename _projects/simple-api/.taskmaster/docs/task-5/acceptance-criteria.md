# Task 5: Implement API Routes - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 5: Implement API Routes. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Todo Routes Implementation ✓

**Given** controllers and validation exist (Tasks 3 & 4)
**When** checking src/routes/todoRoutes.js
**Then** it must implement:
- GET / → List all todos with query filters
- POST / → Create new todo
- GET /:id → Get single todo
- PUT /:id → Update todo
- DELETE /:id → Delete todo
- Proper route order (specific before generic)
- All routes use validation middleware
- All routes call appropriate controllers

**Test**:
```bash
curl -X GET http://localhost:3000/api/todos
curl -X POST http://localhost:3000/api/todos -H "Content-Type: application/json" -d '{"title":"Test"}'
curl -X GET http://localhost:3000/api/todos/1
curl -X PUT http://localhost:3000/api/todos/1 -H "Content-Type: application/json" -d '{"completed":true}'
curl -X DELETE http://localhost:3000/api/todos/1
```

### 2. Health Routes Implementation ✓

**Given** the need for health monitoring
**When** checking src/routes/healthRoutes.js
**Then** it must implement:
- GET / → Main health check with database status
- GET /ready → Readiness probe for load balancers
- GET /live → Liveness probe for orchestration
- Proper status codes (200 healthy, 503 unhealthy)

**Test**:
```bash
curl http://localhost:3000/api/health
# Expected: {"status":"healthy","timestamp":"...","database":"connected"}

curl http://localhost:3000/api/health/ready
# Expected: {"ready":true}

curl http://localhost:3000/api/health/live
# Expected: {"alive":true}
```

### 3. Route Organization ✓

**Given** multiple route modules
**When** checking src/routes/index.js
**Then** it must:
- Import all route modules
- Mount todo routes at /todos
- Mount health routes at /health
- Provide API info at root endpoint
- Export combined router

**Test**:
```bash
curl http://localhost:3000/api
# Expected: API information with endpoints list
```

### 4. Express App Integration ✓

**Given** routes are defined
**When** checking src/app.js
**Then** it must:
- Import routes from ./routes
- Mount routes at /api base path
- Routes come after body parsers
- Routes come before 404 handler
- Routes come before error handler
- Include request ID in responses

**Test**: Server starts and routes are accessible

### 5. RESTful Design Compliance ✓

**Given** REST principles
**When** reviewing all routes
**Then** they must follow:

| Operation | Method | Path | Status Codes |
|-----------|--------|------|--------------|
| List | GET | /api/todos | 200 |
| Create | POST | /api/todos | 201 |
| Read | GET | /api/todos/:id | 200, 404 |
| Update | PUT | /api/todos/:id | 200, 404 |
| Delete | DELETE | /api/todos/:id | 204, 404 |

**Test**: Each route returns appropriate status code

### 6. Validation Middleware Integration ✓

**Given** validation rules from Task 3
**When** making requests
**Then** validation must:
- Reject invalid requests before reaching controller
- Return 400 with validation error details
- Apply to all routes that accept input
- Not interfere with valid requests

**Test Cases**:
```bash
# Missing title (should fail)
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"description":"No title"}'
# Expected: 400 Validation Error

# Invalid ID (should fail)
curl http://localhost:3000/api/todos/abc
# Expected: 400 Validation Error

# Valid request (should succeed)
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Valid Todo"}'
# Expected: 201 Created
```

### 7. Query Parameter Handling ✓

**Given** GET /api/todos accepts filters
**When** using query parameters
**Then** they must:
- Support completed filter (true/false)
- Support limit (1-100)
- Support offset (>= 0)
- Pass through validation
- Reach controller properly

**Test**:
```bash
curl "http://localhost:3000/api/todos?completed=true&limit=5&offset=0"
# Should filter and paginate results
```

### 8. Error Handling Flow ✓

**Given** various error scenarios
**When** errors occur
**Then** the route/app must:
- Let controllers pass errors to next()
- Not catch errors in routes
- Let error middleware handle all errors
- Include request ID in error responses

**Test**: Force errors and verify proper handling

## Route Test Script Validation

**Given** test-routes.sh exists
**When** running the script
**Then** it must test:
- All CRUD operations
- Validation errors
- Query parameters
- Health endpoints
- Success and error cases

**Run Test**:
```bash
chmod +x test-routes.sh
./test-routes.sh
```

## API Structure Verification

### Expected Route Tree
```
GET     /                           # Welcome message
GET     /api                        # API info
GET     /api/todos                  # List todos
POST    /api/todos                  # Create todo
GET     /api/todos/stats/summary    # Statistics
GET     /api/todos/:id              # Get todo
PUT     /api/todos/:id              # Update todo
DELETE  /api/todos/:id              # Delete todo
GET     /api/health                 # Health check
GET     /api/health/ready           # Readiness
GET     /api/health/live            # Liveness
*       /*                          # 404 handler
```

### Route Order Requirements
1. Specific routes before parameter routes
2. `/todos/stats/summary` before `/todos/:id`
3. All routes before 404 handler
4. Error handler last

## Integration Test Scenarios

### Scenario 1: Full CRUD Cycle
```bash
# 1. Create
RESULT=$(curl -s -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Integration Test"}')
ID=$(echo $RESULT | jq -r '.data.id')

# 2. Read
curl -s http://localhost:3000/api/todos/$ID

# 3. Update
curl -s -X PUT http://localhost:3000/api/todos/$ID \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# 4. Delete
curl -s -X DELETE http://localhost:3000/api/todos/$ID

# 5. Verify deletion
curl -s http://localhost:3000/api/todos/$ID
# Should return 404
```

### Scenario 2: Validation Chain
```bash
# Invalid requests should be rejected by validation
# Valid requests should reach controller
# Errors should be handled gracefully
```

## Definition of Done

- [ ] All todo CRUD routes implemented
- [ ] Health check routes implemented
- [ ] Routes properly organized in modules
- [ ] Routes integrated into Express app
- [ ] Validation middleware applied correctly
- [ ] RESTful conventions followed
- [ ] Proper HTTP methods and status codes
- [ ] Query parameter support working
- [ ] Test script created and functional
- [ ] All routes return consistent response format
- [ ] Request IDs included in all responses
- [ ] No business logic in route files

## Performance Criteria

- Route resolution < 1ms
- No blocking operations in routes
- Efficient route matching order
- Minimal middleware overhead

## Security Criteria

- All input validated before processing
- No direct database access in routes
- Proper error information hiding
- Request size limits enforced

## Notes

- Routes should be thin layers
- Business logic stays in controllers
- Validation logic stays in middleware
- Routes just connect the pieces
- Maintain consistent URL patterns
- Follow REST naming conventions