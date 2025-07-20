# Task 4: Implement Todo Controller - Autonomous Prompt

You are an AI agent tasked with implementing the controller layer for the Simple Todo REST API. Your goal is to create controller functions that handle all todo CRUD operations with proper business logic, error handling, and response formatting.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- Task 2 (Database/Model) is complete - Todo model is available
- Task 3 (Express/Middleware) is complete - Error classes are available
- The Todo model and error handling utilities can be imported

## Your Mission
Implement the todo controller with all CRUD operations, proper error handling, input sanitization, and consistent response formatting. The controller should bridge between HTTP requests and the Todo model while handling all business logic.

## Required Actions

### 1. Create Todo Controller
Create `src/controllers/todoController.js` with these methods:

**getAllTodos(req, res, next)**:
- Extract query parameters: completed, limit, offset
- Convert and validate parameters
- Call Todo.findAll() with filters
- Return response with data array and metadata
- Handle errors with proper status codes

**getTodoById(req, res, next)**:
- Parse and validate ID parameter
- Call Todo.findById()
- Return 404 if not found
- Return todo data if found

**createTodo(req, res, next)**:
- Extract title and description from body
- Trim whitespace from strings
- Validate title is not empty after trimming
- Call Todo.create()
- Return 201 status with created todo

**updateTodo(req, res, next)**:
- Parse and validate ID parameter
- Check if todo exists first
- Build update object with only provided fields
- Trim string values
- Call Todo.update()
- Return updated todo or 404

**deleteTodo(req, res, next)**:
- Parse and validate ID parameter
- Check if todo exists first
- Call Todo.delete()
- Return 204 No Content on success

**getTodoStats(req, res, next)** (bonus):
- Get total, completed, and pending counts
- Calculate completion rate
- Return statistics object

### 2. Create Controller Index
Create `src/controllers/index.js`:
- Export todoController

### 3. Response Format Standards

Success responses:
```json
{
  "data": {...},
  "message": "Optional success message"
}
```

List responses:
```json
{
  "data": [...],
  "count": 10,
  "limit": 20,
  "offset": 0
}
```

Error handling:
- Use try/catch blocks
- Call next(error) for error propagation
- Use AppError and NotFoundError from errorHandler
- Validate numeric parameters

### 4. Business Logic Requirements
- Trim all string inputs
- Validate empty strings after trimming
- Convert string booleans to actual booleans
- Handle invalid number conversions
- Check resource existence before updates/deletes
- Provide descriptive error messages

## Validation Criteria
- All CRUD operations implemented
- Proper parameter validation and conversion
- Consistent response formats
- Error handling for all edge cases
- 404 errors for non-existent resources
- Input sanitization (trimming)
- Proper HTTP status codes
- All methods use async/await pattern
- Errors propagate to error middleware

## Important Notes
- Import Todo model from '../models/todo'
- Import error classes from '../middleware/errorHandler'
- Use async/await for cleaner code
- Always use try/catch blocks
- Don't send responses after calling next(error)
- Ensure completed field handles boolean conversion
- Follow RESTful conventions for status codes

## Testing the Controller
After implementation, verify:
1. getAllTodos handles filters correctly
2. Invalid parameters return 400 errors
3. Non-existent IDs return 404
4. Empty title validation works
5. Whitespace is trimmed from inputs
6. Update only modifies provided fields
7. Delete returns 204 with no content
8. All errors have proper status codes

## Expected Outcome
A complete controller implementation with:
- All CRUD operations functioning correctly
- Comprehensive error handling
- Input validation and sanitization
- Consistent response formatting
- Ready for route integration in Task 5
- Bonus statistics endpoint implemented

Execute all steps and ensure each controller method properly handles both success and error scenarios.