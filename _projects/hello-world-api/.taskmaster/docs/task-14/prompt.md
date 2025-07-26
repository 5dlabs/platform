# Autonomous Agent Prompt for Task 14: Manual Testing of API Endpoints

## Task Context
You need to perform comprehensive manual testing of all API endpoints to verify they work correctly. This includes testing successful responses, error handling, and logging functionality.

## Your Mission
Execute thorough manual tests for all API endpoints, verify responses match specifications, test error scenarios, and create a detailed test report documenting all findings.

## Step-by-Step Instructions

### 1. Start the Server
```bash
cd hello-world-api
npm start
# Server should start on port 3000
```

### 2. Test Root Endpoint

#### Using curl:
```bash
# Basic test
curl http://localhost:3000/

# Verbose test with headers
curl -v http://localhost:3000/

# Test with specific headers
curl -H "Accept: application/json" http://localhost:3000/

# Test response time
time curl http://localhost:3000/

# Pretty print JSON
curl -s http://localhost:3000/ | python3 -m json.tool
```

#### Expected Results:
- Status Code: 200
- Response Body: `{"message":"Hello, World!"}`
- Content-Type: application/json

#### Additional Tests:
```bash
# Test wrong methods (should return 404)
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/
curl -X DELETE http://localhost:3000/
```

### 3. Test Health Endpoint

#### Using curl:
```bash
# Basic test
curl http://localhost:3000/health

# Verify timestamp format
curl -s http://localhost:3000/health | python3 -c "
import json, sys
from datetime import datetime
data = json.load(sys.stdin)
print(f'Status: {data[\"status\"]}')
print(f'Timestamp: {data[\"timestamp\"]}')
# Validate ISO format
datetime.fromisoformat(data['timestamp'].replace('Z', '+00:00'))
print('✓ Valid ISO timestamp')
"

# Multiple requests to verify timestamp changes
for i in {1..3}; do
  echo "Request $i:"
  curl -s http://localhost:3000/health | grep timestamp
  sleep 1
done
```

#### Expected Results:
- Status Code: 200
- Response Body: `{"status":"healthy","timestamp":"<ISO-8601 timestamp>"}`
- Timestamp should update on each request

### 4. Test Error Handling

#### 404 Errors:
```bash
# Test various undefined routes
curl -v http://localhost:3000/undefined
curl -v http://localhost:3000/api/users
curl -v http://localhost:3000/test
curl -v http://localhost:3000/../../etc/passwd

# All should return 404 with error message
```

#### Expected 404 Response:
- Status Code: 404
- Response Body: `{"error":"Not Found"}`

### 5. Test Request Logging

#### Monitor Server Logs:
While running tests, observe the server console for:
```
2024-01-15T10:30:00.000Z - GET /
2024-01-15T10:30:01.000Z - GET /health
2024-01-15T10:30:02.000Z - GET /undefined
404 Not Found: GET /undefined
```

### 6. Create Test Report

Create `test-report.md`:

```markdown
# API Test Report

**Date:** [Current Date]
**Tester:** [Your Name]
**Environment:** Local Development

## Test Summary

| Endpoint | Method | Expected Status | Actual Status | Result |
|----------|---------|----------------|---------------|---------|
| / | GET | 200 | 200 | ✅ PASS |
| /health | GET | 200 | 200 | ✅ PASS |
| /undefined | GET | 404 | 404 | ✅ PASS |
| / | POST | 404 | 404 | ✅ PASS |

## Detailed Test Results

### 1. Root Endpoint (GET /)

**Request:**
```bash
curl http://localhost:3000/
```

**Response:**
```json
{
  "message": "Hello, World!"
}
```

**Status:** ✅ PASS
- Correct status code (200)
- Correct response format
- Proper Content-Type header

### 2. Health Endpoint (GET /health)

**Request:**
```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

**Status:** ✅ PASS
- Correct status code (200)
- Valid ISO timestamp
- Timestamp updates on each request

### 3. Error Handling (404)

**Request:**
```bash
curl http://localhost:3000/nonexistent
```

**Response:**
```json
{
  "error": "Not Found"
}
```

**Status:** ✅ PASS
- Correct status code (404)
- Consistent error format
- Works for all undefined routes

### 4. Request Logging

**Console Output:**
```
2024-01-15T10:30:00.000Z - GET /
2024-01-15T10:30:01.000Z - GET /health
2024-01-15T10:30:02.000Z - GET /nonexistent
```

**Status:** ✅ PASS
- All requests logged
- Timestamp included
- Method and path recorded

## Performance Metrics

| Endpoint | Average Response Time |
|----------|---------------------|
| GET / | < 10ms |
| GET /health | < 10ms |

## Issues Found

None - All tests passed successfully.

## Recommendations

1. Consider adding request ID for tracking
2. Implement rate limiting for production
3. Add more detailed health checks
4. Consider adding API versioning

## Conclusion

The API is functioning correctly according to specifications. All endpoints return expected responses with appropriate status codes. Error handling is consistent and logging is operational.
```

### 7. Postman Collection (Optional)

Create a Postman collection:

```json
{
  "info": {
    "name": "Hello World API",
    "description": "Test collection for Hello World API endpoints"
  },
  "item": [
    {
      "name": "Root Endpoint",
      "request": {
        "method": "GET",
        "url": "{{baseUrl}}/",
        "description": "Returns Hello World message"
      }
    },
    {
      "name": "Health Check",
      "request": {
        "method": "GET",
        "url": "{{baseUrl}}/health",
        "description": "Returns API health status"
      }
    },
    {
      "name": "404 Test",
      "request": {
        "method": "GET",
        "url": "{{baseUrl}}/undefined",
        "description": "Tests 404 error handling"
      }
    }
  ],
  "variable": [
    {
      "key": "baseUrl",
      "value": "http://localhost:3000"
    }
  ]
}
```

## Validation Steps

### 1. Verify All Tests Pass
```bash
# Run all curl commands and verify responses
# Check that all return expected status codes and bodies
```

### 2. Check Test Coverage
Ensure you've tested:
- All endpoints (/, /health)
- All error scenarios (404)
- Different HTTP methods
- Request logging
- Response times

### 3. Document Any Issues
If any tests fail:
- Document the failure
- Include error messages
- Suggest fixes
- Re-test after fixes

## Expected Result
- All endpoints tested successfully
- Test report created with results
- No critical issues found
- API ready for use
- Documentation of any minor issues

## Important Notes
- Always start the server before testing
- Test in a clean environment
- Document actual vs expected results
- Include timestamps in test report
- Save curl commands for future use