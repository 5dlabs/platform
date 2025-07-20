# Task 4: Implement Todo Controller - Acceptance Criteria

## Overview
This document defines the acceptance criteria for Task 4: Implement Todo Controller. All criteria must be met for the task to be considered complete.

## Functional Criteria

### 1. Controller Methods
All controller methods must be implemented:
- [ ] `getAllTodos(req, res, next)`
- [ ] `getTodoById(req, res, next)`
- [ ] `createTodo(req, res, next)`
- [ ] `updateTodo(req, res, next)`
- [ ] `deleteTodo(req, res, next)`
- [ ] `getTodoStats(req, res, next)`

### 2. getAllTodos Functionality
- [ ] Returns all todos when no filters provided
- [ ] Filters by completed status when `?completed=true/false`
- [ ] Applies limit when `?limit=n` provided
- [ ] Applies offset when `?offset=n` provided
- [ ] Returns data array with count metadata
- [ ] Handles invalid query parameters with 400 error

Response format:
```json
{
  "data": [...],
  "count": 10,
  "limit": 20,
  "offset": 0
}
```

### 3. getTodoById Functionality
- [ ] Returns single todo when found
- [ ] Returns 404 error when not found
- [ ] Validates ID is numeric
- [ ] Handles invalid ID format with 400 error

Response format:
```json
{
  "data": { todo object }
}
```

### 4. createTodo Functionality
- [ ] Creates new todo with title and description
- [ ] Trims whitespace from string inputs
- [ ] Validates title is not empty after trimming
- [ ] Returns 201 status on success
- [ ] Returns created todo with success message
- [ ] Handles database errors appropriately

Response format:
```json
{
  "data": { created todo },
  "message": "Todo created successfully"
}
```

### 5. updateTodo Functionality
- [ ] Updates only provided fields
- [ ] Checks todo exists before updating
- [ ] Trims string inputs
- [ ] Validates title not empty if provided
- [ ] Converts completed to boolean
- [ ] Returns 404 if todo not found
- [ ] Returns updated todo with message

Response format:
```json
{
  "data": { updated todo },
  "message": "Todo updated successfully"
}
```

### 6. deleteTodo Functionality
- [ ] Deletes existing todo
- [ ] Returns 204 No Content on success
- [ ] Returns 404 if todo not found
- [ ] Validates ID parameter
- [ ] No response body on success

### 7. getTodoStats Functionality
- [ ] Returns total todo count
- [ ] Returns completed count
- [ ] Returns pending count
- [ ] Calculates completion rate percentage
- [ ] Handles empty database (0% rate)

Response format:
```json
{
  "data": {
    "total": 10,
    "completed": 3,
    "pending": 7,
    "completionRate": 30
  }
}
```

## Technical Criteria

### 1. Error Handling
- [ ] All methods use try/catch blocks
- [ ] Errors are passed to next() middleware
- [ ] Custom error classes are used appropriately
- [ ] Error messages are descriptive

### 2. Input Processing
- [ ] String inputs are trimmed
- [ ] Numeric inputs are parsed correctly
- [ ] Boolean conversions handle "true"/"false" strings
- [ ] Invalid inputs return appropriate errors

### 3. Response Standards
- [ ] Correct HTTP status codes used:
  - 200 for successful GET/PUT
  - 201 for successful POST
  - 204 for successful DELETE
  - 400 for bad requests
  - 404 for not found
  - 500 for server errors
- [ ] Consistent response structure

### 4. Business Logic
- [ ] No direct database access (uses Model)
- [ ] Input validation beyond middleware
- [ ] Resource existence checks
- [ ] Proper null/undefined handling

## Validation Tests

### 1. GetAllTodos Tests
```javascript
// Test without filters
await getAllTodos(req, res, next);
// res.json called with data array and count

// Test with filters
req.query = { completed: 'true', limit: '10' };
await getAllTodos(req, res, next);
// Filters applied correctly

// Test invalid limit
req.query = { limit: 'invalid' };
await getAllTodos(req, res, next);
// next() called with 400 error
```

### 2. CreateTodo Tests
```javascript
// Valid creation
req.body = { title: '  New Todo  ', description: 'Test' };
await createTodo(req, res, next);
// res.status(201) and trimmed title saved

// Empty title after trim
req.body = { title: '   ' };
await createTodo(req, res, next);
// next() called with validation error
```

### 3. UpdateTodo Tests
```javascript
// Partial update
req.params.id = '1';
req.body = { completed: true };
await updateTodo(req, res, next);
// Only completed field updated

// Non-existent todo
req.params.id = '999';
await updateTodo(req, res, next);
// next() called with 404 error
```

## Edge Cases to Verify

1. **Empty Database**: getAllTodos returns empty array
2. **Invalid IDs**: Non-numeric IDs return 400
3. **Whitespace Handling**: Strings are trimmed properly
4. **Null Values**: Description can be null
5. **Boolean Strings**: "true"/"false" converted correctly
6. **Large Numbers**: Handle large limit/offset values

## Success Indicators

- [ ] All CRUD operations work through controllers
- [ ] Business logic is properly implemented
- [ ] Error handling is comprehensive
- [ ] Input sanitization works correctly
- [ ] Response formats are consistent
- [ ] HTTP status codes are appropriate

## Code Quality Criteria

- [ ] Methods use async/await pattern
- [ ] No callback hell or promise chains
- [ ] Clear variable names
- [ ] Proper error propagation
- [ ] DRY principles followed
- [ ] Comments for complex logic

## Integration Points

- [ ] Correctly imports Todo model
- [ ] Uses error classes from errorHandler
- [ ] Methods match route handler signature
- [ ] Exports controller object properly

## Notes for Reviewers

When reviewing this task:
1. Check all controller methods exist
2. Verify response formats match spec
3. Test error scenarios thoroughly
4. Ensure trimming works correctly
5. Validate status codes are correct
6. Confirm stats calculation accuracy

Task is complete when all checkboxes above can be marked as done.