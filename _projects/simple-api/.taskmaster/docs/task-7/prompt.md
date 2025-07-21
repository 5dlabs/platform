# Task 7: Write Comprehensive Tests - Autonomous Prompt

You are tasked with implementing a complete test suite for a Simple Todo REST API. The tests must achieve at least 90% code coverage and ensure all functionality works correctly.

## Your Mission

Create comprehensive unit and integration tests using Jest and Supertest. Tests should cover all models, controllers, middleware, and API endpoints with proper isolation and thorough edge case testing.

## Required Actions

1. **Set Up Test Infrastructure**
   
   Create test configuration:
   - `tests/setup.js` - Global test setup
   - `tests/testDb.js` - In-memory test database
   - Update `jest.config.js` - Configure Jest with coverage thresholds
   
   Key requirements:
   - Use in-memory SQLite for test isolation
   - Set 90% coverage threshold
   - Configure test environment variables
   - Mock console logs unless debugging

2. **Unit Tests for Todo Model (`tests/unit/models/todo.test.js`)**
   
   Test all model methods:
   - `findAll()` - Test with/without filters, pagination
   - `findById()` - Test existing/non-existing IDs
   - `create()` - Test valid/invalid data
   - `update()` - Test partial/full updates
   - `delete()` - Test successful/failed deletions
   
   Include edge cases:
   - Empty database
   - Invalid data types
   - Database errors
   - Boundary conditions

3. **Unit Tests for Controllers (`tests/unit/controllers/todoController.test.js`)**
   
   Mock the Todo model and test:
   - `getAllTodos` - Query parsing, error handling
   - `getTodoById` - Found/not found scenarios
   - `createTodo` - Input trimming, response format
   - `updateTodo` - Partial updates, 404 handling
   - `deleteTodo` - Success/failure cases
   
   Verify:
   - Correct model methods called
   - Proper status codes returned
   - Error passed to next()
   - Response formats

4. **Unit Tests for Middleware (`tests/unit/middleware/validation.test.js`)**
   
   Test validation rules:
   - All validation chains exist
   - Proper validation applied
   - Error formatting works
   
   Note: Full validation testing happens in integration tests

5. **Integration Tests (`tests/integration/todos.test.js`)**
   
   Test complete request/response cycles:
   
   **GET /api/todos**
   - Empty database
   - Multiple todos
   - Query filters (completed, limit, offset)
   - Validation errors
   
   **POST /api/todos**
   - Valid creation
   - Missing title
   - Title too long
   - With/without description
   
   **GET /api/todos/:id**
   - Existing todo
   - Non-existent todo
   - Invalid ID format
   
   **PUT /api/todos/:id**
   - Update all fields
   - Partial update
   - Non-existent todo
   - Validation errors
   
   **DELETE /api/todos/:id**
   - Successful deletion
   - Non-existent todo
   - Verify deletion
   
   **Error Handling**
   - 404 for unknown routes
   - Malformed JSON
   - Server errors

6. **Health Check Tests**
   
   Test health endpoints:
   - Main health check
   - Database connectivity
   - Response format

## Test Structure Requirements

### Unit Test Pattern
```javascript
describe('Component Name', () => {
  beforeEach(() => {
    // Setup
  });

  afterEach(() => {
    // Cleanup
  });

  describe('method name', () => {
    test('should do something when condition', () => {
      // Arrange
      // Act
      // Assert
    });
  });
});
```

### Integration Test Pattern
```javascript
const request = require('supertest');
const app = require('../../src/app');

describe('Endpoint', () => {
  test('should return expected response', async () => {
    const res = await request(app)
      .get('/api/endpoint')
      .expect(200);
    
    expect(res.body).toMatchObject({
      // Expected response
    });
  });
});
```

## Coverage Requirements

Achieve 90% coverage in:
- Statements
- Branches
- Functions
- Lines

Exclude from coverage:
- Documentation files (swagger)
- Test files themselves

## Success Verification

- [ ] All tests pass: `npm test`
- [ ] Coverage meets 90% threshold
- [ ] Unit tests run in < 5 seconds
- [ ] Integration tests run in < 10 seconds
- [ ] No test pollution between runs
- [ ] Mocks properly cleared
- [ ] Edge cases covered
- [ ] Error scenarios tested

## Important Testing Principles

1. **Isolation**: Each test should be independent
2. **Repeatability**: Tests should produce same results
3. **Fast**: Unit tests especially should be quick
4. **Clear**: Test names should describe what they test
5. **Complete**: Test both success and failure paths

## Common Testing Patterns

### Testing Async Errors
```javascript
test('should handle errors', async () => {
  Todo.findAll.mockImplementation(() => {
    throw new Error('Database error');
  });
  
  todoController.getAllTodos(req, res, next);
  
  expect(next).toHaveBeenCalledWith(
    expect.objectContaining({ message: 'Database error' })
  );
});
```

### Testing 404 Responses
```javascript
test('should return 404 when not found', async () => {
  const res = await request(app)
    .get('/api/todos/999')
    .expect(404);
  
  expect(res.body.error).toBe('Todo not found');
});
```

### Database Reset Pattern
```javascript
beforeEach(() => {
  resetDatabase();
});

afterEach(() => {
  jest.clearAllMocks();
});
```

## Notes

- Use beforeEach to reset state
- Mock external dependencies
- Test edge cases thoroughly
- Verify error handling paths
- Check coverage reports for gaps
- Write descriptive test names
- Group related tests with describe

Once complete, run `npm test` to verify all tests pass with > 90% coverage. The test suite ensures the API is reliable and maintains quality as it evolves.