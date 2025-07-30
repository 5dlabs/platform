# Task 5: Implement Comprehensive Testing Suite - Autonomous Agent Prompt

You are an experienced Node.js developer tasked with creating a comprehensive testing suite for the Hello World API. You need to implement unit tests, integration tests, and load tests to ensure the application works correctly and achieves >90% code coverage.

## Your Mission
Create a complete testing suite using Jest and Supertest, including configuration, unit tests for utilities, integration tests for all endpoints, and a load testing script to verify performance under concurrent load.

## Detailed Instructions

### 1. Create Jest Configuration (jest.config.js)
Create a configuration file in the root directory with these exact settings:

```javascript
module.exports = {
  testEnvironment: 'node',
  collectCoverage: true,
  coverageThreshold: {
    global: {
      branches: 90,
      functions: 90,
      lines: 90,
      statements: 90
    }
  },
  coverageReporters: ['text', 'lcov', 'clover'],
  testMatch: ['**/tests/**/*.test.js'],
  verbose: true
};
```

**Key Points:**
- Enforces 90% coverage threshold
- Generates multiple coverage report formats
- Only runs files matching *.test.js pattern
- Uses Node.js test environment

### 2. Create Unit Tests Directory Structure
Create the following directories:
- `tests/unit/`
- `tests/integration/`
- `tests/load/`

### 3. Implement Unit Tests (tests/unit/response.test.js)
Create comprehensive unit tests for the response utilities:

**Test Requirements:**
- Test the `success` function with and without data
- Test the `error` function with all parameter combinations
- Verify exact response format
- Check timestamp validity
- Test default values

**Test Cases to Include:**
1. Success response with data
2. Success response without data (null)
3. Error with custom status and data
4. Error with default status (400)
5. Error without data
6. Timestamp format validation

### 4. Implement Integration Tests (tests/integration/api.test.js)
Create thorough integration tests for all API endpoints:

**Required Test Coverage:**

**Health Endpoint:**
- 200 status and correct response format
- Presence of correlation ID header

**Hello Endpoints:**
- Basic greeting returns correct message
- Personalized greeting with valid name
- Name sanitization (test with special characters)
- Empty name returns 400 error

**Echo Endpoint:**
- Echoes complex JSON objects
- Returns 400 for empty body
- Handles different data types

**Info Endpoint:**
- Returns all required fields
- Uptime is a valid format
- Version matches package.json

**Documentation Endpoints:**
- /docs returns HTML
- /docs.json returns valid OpenAPI spec
- Root redirect to /docs

**Error Handling:**
- 404 for undefined routes
- Correlation ID in all responses
- Custom correlation ID is preserved

### 5. Implement Load Test Script (tests/load/load.test.js)
Create a load testing script with these features:

**Requirements:**
- Send 100 concurrent requests to /health endpoint
- Track individual response times
- Calculate average response time
- Calculate success rate
- Display results summary
- Can be run standalone

**Implementation Details:**
- Use native Node.js http module
- Use Promise.all for concurrent execution
- Track timing for each request
- Handle errors gracefully
- Return structured results

### 6. Package.json Scripts Update
Ensure package.json has the test script configured:
```json
{
  "scripts": {
    "test": "jest --coverage"
  }
}
```

## Testing Guidelines

### Unit Test Best Practices
```javascript
// Use descriptive test names
it('should create a success response with the correct format', () => {
  // Arrange
  const message = 'Test success';
  const data = { key: 'value' };
  
  // Act
  const response = success(message, data);
  
  // Assert
  expect(response).toHaveProperty('status', 'success');
  expect(response).toHaveProperty('message', message);
  expect(response).toHaveProperty('data', data);
});
```

### Integration Test Best Practices
```javascript
// Test complete request/response cycle
it('should return 200 and healthy status', async () => {
  const res = await request(app).get('/health');
  
  expect(res.statusCode).toBe(200);
  expect(res.body).toMatchObject({
    status: 'success',
    message: expect.any(String),
    data: { status: 'up' },
    timestamp: expect.any(String)
  });
});
```

### Load Test Execution
```bash
# First, start the server in one terminal
npm run dev

# In another terminal, run the load test
node tests/load/load.test.js
```

## Expected Test Results

### Coverage Report Example
```
----------------------|---------|----------|---------|---------|-------------------
File                  | % Stmts | % Branch | % Funcs | % Lines | Uncovered Line #s 
----------------------|---------|----------|---------|---------|-------------------
All files            |   95.65 |    92.31 |     100 |   95.65 |                   
 src                 |     100 |      100 |     100 |     100 |                   
  app.js             |     100 |      100 |     100 |     100 |                   
  server.js          |     100 |      100 |     100 |     100 |                   
 src/routes          |   94.74 |       90 |     100 |   94.74 |                   
  echo.js            |     100 |      100 |     100 |     100 |                   
  health.js          |     100 |      100 |     100 |     100 |                   
  hello.js           |   88.89 |       80 |     100 |   88.89 | 15                
  index.js           |     100 |      100 |     100 |     100 |                   
  info.js            |     100 |      100 |     100 |     100 |                   
 src/utils           |     100 |      100 |     100 |     100 |                   
  response.js        |     100 |      100 |     100 |     100 |                   
----------------------|---------|----------|---------|---------|-------------------
```

### Load Test Output Example
```
Running load test with 100 concurrent requests
Load test completed in 523ms
Average response time: 45.23ms
Success rate: 100.00%
```

## Common Testing Issues

### Issue 1: Tests fail due to port conflicts
**Solution:** Don't start the server in tests, use the app directly with supertest

### Issue 2: Coverage doesn't meet threshold
**Solution:** Add more test cases for uncovered branches, especially error paths

### Issue 3: Load test connection errors
**Solution:** Ensure server is running before executing load test

### Issue 4: Async test timeouts
**Solution:** Use proper async/await syntax and ensure promises resolve

## Validation Steps
1. Run `npm test` and verify all tests pass
2. Check coverage report shows >90% for all metrics
3. Run load test separately and verify 100% success rate
4. Verify average response time is <100ms
5. Check that test files follow naming convention

Complete this task by implementing all test files exactly as specified. The testing suite should provide confidence that the API works correctly under various conditions.