# Task 5: Implement API Routes - Autonomous Prompt

You are tasked with implementing the routing layer for a Simple Todo REST API. Routes define the URL structure and connect HTTP requests to the appropriate controllers through validation middleware.

## Your Mission

Create Express routes that define all API endpoints, apply validation middleware, and delegate to the appropriate controller functions. Follow RESTful design principles and ensure all routes are properly validated.

## Required Actions

1. **Create Todo Routes (`src/routes/todoRoutes.js`)**
   
   Implement these RESTful routes:
   
   ```
   GET    /api/todos          → List all todos (with filters)
   POST   /api/todos          → Create new todo
   GET    /api/todos/:id      → Get specific todo
   PUT    /api/todos/:id      → Update todo
   DELETE /api/todos/:id      → Delete todo
   GET    /api/todos/stats/summary → Get statistics
   ```
   
   For each route:
   - Use appropriate HTTP method
   - Apply validation middleware from Task 3
   - Call corresponding controller method from Task 4
   - Maintain consistent URL patterns

2. **Create Health Routes (`src/routes/healthRoutes.js`)**
   
   Implement health check endpoints:
   - `GET /api/health` - Main health check
   - `GET /api/health/ready` - Readiness probe
   - `GET /api/health/live` - Liveness probe

3. **Create Root Router (`src/routes/index.js`)**
   
   - Create main router to organize all route modules
   - Mount todo routes at `/todos`
   - Mount health routes at `/health`
   - Add root endpoint with API information

4. **Update Express App (`src/app.js`)**
   
   Integrate routes into the application:
   - Import and mount routes at `/api`
   - Add root `/` endpoint with welcome message
   - Ensure routes are added after body parsers
   - Ensure routes are added before error handlers

5. **Create Route Test Script**
   
   Create `test-routes.sh` to verify all routes work:
   - Test each CRUD operation
   - Test validation errors
   - Test query parameters
   - Test health endpoints

## Route Specifications

### Todo Routes

| Method | Path | Validation | Controller Method |
|--------|------|------------|-------------------|
| GET | /api/todos | todoValidation.list | getAllTodos |
| POST | /api/todos | todoValidation.create | createTodo |
| GET | /api/todos/:id | todoValidation.getOne | getTodoById |
| PUT | /api/todos/:id | todoValidation.update | updateTodo |
| DELETE | /api/todos/:id | todoValidation.delete | deleteTodo |
| GET | /api/todos/stats/summary | none | getTodoStats |

### Important Route Patterns

```javascript
// Basic structure for each route
router.method(
  path,
  validationMiddleware,  // From Task 3
  controllerMethod       // From Task 4
);

// Example:
router.get('/', todoValidation.list, todoController.getAllTodos);
```

## Success Verification

Your routes should:

- [ ] Follow RESTful conventions
- [ ] Use correct HTTP methods
- [ ] Apply validation before controllers
- [ ] Return proper status codes (via controllers)
- [ ] Handle route parameters correctly
- [ ] Be mounted at correct base paths
- [ ] Work with the test script

## Testing Commands

```bash
# Start server
npm run dev

# Test routes (after creating test script)
chmod +x test-routes.sh
./test-routes.sh

# Manual tests
# Create todo
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Todo"}'

# List todos
curl http://localhost:3000/api/todos

# Get single todo
curl http://localhost:3000/api/todos/1

# Update todo  
curl -X PUT http://localhost:3000/api/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Delete todo
curl -X DELETE http://localhost:3000/api/todos/1

# Health check
curl http://localhost:3000/api/health
```

## Important Notes

- Routes should be thin - just connect validation to controllers
- Don't implement business logic in routes
- Route order matters - specific routes before generic ones
- All routes should go through validation (except health/stats)
- Use Express Router for modular organization
- Maintain consistent URL patterns

## Common Pitfalls to Avoid

1. **Wrong route order**: `/todos/:id` must come after `/todos/stats/summary`
2. **Missing validation**: Every route that accepts input needs validation
3. **Forgetting exports**: Each route file must export its router
4. **Wrong mounting**: Routes must be mounted at correct base paths
5. **Missing middleware**: Validation must come before controller

## File Structure After Completion

```
src/
├── routes/
│   ├── index.js        # Main router
│   ├── todoRoutes.js   # Todo CRUD routes  
│   └── healthRoutes.js # Health check routes
├── controllers/        # From Task 4
├── middleware/         # From Task 3
└── models/            # From Task 2
```

## Route Documentation Comments

Add JSDoc comments above each route:
```javascript
/**
 * @route   GET /api/todos
 * @desc    Get all todos with optional filters
 * @access  Public
 */
```

Once complete, your API will have fully functional endpoints ready for testing and documentation. The routes provide the URL structure that clients will use to interact with the todo service.