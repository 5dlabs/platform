# Task 4: Implement Todo Controller - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 4: Implement Todo Controller. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Todo Controller Structure ✓

**Given** the Todo model exists (Task 2)
**When** checking src/controllers/todoController.js
**Then** it must:
- Export an object with all required methods
- Import the Todo model correctly
- Use consistent error handling with try/catch
- Pass errors to next() function
- Not contain any route definitions
- Not contain any SQL queries

**Test**:
```bash
node -e "const ctrl = require('./src/controllers/todoController'); console.log(Object.keys(ctrl).length >= 5 ? 'Controller OK' : 'Missing methods')"
```

### 2. GetAllTodos Implementation ✓

**Given** a request for all todos
**When** getAllTodos is called
**Then** it must:
- Parse query parameters (completed, limit, offset)
- Convert 'true'/'false' strings to booleans
- Parse numeric values for limit/offset
- Call Todo.findAll() with correct filters
- Return response with data, count, and filters
- Handle errors with next()

**Test Case**:
```javascript
// req.query = { completed: 'true', limit: '10', offset: '0' }
// Should call Todo.findAll({ completed: true, limit: 10, offset: 0 })
// Response: { data: [...], count: X, filters: {...} }
```

### 3. GetTodoById Implementation ✓

**Given** a request for a specific todo
**When** getTodoById is called
**Then** it must:
- Parse ID from req.params.id as integer
- Call Todo.findById(id)
- Return 404 error if todo not found
- Return todo data if found
- Use consistent response format

**Test Cases**:
| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Todo exists | id: "1" | Status 200, { data: todo } |
| Todo not found | id: "999" | Status 404, error passed to next() |
| Invalid ID | id: "abc" | Handle gracefully |

### 4. CreateTodo Implementation ✓

**Given** a request to create a todo
**When** createTodo is called
**Then** it must:
- Extract title and description from req.body
- Trim whitespace from strings
- Handle missing description (set to null)
- Call Todo.create() with cleaned data
- Return 201 status code
- Include success message and created todo

**Test Case**:
```javascript
// req.body = { title: "  New Todo  ", description: "  Description  " }
// Should create with: { title: "New Todo", description: "Description" }
// Response: 201, { message: "...", data: createdTodo }
```

### 5. UpdateTodo Implementation ✓

**Given** a request to update a todo
**When** updateTodo is called
**Then** it must:
- Parse ID from params
- Build update object with only provided fields
- Trim string values
- Not include undefined fields in update
- Call Todo.update(id, updates)
- Return 404 if todo not found
- Return updated todo if successful

**Test Cases**:
```javascript
// Partial update: { title: "Updated" }
// Should only update title, not touch other fields

// Full update: { title: "New", description: "Desc", completed: true }
// Should update all provided fields

// Empty update: {}
// Should handle gracefully
```

### 6. DeleteTodo Implementation ✓

**Given** a request to delete a todo
**When** deleteTodo is called
**Then** it must:
- Parse ID from params
- Call Todo.delete(id)
- Return 404 if todo not found
- Return 204 No Content if successful
- Not return any body for successful deletion

**Test**:
```javascript
// Successful deletion: res.status(204).end()
// Not found: next(error) with status 404
```

### 7. Error Handling Pattern ✓

**Given** any controller method
**When** an error occurs
**Then** it must:
- Catch all synchronous errors
- Pass errors to next() function
- Create custom errors with status codes for 404s
- Never throw unhandled errors
- Never send multiple responses

**Test**: Each method should have try/catch block

### 8. Health Controller ✓

**Given** the need for health checks
**When** checking src/controllers/healthController.js
**Then** it must:
- Test database connectivity
- Return service status
- Include timestamp
- Return 200 when healthy
- Return 503 when unhealthy
- Include environment information

**Test**:
```bash
node -e "const hc = require('./src/controllers/healthController'); console.log(typeof hc.checkHealth === 'function' ? 'Health OK' : 'Failed')"
```

## Response Format Validation

### Standard Success Response
```json
{
  "data": { ... } or [ ... ],
  "count": 10,  // for lists
  "filters": { ... },  // for lists
  "message": "..."  // for mutations
}
```

### Standard Error (passed to error handler)
```javascript
const error = new Error('Todo not found');
error.status = 404;
next(error);
```

## Test Scenarios

### Scenario 1: Controller Method Signatures
```javascript
const todoController = require('./src/controllers/todoController');

// All methods should accept (req, res, next)
const methods = ['getAllTodos', 'getTodoById', 'createTodo', 'updateTodo', 'deleteTodo'];
methods.forEach(method => {
  console.log(`${method}: ${todoController[method].length === 3 ? 'OK' : 'Invalid signature'}`);
});
```

### Scenario 2: Mock Testing
```javascript
// Mock objects for testing
const mockReq = {
  params: { id: '1' },
  query: { completed: 'true' },
  body: { title: 'Test' }
};

const mockRes = {
  json: jest.fn(),
  status: jest.fn(() => mockRes),
  end: jest.fn()
};

const mockNext = jest.fn();

// Test should not throw errors
todoController.getAllTodos(mockReq, mockRes, mockNext);
```

### Scenario 3: Error Propagation
```javascript
// When Todo.findById throws an error
// Controller should catch and pass to next()
// Should never let errors bubble up unhandled
```

## Definition of Done

- [ ] All 5 CRUD controller methods implemented
- [ ] Health controller implemented
- [ ] Consistent error handling with try/catch and next()
- [ ] Proper HTTP status codes (200, 201, 204, 404)
- [ ] Request data properly parsed and cleaned
- [ ] Response format consistent across all methods
- [ ] No direct database access (only through Todo model)
- [ ] No route definitions in controllers
- [ ] All methods handle edge cases gracefully
- [ ] Controller utility functions created if needed

## Performance Criteria

- Controllers add minimal overhead (< 5ms)
- No unnecessary database calls
- Efficient parameter parsing
- No memory leaks from uncaught errors

## Code Quality Criteria

- Clear function names and purposes
- JSDoc comments for documentation
- Consistent code style
- DRY principle followed
- Single responsibility per method
- Proper error messages

## Notes

- Controllers should be pure functions (testable in isolation)
- Assume validation is done by middleware (Task 3)
- Focus on business logic, not HTTP specifics
- All database operations go through the Todo model
- Error formatting is handled by Express error middleware