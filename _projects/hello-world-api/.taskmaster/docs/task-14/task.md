# Task 14: Manual Testing of API Endpoints

## Overview
**Title**: Manual Testing of API Endpoints  
**Status**: pending  
**Priority**: high  
**Dependencies**: Task 9 (Root Endpoint), Task 10 (Health Check), Task 11 (Error Handling)  

## Description
Perform manual testing of all implemented API endpoints to verify functionality. This task ensures that all endpoints work as expected, error handling functions properly, and the API meets its specifications through comprehensive manual testing.

## Technical Approach

### 1. Testing Methodology
- Create structured test cases
- Use multiple testing tools (curl, Postman, browser)
- Document all test results
- Verify both success and error scenarios

### 2. Test Coverage
- All implemented endpoints
- Various HTTP methods
- Error conditions
- Edge cases
- Performance characteristics

### 3. Documentation
- Test cases and expected results
- Actual results and screenshots
- Issues discovered
- Recommendations for improvement

## Implementation Details

### Test Suite Structure

#### 1. Endpoint Tests
```bash
# Root Endpoint Tests
GET /
- Expected: 200 OK, {"message": "Hello, World!"}
- Content-Type: application/json
- Response time: < 50ms

# Health Check Tests  
GET /health
- Expected: 200 OK, {"status": "healthy", "timestamp": "<ISO>"}
- Timestamp validation
- Response time: < 50ms
```

#### 2. Error Handling Tests
```bash
# 404 Tests
GET /undefined
- Expected: 404 Not Found, {"error": "Not Found"}

POST /
- Expected: 404 Not Found

# Server Error Tests (if test route added)
GET /test-error
- Expected: 500 Internal Server Error
- Error logged to console
```

#### 3. Request Logging Tests
```
All requests should log:
- Timestamp (ISO format)
- HTTP method
- Request path
- No sensitive data
```

### Testing Tools

#### 1. cURL Commands
```bash
# Test root endpoint
curl -v http://localhost:3000/

# Test health endpoint
curl -v http://localhost:3000/health

# Test 404 handling
curl -v http://localhost:3000/undefined

# Test different methods
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/health
curl -X DELETE http://localhost:3000/
```

#### 2. Postman Collection
```json
{
  "info": {
    "name": "Hello World API Tests",
    "description": "Test collection for API endpoints"
  },
  "item": [
    {
      "name": "Root Endpoint",
      "request": {
        "method": "GET",
        "url": "{{baseUrl}}/"
      }
    },
    {
      "name": "Health Check",
      "request": {
        "method": "GET",
        "url": "{{baseUrl}}/health"
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

#### 3. Browser Testing
- Navigate to http://localhost:3000/
- Navigate to http://localhost:3000/health
- Check developer console for errors
- Verify JSON rendering

## Subtasks Breakdown

### 1. Set up Postman Collection for API Testing
- **Status**: pending
- **Dependencies**: None
- **Actions**:
  - Install/open Postman
  - Create new collection
  - Set up environment variables
  - Organize test structure

### 2. Test Root Endpoint Functionality
- **Status**: pending
- **Dependencies**: Subtask 1
- **Test Cases**:
  - GET / returns correct response
  - Status code is 200
  - Content-Type is application/json
  - Response format matches spec

### 3. Test Health Check Endpoint Functionality
- **Status**: pending
- **Dependencies**: Subtask 1
- **Test Cases**:
  - GET /health returns correct response
  - Timestamp is valid ISO format
  - Timestamp updates on each request
  - Status field is "healthy"

### 4. Test Error Handling and 404 Responses
- **Status**: pending
- **Dependencies**: Subtask 1
- **Test Cases**:
  - Undefined routes return 404
  - Wrong HTTP methods return 404
  - Error format is consistent
  - No stack traces exposed

### 5. Verify Request Logging and Create Test Report
- **Status**: pending
- **Dependencies**: All subtasks
- **Deliverables**:
  - Console log verification
  - Test report document
  - Postman collection export
  - Recommendations

## Test Cases

### Test Case Template
```markdown
### Test ID: TC001
**Description**: Verify root endpoint returns Hello World message
**Endpoint**: GET /
**Prerequisites**: Server running on port 3000

**Steps**:
1. Send GET request to http://localhost:3000/
2. Verify response status
3. Verify response body
4. Check response headers

**Expected Result**:
- Status: 200 OK
- Body: {"message": "Hello, World!"}
- Content-Type: application/json

**Actual Result**: [To be filled during testing]
**Status**: [Pass/Fail]
**Notes**: [Any observations]
```

### Complete Test Suite

#### TC001: Root Endpoint - Success
- **Method**: GET
- **Path**: /
- **Expected Status**: 200
- **Expected Body**: {"message": "Hello, World!"}

#### TC002: Health Check - Success
- **Method**: GET
- **Path**: /health
- **Expected Status**: 200
- **Expected Body**: {"status": "healthy", "timestamp": "<valid ISO>"}

#### TC003: Undefined Route - 404
- **Method**: GET
- **Path**: /undefined
- **Expected Status**: 404
- **Expected Body**: {"error": "Not Found"}

#### TC004: Wrong Method on Root
- **Method**: POST
- **Path**: /
- **Expected Status**: 404
- **Expected Body**: {"error": "Not Found"}

#### TC005: Request Logging
- **Action**: Make any request
- **Expected**: Console shows timestamp, method, path

## Test Report Template

```markdown
# API Testing Report

## Summary
- **Date**: [Test date]
- **Tester**: [Name]
- **Environment**: Local development
- **Base URL**: http://localhost:3000

## Test Results Summary
- Total Tests: X
- Passed: X
- Failed: X
- Blocked: X

## Detailed Results
[Test case results]

## Issues Found
1. [Issue description]
   - Severity: [High/Medium/Low]
   - Steps to reproduce
   - Expected vs Actual

## Recommendations
1. [Improvement suggestions]

## Conclusion
[Overall assessment]
```

## Common Testing Scenarios

### Performance Testing
```bash
# Simple load test
for i in {1..100}; do
  curl -s -o /dev/null -w "%{http_code} %{time_total}\n" http://localhost:3000/
done
```

### Concurrent Request Testing
```bash
# Test concurrent requests
for i in {1..10}; do
  curl http://localhost:3000/ &
done
wait
```

### Header Validation
```bash
# Check all headers
curl -I http://localhost:3000/
```

## Expected Issues and Solutions

### Issue: Port not accessible
**Check**: Is server running?
**Solution**: Start server with `npm start`

### Issue: JSON parsing errors
**Check**: Response format
**Solution**: Verify Content-Type header

### Issue: Timestamp validation
**Check**: ISO format regex
**Solution**: Ensure timezone included

## Quality Criteria

### Test Completeness
- All endpoints tested
- All HTTP methods verified
- Error scenarios covered
- Edge cases considered

### Documentation Quality
- Clear test descriptions
- Reproducible steps
- Accurate results
- Actionable findings

## Next Steps
After completing this task:
- All endpoints verified working
- Issues documented for fixing
- Test suite available for regression testing
- Confidence in API functionality
- Ready for deployment or further development

The manual testing provides confidence that the API functions correctly and meets all requirements.