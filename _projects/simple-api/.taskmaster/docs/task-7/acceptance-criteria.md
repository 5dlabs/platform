# Task 7: Write Comprehensive Tests - Acceptance Criteria

## Overview

This document defines the acceptance criteria for Task 7: Write Comprehensive Tests. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Test Infrastructure Setup ✓

**Given** the need for isolated testing
**When** checking test configuration
**Then** the following must exist:

| File | Purpose | Requirements |
|------|---------|--------------|
| tests/setup.js | Global test setup | Set NODE_ENV=test, configure timeouts |
| tests/testDb.js | Test database | In-memory SQLite, reset helpers |
| jest.config.js | Jest configuration | 90% coverage thresholds, proper paths |

**Test**:
```bash
# Verify files exist
ls tests/setup.js tests/testDb.js jest.config.js
# All files should exist
```

### 2. Code Coverage Requirements ✓

**Given** the PRD requires 90% coverage
**When** running tests with coverage
**Then** coverage must meet thresholds:

```javascript
coverageThreshold: {
  global: {
    branches: 90,
    functions: 90,
    lines: 90,
    statements: 90
  }
}
```

**Test**:
```bash
npm test
# Should show coverage summary meeting all thresholds
```

### 3. Unit Tests - Todo Model ✓

**Given** the Todo model from Task 2
**When** checking tests/unit/models/todo.test.js
**Then** it must test:

| Method | Test Cases |
|--------|------------|
| findAll | No filters, with filters, pagination, empty DB |
| findById | Existing ID, non-existent ID, invalid ID |
| create | Valid data, auto-increment, null description |
| update | Full update, partial update, non-existent |
| delete | Successful delete, non-existent, verify removal |

**Test Coverage**: Each method should have 100% coverage

### 4. Unit Tests - Controllers ✓

**Given** controllers from Task 4
**When** checking tests/unit/controllers/todoController.test.js
**Then** it must test:

| Controller Method | Test Requirements |
|-------------------|-------------------|
| getAllTodos | Mock Todo.findAll, test query parsing, error handling |
| getTodoById | Mock responses, test 404, parameter parsing |
| createTodo | Input trimming, response format, status 201 |
| updateTodo | Partial updates, 404 handling, field selection |
| deleteTodo | Success (204), failure (404), mock verification |

**Mock Requirements**:
- Todo model must be mocked
- req, res, next objects properly mocked
- Verify correct methods called with correct args

### 5. Unit Tests - Middleware ✓

**Given** validation middleware from Task 3
**When** checking tests/unit/middleware/validation.test.js
**Then** it must verify:
- All validation rule arrays exist
- Each operation has validation rules
- Validation handler included in chains

**Note**: Detailed validation testing happens in integration tests

### 6. Integration Tests ✓

**Given** the complete API
**When** checking tests/integration/todos.test.js
**Then** it must test all endpoints:

#### GET /api/todos
- [ ] Empty database returns empty array
- [ ] Multiple todos returned correctly
- [ ] Completed filter works
- [ ] Pagination works (limit/offset)
- [ ] Invalid query parameters rejected

#### POST /api/todos
- [ ] Valid todo creation (201)
- [ ] Missing title rejected (400)
- [ ] Title too long rejected (400)
- [ ] Description optional
- [ ] Response includes created todo

#### GET /api/todos/:id
- [ ] Existing todo returned (200)
- [ ] Non-existent returns 404
- [ ] Invalid ID format rejected (400)

#### PUT /api/todos/:id
- [ ] Full update works
- [ ] Partial update works
- [ ] Non-existent returns 404
- [ ] Validation enforced

#### DELETE /api/todos/:id
- [ ] Successful deletion (204)
- [ ] Non-existent returns 404
- [ ] Verify todo actually deleted

#### Error Handling
- [ ] Unknown routes return 404
- [ ] Malformed JSON rejected
- [ ] Request ID included in errors

### 7. Health Check Tests ✓

**Given** health endpoints
**When** testing /api/health
**Then** verify:
- Returns 200 with healthy status
- Database connection tested
- Proper response format
- Timestamp included

### 8. Test Organization ✓

**Given** the test suite
**When** reviewing structure
**Then** tests must be organized as:

```
tests/
├── setup.js
├── testDb.js
├── unit/
│   ├── models/
│   │   └── todo.test.js
│   ├── controllers/
│   │   └── todoController.test.js
│   └── middleware/
│       └── validation.test.js
└── integration/
    └── todos.test.js
```

### 9. Test Quality Standards ✓

**Given** test code
**When** reviewing implementation
**Then** tests must:
- Use descriptive test names
- Follow AAA pattern (Arrange, Act, Assert)
- Clean up after themselves
- Run independently
- Test both success and failure paths
- Include edge cases

**Example Test Name**:
```javascript
test('should return 404 when todo does not exist', async () => {
  // Clear what is being tested
});
```

### 10. Test Execution Performance ✓

**Given** the complete test suite
**When** running tests
**Then** performance targets:
- Unit tests complete in < 5 seconds
- Integration tests complete in < 10 seconds
- Total test suite < 15 seconds
- No test timeouts

**Test**:
```bash
time npm test
# Should complete within time limits
```

## Test Scenarios Validation

### Scenario 1: Fresh Test Run
```bash
# Clean install and test
rm -rf node_modules coverage
npm install
npm test
# All tests pass, coverage meets thresholds
```

### Scenario 2: Test Isolation
```bash
# Run single test multiple times
npm test -- tests/unit/models/todo.test.js
npm test -- tests/unit/models/todo.test.js
# Same results each time
```

### Scenario 3: Coverage Report
```bash
npm run test:coverage
open coverage/index.html
# Visual coverage report shows gaps
```

## Mock Verification Examples

### Controller Mock Pattern
```javascript
// Arrange
const mockTodos = [{ id: 1, title: 'Test' }];
Todo.findAll.mockReturnValue(mockTodos);

// Act
todoController.getAllTodos(req, res, next);

// Assert
expect(Todo.findAll).toHaveBeenCalledWith(expectedFilters);
expect(res.json).toHaveBeenCalledWith(expectedResponse);
```

### Error Handling Pattern
```javascript
// Arrange
Todo.findById.mockImplementation(() => {
  throw new Error('DB Error');
});

// Act
todoController.getTodoById(req, res, next);

// Assert
expect(next).toHaveBeenCalledWith(expect.any(Error));
expect(res.json).not.toHaveBeenCalled();
```

## Definition of Done

- [ ] All test files created and organized properly
- [ ] 90% code coverage achieved (all metrics)
- [ ] All tests pass successfully
- [ ] Unit tests mock dependencies correctly
- [ ] Integration tests use test database
- [ ] Database reset between tests
- [ ] No test pollution or dependencies
- [ ] Edge cases and errors tested
- [ ] Test execution time within limits
- [ ] Coverage report generated
- [ ] No console errors during tests
- [ ] Tests can run in any order

## Coverage Exclusions

Acceptable exclusions from coverage:
- Swagger documentation files
- Test setup files
- Simple getters/setters
- Unreachable defensive code

## Notes

- Use `npm run test:watch` during development
- Check coverage/index.html for detailed reports
- Focus on testing behavior, not implementation
- Ensure mocks are cleared between tests
- Write tests that serve as documentation
- Prioritize testing critical paths first