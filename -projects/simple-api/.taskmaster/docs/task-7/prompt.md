# Task 7: Write Comprehensive Tests - Autonomous Prompt

You are an AI agent tasked with implementing a comprehensive test suite for a Simple Todo REST API. Your goal is to achieve at least 90% code coverage while ensuring all critical functionality is thoroughly tested.

## Context
- **Project**: Simple Todo REST API
- **Prerequisites**: Tasks 2-5 must be completed (models, middleware, controllers, routes)
- **Testing Framework**: Jest with supertest for integration tests
- **Target Coverage**: Minimum 90% across all metrics
- **Working Directory**: Project root (simple-api/)
- **References**:
  - Architecture: .taskmaster/docs/architecture.md (Testing Strategy section)
  - Requirements: .taskmaster/docs/prd.txt (Success Criteria)

## Your Mission

Create a complete test suite including unit tests for models and controllers, integration tests for API endpoints, middleware tests, and proper test configuration. Ensure all edge cases are covered and the test suite is maintainable.

## Detailed Implementation Steps

1. **Create Test Structure**
   ```bash
   mkdir -p tests/unit/{models,controllers,middleware}
   mkdir -p tests/integration
   mkdir -p tests/fixtures
   mkdir -p tests/setup
   ```

2. **Create Test Database Setup** (`tests/setup/testDb.js`)
   - Create in-memory SQLite database for testing
   - Replicate production schema exactly
   - Include all constraints and triggers
   - Export function to create fresh database

3. **Create Jest Setup** (`tests/setup/jest.setup.js`)
   - Set NODE_ENV to 'test'
   - Configure test timeout
   - Add global test utilities
   - Create helper functions for common test data

4. **Implement Model Tests** (`tests/unit/models/todo.test.js`)
   - Mock the database module to use test database
   - Test all CRUD operations:
     - create: Valid data, missing title, exceeding length
     - findAll: No filters, with filters, pagination
     - findById: Existing, non-existent
     - update: Single field, multiple fields, non-existent
     - delete: Existing, non-existent
     - deleteAll: With data, empty database
     - count: All, filtered
   - Test data type conversions (boolean handling)
   - Test constraint violations

5. **Implement Controller Tests** (`tests/unit/controllers/todoController.test.js`)
   - Mock the Todo model completely
   - Test each controller method:
     - getAllTodos: Success, with filters, error handling
     - getTodoById: Found, not found, error handling
     - createTodo: Success, constraint error
     - updateTodo: Success, not found, partial updates
     - deleteTodo: Success, not found
     - getTodoStats: With data, empty database
   - Verify correct status codes
   - Verify error enhancement

6. **Implement Integration Tests** (`tests/integration/todos.test.js`)
   - Use supertest with the Express app
   - Mock database with in-memory version
   - Test complete request flows:
     - GET /api/todos with all query combinations
     - GET /api/todos/:id with valid/invalid IDs
     - POST /api/todos with valid/invalid data
     - PUT /api/todos/:id with partial/full updates
     - DELETE /api/todos/:id
     - GET /api/todos/stats
   - Test validation errors
   - Test health endpoints
   - Test 404 handling

7. **Implement Middleware Tests** (`tests/unit/middleware/validation.test.js`)
   - Test handleValidationErrors function
   - Verify validation rules exist
   - Test error formatting
   - Mock express-validator results

8. **Configure Jest** (`jest.config.js`)
   - Set test environment to 'node'
   - Configure coverage collection
   - Set coverage thresholds (90%)
   - Define test patterns
   - Configure setup files

## Test Implementation Patterns

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
    test('should handle success case', () => {
      // Arrange
      // Act
      // Assert
    });

    test('should handle error case', () => {
      // Test error scenarios
    });
  });
});
```

### Integration Test Pattern
```javascript
const response = await request(app)
  .post('/api/todos')
  .send({ title: 'Test' })
  .expect(201);

expect(response.body).toMatchObject({
  id: expect.any(Number),
  title: 'Test'
});
```

## Critical Test Scenarios

### Model Tests
- ✅ All CRUD operations work correctly
- ✅ Constraints are enforced
- ✅ Boolean conversion works
- ✅ Timestamps update correctly
- ✅ Pagination and filtering

### Controller Tests
- ✅ Correct HTTP status codes
- ✅ Error objects have status and code
- ✅ Only provided fields are updated
- ✅ Null checks work correctly

### Integration Tests
- ✅ Validation rejects invalid input
- ✅ 404 returned for missing resources
- ✅ Database changes persist
- ✅ Error responses match schema

## Success Criteria
- ✅ All tests pass consistently
- ✅ Coverage meets 90% threshold
- ✅ Unit tests are isolated (mocked dependencies)
- ✅ Integration tests use test database
- ✅ Edge cases are covered
- ✅ Error scenarios are tested
- ✅ Tests run quickly (< 30 seconds)
- ✅ Test output is clear and helpful

## Running Tests
```bash
# Run all tests with coverage
npm test

# Run only unit tests
npm run test:unit

# Run only integration tests
npm run test:integration

# Run tests in watch mode
npm run test:watch

# View coverage report
open coverage/index.html
```

## Common Pitfalls to Avoid
1. Not mocking external dependencies in unit tests
2. Using real database in tests
3. Not testing error scenarios
4. Forgetting to test edge cases
5. Tests depending on execution order
6. Not cleaning up between tests

## Coverage Goals
```
File                | % Stmts | % Branch | % Funcs | % Lines |
--------------------|---------|----------|---------|---------|
All files           |      90 |       90 |      90 |      90 |
 models/            |      95 |       95 |      95 |      95 |
 controllers/       |      90 |       90 |      90 |      90 |
 middleware/        |      85 |       85 |      85 |      85 |
 routes/            |      90 |       90 |      90 |      90 |
```

Remember: Tests are documentation. They should clearly show how the system works and what it expects. Write tests that help future developers understand the code.