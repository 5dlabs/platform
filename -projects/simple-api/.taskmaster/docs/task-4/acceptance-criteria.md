# Task 4: Implement Todo Controller - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 4: Implement Todo Controller. All criteria must be met for the task to be considered complete.

## Functional Acceptance Criteria

### 1. Todo Controller Implementation ✓
- [ ] `src/controllers/todoController.js` file exists
- [ ] Controller exports object with all required methods
- [ ] Proper imports from models directory

### 2. getAllTodos Method ✓
- [ ] **Request Processing**:
  - [ ] Extracts completed, limit, offset from query params
  - [ ] Parses string values to appropriate types
  - [ ] Applies default values (limit: 100, offset: 0)
- [ ] **Response**:
  - [ ] Returns 200 status code
  - [ ] Returns array of todo objects
  - [ ] Respects filtering and pagination
- [ ] **Error Handling**:
  - [ ] Catches and passes errors to next()

### 3. getTodoById Method ✓
- [ ] **Request Processing**:
  - [ ] Extracts ID from params
  - [ ] Parses ID to integer
- [ ] **Response**:
  - [ ] Returns 200 with todo object when found
  - [ ] Returns 404 when todo not found
- [ ] **Error Handling**:
  - [ ] Creates error with status 404 and code 'TODO_NOT_FOUND'
  - [ ] Passes errors to next()

### 4. createTodo Method ✓
- [ ] **Request Processing**:
  - [ ] Extracts title and description from body
  - [ ] Handles missing description (sets to null)
- [ ] **Response**:
  - [ ] Returns 201 Created status
  - [ ] Returns created todo object with ID
- [ ] **Error Handling**:
  - [ ] Transforms SQLITE_CONSTRAINT to 400 Bad Request
  - [ ] Sets user-friendly error message
  - [ ] Passes errors to next()

### 5. updateTodo Method ✓
- [ ] **Request Processing**:
  - [ ] Extracts ID from params and parses to integer
  - [ ] Builds updates object with only provided fields
  - [ ] Handles title, description, and completed updates
- [ ] **Response**:
  - [ ] Returns 200 with updated todo when successful
  - [ ] Returns 404 when todo not found
- [ ] **Error Handling**:
  - [ ] Creates 404 error for missing todos
  - [ ] Transforms constraint errors to 400
  - [ ] Passes errors to next()

### 6. deleteTodo Method ✓
- [ ] **Request Processing**:
  - [ ] Extracts ID from params and parses to integer
- [ ] **Response**:
  - [ ] Returns 204 No Content on success
  - [ ] Returns 404 when todo not found
- [ ] **Error Handling**:
  - [ ] Creates 404 error when delete returns false
  - [ ] Passes errors to next()

### 7. getTodoStats Method (Bonus) ✓
- [ ] Returns statistics object with:
  - [ ] total: Total number of todos
  - [ ] completed: Number of completed todos
  - [ ] pending: Number of pending todos
  - [ ] completionRate: Ratio of completed/total
- [ ] Returns 200 status code
- [ ] Handles division by zero for completion rate

### 8. Health Controller Implementation ✓
- [ ] `src/controllers/healthController.js` file exists
- [ ] **Basic Health Check**:
  - [ ] Returns status: 'ok'
  - [ ] Includes current timestamp
  - [ ] Includes process uptime
  - [ ] Includes environment
- [ ] **Detailed Health Check**:
  - [ ] Checks database connectivity
  - [ ] Returns 'ok' or 'degraded' status
  - [ ] Returns 503 if database unhealthy
  - [ ] Includes detailed check results

### 9. Controller Index ✓
- [ ] `src/controllers/index.js` exists
- [ ] Exports todoController
- [ ] Exports healthController
- [ ] Enables destructured imports

## Non-Functional Acceptance Criteria

### Error Handling Standards
- [ ] All methods use try-catch blocks
- [ ] Errors enhanced with status and code
- [ ] Database errors transformed to user messages
- [ ] All errors passed to next() function

### Response Standards
- [ ] Consistent status codes across methods
- [ ] Empty body for 204 responses
- [ ] JSON responses for all other statuses
- [ ] No sensitive information in errors

### Code Quality
- [ ] Clear method documentation
- [ ] Consistent coding style
- [ ] No business logic (delegated to models)
- [ ] Clean separation of concerns

### Performance
- [ ] No blocking operations
- [ ] Efficient query parameter parsing
- [ ] Minimal overhead in controllers

## Test Cases

### Test Case 1: Get All Todos
```javascript
// Mock request
const req = { query: { completed: 'true', limit: '10' } };
const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };
const next = jest.fn();

todoController.getAllTodos(req, res, next);

// Verify
expect(res.status).toHaveBeenCalledWith(200);
expect(Todo.findAll).toHaveBeenCalledWith({
  completed: true,
  limit: 10,
  offset: 0
});
```

### Test Case 2: Get Todo By ID - Not Found
```javascript
// Mock Todo.findById to return null
Todo.findById.mockReturnValue(null);

const req = { params: { id: '999' } };
const next = jest.fn();

todoController.getTodoById(req, res, next);

// Verify
expect(next).toHaveBeenCalledWith(
  expect.objectContaining({
    status: 404,
    code: 'TODO_NOT_FOUND'
  })
);
```

### Test Case 3: Create Todo
```javascript
const req = { body: { title: 'New Todo' } };
const mockTodo = { id: 1, title: 'New Todo', completed: false };
Todo.create.mockReturnValue(mockTodo);

todoController.createTodo(req, res, next);

// Verify
expect(res.status).toHaveBeenCalledWith(201);
expect(res.json).toHaveBeenCalledWith(mockTodo);
```

### Test Case 4: Update Todo - Partial Update
```javascript
const req = {
  params: { id: '1' },
  body: { completed: true }  // Only updating completed
};

todoController.updateTodo(req, res, next);

// Verify only completed field is updated
expect(Todo.update).toHaveBeenCalledWith(1, { completed: true });
```

### Test Case 5: Delete Todo - Success
```javascript
const req = { params: { id: '1' } };
Todo.delete.mockReturnValue(true);

todoController.deleteTodo(req, res, next);

// Verify
expect(res.status).toHaveBeenCalledWith(204);
expect(res.end).toHaveBeenCalled();
```

### Test Case 6: Health Check
```javascript
const res = { status: jest.fn().mockReturnThis(), json: jest.fn() };

healthController.getHealth(req, res);

// Verify response includes required fields
expect(res.json).toHaveBeenCalledWith(
  expect.objectContaining({
    status: 'ok',
    timestamp: expect.any(String),
    uptime: expect.any(Number)
  })
);
```

## Definition of Done
- [ ] All functional acceptance criteria are met
- [ ] All non-functional acceptance criteria are met
- [ ] All test cases pass successfully
- [ ] Controllers handle all success scenarios
- [ ] Controllers handle all error scenarios
- [ ] Proper status codes for all responses
- [ ] No direct database access in controllers
- [ ] Ready for route integration

## Notes
- Controllers should be thin - delegate complex logic to models
- Error messages should be user-friendly, not technical
- Always parse string parameters to appropriate types
- Let middleware handle validation - controllers trust the input