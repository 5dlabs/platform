# Task 7: Write Comprehensive Tests - Autonomous Prompt

You are an AI agent tasked with implementing a comprehensive test suite for the Simple Todo REST API. Your goal is to write unit and integration tests that achieve minimum 90% code coverage while thoroughly testing all functionality.

## Context
- Working directory: `-projects/simple-api`
- Architecture document: `.taskmaster/docs/architecture.md`
- Product requirements: `.taskmaster/docs/prd.txt`
- All implementation tasks (1-6) are complete
- Jest and supertest are installed as dev dependencies

## Your Mission
Create a complete test suite with unit tests for models and controllers, integration tests for API endpoints, and proper test infrastructure. Ensure all edge cases are covered and maintain minimum 90% code coverage across all metrics.

## Required Actions

### 1. Configure Jest
Create `jest.config.js` in root:
- Set test environment to 'node'
- Configure coverage collection from src/
- Set coverage thresholds to 90%
- Define test match patterns
- Set up test timeout
- Reference setup file

### 2. Create Test Setup
Create `tests/setup.js`:
- Mock console methods for cleaner output
- Set NODE_ENV to 'test'
- Set PORT to 0 for random port
- Configure any global test utilities

### 3. Create Test Helpers
Create `tests/helpers/testDb.js`:
- Function to create in-memory test database
- Test data fixtures for todos
- Function to seed test data
- Database reset utilities

### 4. Unit Tests - Models
Create `tests/unit/models/todo.test.js`:

Test Todo model methods:
- **create**: Valid todo, without description, constraint violations
- **findAll**: All todos, filtered by completed, with pagination
- **findById**: Existing todo, non-existent todo
- **update**: All fields, partial updates, non-existent todo
- **delete**: Existing todo, non-existent todo
- **count**: Total count, filtered count

Use beforeEach to reset database state.

### 5. Unit Tests - Controllers
Create `tests/unit/controllers/todoController.test.js`:

Mock the Todo model and test:
- **getAllTodos**: Success, with filters, invalid parameters
- **getTodoById**: Success, not found, invalid ID
- **createTodo**: Success, validation, trimming
- **updateTodo**: Success, not found, partial updates
- **deleteTodo**: Success, not found
- **getTodoStats**: Calculation accuracy

Mock req, res, next objects properly.

### 6. Integration Tests
Create `tests/integration/todos.test.js`:

Test full API flow:
- **GET /api/todos**: Empty list, with data, filtering, pagination
- **POST /api/todos**: Valid creation, validation errors
- **GET /api/todos/:id**: Existing, non-existent
- **PUT /api/todos/:id**: Valid update, not found, validation
- **DELETE /api/todos/:id**: Success, not found
- **GET /api/health**: Service health

Use supertest to make actual HTTP requests.

### 7. Additional Test Files
Create tests for:
- Middleware validation
- Error handling
- Health endpoints
- Edge cases and boundaries

### 8. Test Patterns

Unit test structure:
```javascript
describe('Component Name', () => {
  beforeEach(() => {
    // Setup
  });
  
  describe('method', () => {
    test('should do something', () => {
      // Arrange
      // Act
      // Assert
    });
  });
});
```

Integration test pattern:
```javascript
const response = await request(app)
  .get('/api/todos')
  .expect(200);
  
expect(response.body).toHaveProperty('data');
```

## Validation Criteria
- All tests pass successfully
- Coverage exceeds 90% for:
  - Statements
  - Branches
  - Functions
  - Lines
- Edge cases are tested
- Error scenarios covered
- Mock usage is appropriate
- Tests are maintainable
- No test interdependencies

## Important Notes
- Mock database for unit tests
- Use in-memory database for integration
- Reset state between tests
- Test both success and failure paths
- Verify error response formats
- Check status codes
- Validate response structures
- Test boundary conditions

## Test Execution Commands
Provide these npm scripts:
- `npm test` - Run all tests with coverage
- `npm run test:watch` - Watch mode
- `npm run test:unit` - Unit tests only
- `npm run test:integration` - Integration only

## Expected Test Coverage
Minimum 90% coverage for:
- Models: All CRUD operations
- Controllers: All methods and error paths
- Routes: All endpoints
- Middleware: Validation and errors
- Utilities: Any helper functions

## Testing Checklist
Ensure tests cover:
1. Happy path for all operations
2. Validation failures
3. Not found scenarios
4. Database constraints
5. Empty states
6. Pagination boundaries
7. Boolean conversions
8. String trimming
9. Concurrent operations
10. Error propagation

## Expected Outcome
A comprehensive test suite that:
- Validates all functionality
- Achieves >90% coverage
- Runs quickly and reliably
- Catches regressions
- Documents behavior through tests
- Provides confidence for deployment

Execute all test creation steps and run the full suite to ensure complete coverage and passing tests.