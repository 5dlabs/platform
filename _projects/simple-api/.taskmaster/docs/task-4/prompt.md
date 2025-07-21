# Task 4: Implement Todo Controller - Autonomous Prompt

You are tasked with implementing the controller layer for a Simple Todo REST API. Controllers handle the business logic between the models (database) and the routes (HTTP layer).

## Your Mission

Create controller functions that process HTTP requests, interact with the Todo model, and return appropriate responses. Each controller should handle one specific operation following REST principles and maintain consistent error handling.

## Required Actions

1. **Create Main Todo Controller (`src/controllers/todoController.js`)**
   
   Implement these controller methods:
   
   - **getAllTodos**: 
     - Accept query parameters: completed (string 'true'/'false'), limit (number), offset (number)
     - Convert string parameters to appropriate types
     - Call Todo.findAll() with filters
     - Return todos with count and applied filters
   
   - **getTodoById**:
     - Parse ID from request params
     - Call Todo.findById()
     - Return 404 if not found
     - Return todo data if found
   
   - **createTodo**:
     - Extract title and description from request body
     - Trim whitespace from strings
     - Call Todo.create()
     - Return 201 status with created todo
   
   - **updateTodo**:
     - Parse ID from params
     - Extract update fields from body (only include provided fields)
     - Call Todo.update()
     - Return 404 if not found
     - Return updated todo if successful
   
   - **deleteTodo**:
     - Parse ID from params
     - Call Todo.delete()
     - Return 404 if not found
     - Return 204 No Content if successful

2. **Create Controller Utilities (`src/controllers/utils.js`)**
   
   Helper functions:
   - `formatTodo(todo)`: Ensure consistent todo format
   - `formatTodos(todos)`: Format array of todos
   - `createError(message, status)`: Create error with HTTP status
   - `parseBoolean(value)`: Convert 'true'/'false' strings to boolean

3. **Create Health Controller (`src/controllers/healthController.js`)**
   
   - Test database connection
   - Return service status
   - Include timestamp, version, environment
   - Return 503 if database is unreachable

## Implementation Requirements

### Error Handling Pattern
```javascript
try {
  // Controller logic
} catch (err) {
  next(err); // Pass to Express error handler
}
```

### Response Formats

**Success Response (List)**:
```json
{
  "data": [...],
  "count": 10,
  "filters": { "completed": true, "limit": 10, "offset": 0 }
}
```

**Success Response (Single)**:
```json
{
  "data": { "id": 1, "title": "...", ... }
}
```

**Success Response (Create/Update)**:
```json
{
  "message": "Todo created successfully",
  "data": { ... }
}
```

### Important Patterns

1. **Always use next(err)** to pass errors to Express error handler
2. **Parse numeric IDs**: `parseInt(req.params.id, 10)`
3. **Trim string inputs**: `title.trim()`
4. **Handle optional fields**: Check if undefined before including in updates
5. **Create custom errors**: Include status code for 404s

## Success Verification

Your implementation should:

- [ ] Export an object with all required methods
- [ ] Handle all CRUD operations correctly
- [ ] Return appropriate HTTP status codes (200, 201, 204, 404)
- [ ] Pass all errors to next() function
- [ ] Parse and validate input parameters
- [ ] Format responses consistently
- [ ] Handle edge cases (non-existent IDs, invalid input)
- [ ] Never send multiple responses for one request

## Testing Your Implementation

```javascript
// Quick test (before routes are implemented)
const todoController = require('./src/controllers/todoController');

// Check methods exist
console.log('Methods:', Object.keys(todoController));
// Should show: ['getAllTodos', 'getTodoById', 'createTodo', 'updateTodo', 'deleteTodo']

// Mock request/response for testing
const mockReq = {
  params: { id: '1' },
  query: { completed: 'true', limit: '10' },
  body: { title: 'Test Todo' }
};

const mockRes = {
  json: (data) => console.log('Response:', data),
  status: (code) => ({
    json: (data) => console.log(`Status ${code}:`, data),
    end: () => console.log(`Status ${code}: No Content`)
  })
};

const mockNext = (err) => console.log('Error:', err.message);
```

## Important Notes

- Do NOT implement routes (that's Task 5)
- Do NOT modify the Todo model (from Task 2)
- Focus only on controller logic
- Assume validation has been done by middleware
- Use the Todo model methods exactly as defined
- All database errors should bubble up through next()

## Context

Controllers are the bridge between:
- **Models** (Todo database operations from Task 2)
- **Routes** (HTTP endpoints to be created in Task 5)

They should:
- Process request data
- Call appropriate model methods
- Format responses
- Handle errors gracefully
- Never contain SQL or database logic
- Never contain route definitions

## Common Pitfalls to Avoid

1. **Forgetting to parse IDs**: Always use parseInt for numeric IDs
2. **Not handling 404s**: Check if Todo operations return null
3. **Multiple responses**: Use return after sending response
4. **Sync errors**: Always wrap in try/catch
5. **Missing next parameter**: Controllers need (req, res, next)

Once complete, your controllers will be ready to be connected to routes in Task 5, providing full CRUD functionality for the Todo API.