# Task 5: Implement Comprehensive Testing Suite

## Overview
This task creates a comprehensive testing suite for the Hello World API, including unit tests, integration tests, and load tests. The goal is to achieve >90% code coverage while ensuring all functionality works correctly under various conditions, including concurrent load.

## Objectives
- Configure Jest for testing with coverage requirements
- Create unit tests for utility functions
- Implement integration tests for all API endpoints
- Develop load testing capability
- Achieve >90% code coverage across all metrics
- Ensure tests can run in CI/CD pipeline

## Technical Approach

### Testing Architecture
The testing suite consists of three layers:
1. **Unit Tests**: Isolated testing of individual functions and modules
2. **Integration Tests**: End-to-end testing of API endpoints
3. **Load Tests**: Performance and concurrency testing

### Testing Technologies
- **Jest**: Primary testing framework
- **Supertest**: HTTP assertion library for API testing
- **Native HTTP module**: Load testing implementation

### Coverage Requirements
- Branches: >90%
- Functions: >90%
- Lines: >90%
- Statements: >90%

## Implementation Details

### Step 1: Configure Jest (jest.config.js)
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

### Step 2: Unit Tests for Utilities (tests/unit/response.test.js)
```javascript
const { success, error } = require('../../src/utils/response');

describe('Response Utilities', () => {
  describe('success()', () => {
    it('should create a success response with the correct format', () => {
      const message = 'Test success';
      const data = { key: 'value' };
      const response = success(message, data);
      
      expect(response).toHaveProperty('status', 'success');
      expect(response).toHaveProperty('message', message);
      expect(response).toHaveProperty('data', data);
      expect(response).toHaveProperty('timestamp');
      expect(new Date(response.timestamp)).toBeInstanceOf(Date);
    });
    
    it('should handle null data', () => {
      const response = success('Test success');
      expect(response.data).toBeNull();
    });
  });
  
  describe('error()', () => {
    it('should create an error with the correct properties', () => {
      const message = 'Test error';
      const status = 404;
      const data = { reason: 'Not found' };
      
      const err = error(message, status, data);
      
      expect(err).toBeInstanceOf(Error);
      expect(err.message).toBe(message);
      expect(err.status).toBe(status);
      expect(err.data).toEqual(data);
    });
    
    it('should use default status 400 when not provided', () => {
      const err = error('Test error');
      expect(err.status).toBe(400);
    });
    
    it('should handle null data', () => {
      const err = error('Test error', 500);
      expect(err.data).toBeNull();
    });
  });
});
```

### Step 3: Integration Tests (tests/integration/api.test.js)
```javascript
const request = require('supertest');
const app = require('../../src/app');

describe('API Endpoints', () => {
  describe('GET /health', () => {
    it('should return 200 and healthy status', async () => {
      const res = await request(app).get('/health');
      
      expect(res.statusCode).toBe(200);
      expect(res.body).toHaveProperty('status', 'success');
      expect(res.body).toHaveProperty('data.status', 'up');
      expect(res.body).toHaveProperty('timestamp');
    });
  });
  
  describe('GET /hello', () => {
    it('should return 200 and hello world message', async () => {
      const res = await request(app).get('/hello');
      
      expect(res.statusCode).toBe(200);
      expect(res.body).toHaveProperty('status', 'success');
      expect(res.body).toHaveProperty('data.greeting', 'Hello, World!');
    });
  });
  
  describe('GET /hello/:name', () => {
    it('should return 200 and personalized greeting', async () => {
      const name = 'John';
      const res = await request(app).get(`/hello/${name}`);
      
      expect(res.statusCode).toBe(200);
      expect(res.body).toHaveProperty('status', 'success');
      expect(res.body).toHaveProperty('data.greeting', `Hello, ${name}!`);
    });
    
    it('should sanitize name parameter', async () => {
      const res = await request(app).get('/hello/<script>alert(1)</script>');
      
      expect(res.statusCode).toBe(200);
      expect(res.body.data.greeting).toBe('Hello, scriptalert1script!');
    });
    
    it('should return 400 for empty name parameter', async () => {
      const res = await request(app).get('/hello/ ');
      
      expect(res.statusCode).toBe(400);
      expect(res.body).toHaveProperty('status', 'error');
    });
  });
  
  describe('POST /echo', () => {
    it('should return 200 and echo the request body', async () => {
      const payload = { test: 'data', nested: { value: 123 } };
      const res = await request(app)
        .post('/echo')
        .send(payload)
        .set('Content-Type', 'application/json');
      
      expect(res.statusCode).toBe(200);
      expect(res.body).toHaveProperty('status', 'success');
      expect(res.body).toHaveProperty('data', payload);
    });
    
    it('should return 400 for empty request body', async () => {
      const res = await request(app)
        .post('/echo')
        .send({})
        .set('Content-Type', 'application/json');
      
      expect(res.statusCode).toBe(400);
      expect(res.body).toHaveProperty('status', 'error');
    });
  });
  
  describe('GET /info', () => {
    it('should return 200 and service information', async () => {
      const res = await request(app).get('/info');
      
      expect(res.statusCode).toBe(200);
      expect(res.body).toHaveProperty('status', 'success');
      expect(res.body.data).toHaveProperty('version');
      expect(res.body.data).toHaveProperty('uptime');
      expect(res.body.data).toHaveProperty('environment');
    });
  });
  
  describe('GET /docs', () => {
    it('should return 200 and HTML content', async () => {
      const res = await request(app).get('/docs');
      
      expect(res.statusCode).toBe(200);
      expect(res.headers['content-type']).toMatch(/html/);
    });
  });
  
  describe('GET /docs.json', () => {
    it('should return 200 and valid OpenAPI spec', async () => {
      const res = await request(app).get('/docs.json');
      
      expect(res.statusCode).toBe(200);
      expect(res.headers['content-type']).toMatch(/json/);
      expect(res.body).toHaveProperty('openapi');
      expect(res.body).toHaveProperty('paths');
    });
  });
  
  describe('Error handling', () => {
    it('should return 404 for non-existent routes', async () => {
      const res = await request(app).get('/not-found');
      
      expect(res.statusCode).toBe(404);
      expect(res.body).toHaveProperty('status', 'error');
      expect(res.body).toHaveProperty('message', 'Not Found');
    });
    
    it('should include correlation ID in response headers', async () => {
      const res = await request(app).get('/health');
      
      expect(res.headers).toHaveProperty('x-correlation-id');
      expect(res.headers['x-correlation-id']).toBeTruthy();
    });
    
    it('should use provided correlation ID if available', async () => {
      const correlationId = 'test-correlation-id';
      const res = await request(app)
        .get('/health')
        .set('x-correlation-id', correlationId);
      
      expect(res.headers['x-correlation-id']).toBe(correlationId);
    });
  });
});
```

