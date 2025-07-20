# Task 4: Implement Todo Controller - Autonomous Prompt

You are an AI agent tasked with implementing the controller layer for a Simple Todo REST API. Controllers handle the business logic, coordinate with models, and format responses for the API endpoints.

## Context
- **Project**: Simple Todo REST API
- **Prerequisites**: 
  - Task 2 (Database/Model) must be completed
  - Task 3 (Express/Middleware) must be completed
- **Working Directory**: Project root (simple-api/)
- **Available Models**: Todo model with CRUD operations from Task 2
- **References**:
  - Architecture: .taskmaster/docs/architecture.md (see Controller Layer)
  - Requirements: .taskmaster/docs/prd.txt

## Your Mission

Create controller functions that handle todo CRUD operations by coordinating with the Todo model, processing requests, handling errors appropriately, and returning properly formatted responses with correct HTTP status codes.

## Detailed Implementation Steps

1. **Create Todo Controller** (`src/controllers/todoController.js`)
   - Import Todo model from '../models'
   - Implement controller object with these methods:
     - `getAllTodos`: List todos with filtering/pagination
     - `getTodoById`: Get single todo by ID
     - `createTodo`: Create new todo
     - `updateTodo`: Update existing todo
     - `deleteTodo`: Delete todo
     - `getTodoStats`: Get statistics (bonus feature)
   - Each method should:
     - Use try-catch for error handling
     - Extract parameters from req object
     - Call appropriate model methods
     - Return proper HTTP status codes
     - Pass errors to next() for middleware handling

2. **Create Health Controller** (`src/controllers/healthController.js`)
   - Import db from models for database checks
   - Implement two health check methods:
     - `getHealth`: Basic health check returning status, timestamp, uptime
     - `getDetailedHealth`: Include database connectivity check
   - Return appropriate status codes (200 for healthy, 503 for unhealthy)

3. **Create Controller Index** (`src/controllers/index.js`)
   - Export all controllers for convenient importing

## Controller Implementation Details

### getAllTodos
- Extract query params: completed, limit, offset
- Parse string values to appropriate types
- Apply defaults (limit: 100, offset: 0)
- Call Todo.findAll() with filters
- Return 200 with array of todos

### getTodoById
- Parse ID from params
- Call Todo.findById()
- If not found, create 404 error with code 'TODO_NOT_FOUND'
- Return 200 with todo object

### createTodo
- Extract title and description from body
- Call Todo.create()
- Handle SQLITE_CONSTRAINT errors → 400 Bad Request
- Return 201 with created todo

### updateTodo
- Parse ID from params
- Build updates object with only provided fields
- Call Todo.update()
- If not found, return 404
- Handle constraint errors → 400
- Return 200 with updated todo

### deleteTodo
- Parse ID from params
- Call Todo.delete()
- If not found (returns false), return 404
- Return 204 No Content on success

## Error Handling Pattern

```javascript
try {
  // Controller logic
} catch (error) {
  // Enhance error with status and code if needed
  if (error.code === 'SQLITE_CONSTRAINT') {
    error.status = 400;
    error.message = 'Invalid todo data';
  }
  next(error); // Pass to error middleware
}
```

## Response Format Standards

### Success Responses
- GET list: `200 OK` with array
- GET single: `200 OK` with object
- POST: `201 Created` with created object
- PUT: `200 OK` with updated object
- DELETE: `204 No Content` with empty body

### Error Responses (handled by middleware)
```javascript
{
  error: {
    message: "Todo not found",
    code: "TODO_NOT_FOUND"
  }
}
```

## Success Criteria
- ✅ All CRUD operations implemented
- ✅ Proper HTTP status codes used
- ✅ Errors passed to middleware with next()
- ✅ Not found resources return 404
- ✅ Database errors transformed to user-friendly messages
- ✅ Query parameters parsed correctly
- ✅ Only provided fields are updated
- ✅ Health check verifies database connectivity

## Testing Approach

Test controllers with mocked model:
```javascript
// Example test setup
const req = {
  params: { id: '1' },
  body: { title: 'Test' },
  query: { completed: 'true' }
};
const res = {
  status: jest.fn().mockReturnThis(),
  json: jest.fn(),
  end: jest.fn()
};
const next = jest.fn();

// Call controller method
todoController.getTodoById(req, res, next);
```

## Important Notes
- Controllers should NOT contain SQL or database logic
- Let validation middleware handle input validation
- Don't catch errors just to rethrow them
- Use consistent error codes across the API
- Parse all numeric parameters from strings
- Health endpoint should not require authentication

## Common Pitfalls to Avoid
1. Forgetting to parse string IDs to integers
2. Not checking if resource exists before operations
3. Catching errors without proper handling
4. Returning wrong status codes
5. Including undefined fields in updates

## Expected File Structure
```
simple-api/
└── src/
    └── controllers/
        ├── todoController.js
        ├── healthController.js
        └── index.js
```

Remember: Controllers orchestrate the application logic. Keep them focused on coordinating between the request/response cycle and the business logic in models.