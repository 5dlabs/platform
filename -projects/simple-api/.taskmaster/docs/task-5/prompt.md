# Task 5: Implement API Routes - Autonomous Prompt

You are an AI agent tasked with implementing the routing layer for a Simple Todo REST API. You will create Express routes that connect HTTP endpoints to controller functions with proper validation middleware.

## Context
- **Project**: Simple Todo REST API
- **Prerequisites**: 
  - Task 3 (Express/Middleware) must be completed
  - Task 4 (Controllers) must be completed
- **Working Directory**: Project root (simple-api/)
- **Available Components**:
  - Controllers: todoController, healthController
  - Middleware: todoValidation, handleValidationErrors
- **References**:
  - Architecture: .taskmaster/docs/architecture.md (see Routes Layer)
  - Requirements: .taskmaster/docs/prd.txt (see API Endpoints)

## Your Mission

Create RESTful routes that properly connect HTTP endpoints to controller functions, applying validation middleware where appropriate. Follow REST conventions and ensure all endpoints from the PRD are implemented.

## Detailed Implementation Steps

1. **Create Todo Routes** (`src/routes/todos.js`)
   - Import express and create router
   - Import todoController from '../controllers'
   - Import validation middleware from '../middleware'
   - Implement routes in this order (order matters!):
     - GET / - List todos (with list validation)
     - GET /stats - Todo statistics (before /:id to avoid conflict)
     - GET /:id - Get single todo (with getOne validation)
     - POST / - Create todo (with create validation)
     - PUT /:id - Update todo (with update validation)
     - DELETE /:id - Delete todo (with delete validation)
   - Apply validation middleware before controller functions
   - Export the router

2. **Create Health Routes** (`src/routes/health.js`)
   - Import express and create router
   - Import healthController from '../controllers'
   - Implement routes:
     - GET / - Basic health check
     - GET /detailed - Detailed health with database check
   - No validation needed for health endpoints
   - Export the router

3. **Create Routes Index** (`src/routes/index.js`)
   - Create main router
   - Import todo and health routes
   - Mount routes with proper prefixes:
     - /todos → todo routes
     - /health → health routes
   - Add root endpoint (GET /) that returns API info
   - Export the main router

4. **Update Application** (Update `src/app.js`)
   - Import routes from './routes'
   - Mount all routes under '/api' prefix
   - Ensure routes are added BEFORE 404 handler
   - Ensure error handler remains last

## Route Implementation Pattern

```javascript
// Example route with validation
router.get(
  '/:id',
  todoValidation.getOne,      // Validation rules
  handleValidationErrors,     // Check validation result
  todoController.getTodoById  // Controller function
);
```

## REST Endpoint Mapping

| Method | Path | Controller Method | Validation |
|--------|------|------------------|------------|
| GET | /api/todos | getAllTodos | list |
| GET | /api/todos/stats | getTodoStats | none |
| GET | /api/todos/:id | getTodoById | getOne |
| POST | /api/todos | createTodo | create |
| PUT | /api/todos/:id | updateTodo | update |
| DELETE | /api/todos/:id | deleteTodo | delete |
| GET | /api/health | getHealth | none |
| GET | /api/health/detailed | getDetailedHealth | none |

## Important Route Considerations

1. **Route Order**: Place specific routes before parameter routes
   - `/stats` must come before `/:id`
   - Otherwise `/stats` will match as an ID

2. **Middleware Chain**: Always in this order:
   - Validation rules (if needed)
   - Validation error handler (if validation used)
   - Controller function

3. **API Structure**:
   ```
   /api
   ├── / (API info)
   ├── /todos
   │   ├── GET /
   │   ├── GET /stats
   │   ├── GET /:id
   │   ├── POST /
   │   ├── PUT /:id
   │   └── DELETE /:id
   └── /health
       ├── GET /
       └── GET /detailed
   ```

## Success Criteria
- ✅ All endpoints from PRD implemented
- ✅ Proper REST conventions followed
- ✅ Validation applied to user inputs
- ✅ Routes mounted with /api prefix
- ✅ Route conflicts avoided (order matters)
- ✅ Health endpoints accessible
- ✅ API info endpoint at /api
- ✅ All routes properly exported

## Testing Your Routes

After implementation, test with curl:
```bash
# Test API root
curl http://localhost:3000/api

# Test todo listing
curl http://localhost:3000/api/todos

# Test todo creation
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Test Todo"}'

# Test health check
curl http://localhost:3000/api/health
```

## Common Pitfalls to Avoid
1. Wrong route order causing conflicts
2. Missing validation middleware
3. Forgetting handleValidationErrors after validation
4. Not exporting routers
5. Mounting routes after error handlers

## Expected File Structure
```
simple-api/
└── src/
    └── routes/
        ├── todos.js
        ├── health.js
        └── index.js
```

## Notes
- Routes define the API contract - keep them stable
- Use proper HTTP methods for operations
- Let controllers handle business logic
- Routes should be thin - just wiring
- Document routes with comments for clarity

Remember: Routes are the API's public interface. Make them intuitive, consistent, and well-organized.