### Step 4: Load Test Script (tests/load/load.test.js)
```javascript
const http = require('http');

// Simple load test to verify concurrent request handling
const runLoadTest = async () => {
  const CONCURRENT_REQUESTS = 100;
  const TARGET = 'http://localhost:3000/health';
  
  console.log(`Running load test with ${CONCURRENT_REQUESTS} concurrent requests`);
  
  const startTime = Date.now();
  const promises = [];
  
  for (let i = 0; i < CONCURRENT_REQUESTS; i++) {
    promises.push(
      new Promise((resolve, reject) => {
        const req = http.get(TARGET, (res) => {
          let data = '';
          res.on('data', (chunk) => { data += chunk; });
          res.on('end', () => {
            resolve({
              statusCode: res.statusCode,
              responseTime: Date.now() - startTime
            });
          });
        });
        
        req.on('error', reject);
      })
    );
  }
  
  const results = await Promise.all(promises);
  const totalTime = Date.now() - startTime;
  
  const avgResponseTime = results.reduce((sum, result) => sum + result.responseTime, 0) / results.length;
  const successRate = results.filter(r => r.statusCode === 200).length / results.length * 100;
  
  console.log(`Load test completed in ${totalTime}ms`);
  console.log(`Average response time: ${avgResponseTime.toFixed(2)}ms`);
  console.log(`Success rate: ${successRate.toFixed(2)}%`);
  
  return {
    totalTime,
    avgResponseTime,
    successRate,
    results
  };
};

// Can be run directly: node tests/load/load.test.js
if (require.main === module) {
  runLoadTest().catch(console.error);
}

module.exports = { runLoadTest };
```

### Key Testing Patterns

#### Unit Test Coverage
- Test both success and error paths
- Test edge cases (null, empty, invalid inputs)
- Verify exact response formats
- Check default values

#### Integration Test Coverage
- Test all HTTP methods
- Verify status codes
- Check response body structure
- Test error scenarios
- Validate headers (correlation ID)
- Test input validation

#### Load Test Metrics
- Concurrent request handling
- Response time tracking
- Success rate calculation
- Performance under stress

## Dependencies and Requirements
- Tasks 3 and 4 must be completed (endpoints and documentation)
- Jest and Supertest must be installed
- All source files must exist for testing
- Server must be able to start for integration tests

## Testing Strategy

### Running Tests
```bash
# Run all tests with coverage
npm test

# Run only unit tests
npm test -- tests/unit

# Run only integration tests
npm test -- tests/integration

# Run load test (server must be running)
npm run dev &
node tests/load/load.test.js
```

### Coverage Analysis
```bash
# View coverage report
npm test -- --coverage

# Generate HTML coverage report
npm test -- --coverage --coverageReporters=html
# Open coverage/index.html in browser
```

### CI/CD Integration
```yaml
# Example GitHub Actions workflow
- name: Run tests
  run: |
    npm test
    if [ $? -ne 0 ]; then
      echo "Tests failed"
      exit 1
    fi
```

## Success Criteria
- All tests pass successfully
- Code coverage exceeds 90% for all metrics
- Unit tests execute in < 1 second
- Integration tests execute in < 5 seconds
- Load test shows 100% success rate
- Average response time < 100ms under load
- Tests run successfully in CI/CD pipeline

## Common Testing Scenarios

### Testing Error Handling
```javascript
// Test middleware error handling
it('should handle thrown errors', async () => {
  // Temporarily add error-throwing route
  app.get('/test-error', (req, res, next) => {
    next(new Error('Test error'));
  });
  
  const res = await request(app).get('/test-error');
  expect(res.statusCode).toBe(500);
  expect(res.body.status).toBe('error');
});
```

### Testing Edge Cases
```javascript
// Test various input types
it('should handle different data types in echo', async () => {
  const testCases = [
    { input: null, expectedStatus: 400 },
    { input: 'string', expectedStatus: 200 },
    { input: 123, expectedStatus: 200 },
    { input: [1, 2, 3], expectedStatus: 200 },
    { input: { nested: { deep: true } }, expectedStatus: 200 }
  ];
  
  for (const testCase of testCases) {
    const res = await request(app)
      .post('/echo')
      .send(testCase.input);
    expect(res.statusCode).toBe(testCase.expectedStatus);
  }
});
```

## Related Tasks
- Task 3: API Endpoints (provides functionality to test)
- Task 4: API Documentation (documentation endpoints to test)
- Task 6: ESLint Configuration (ensures code quality for tests)
- Task 7: Containerization (tests must work in container)
- Task 8: Kubernetes Deployment (tests validate deployment readiness)