# Task 14: Manual Testing of API Endpoints

## Overview
This task involves comprehensive manual testing of all implemented API endpoints to verify functionality, validate responses, and ensure proper error handling. The testing will be performed using tools like curl, Postman, or web browsers, with results documented for future reference.

## Purpose and Objectives
- Manually test all API endpoints for correct functionality
- Verify response formats and status codes
- Test error handling and edge cases
- Validate request logging functionality
- Document test results and findings
- Create reusable test artifacts (Postman collection)
- Identify any issues or improvements needed

## Technical Approach

### Testing Strategy
1. **Systematic Testing**: Test each endpoint methodically
2. **Multiple Tools**: Use curl, Postman, and browsers
3. **Edge Cases**: Test various HTTP methods and invalid routes
4. **Documentation**: Record all test results
5. **Reproducibility**: Create Postman collection for future testing

### Test Coverage Areas
- Happy path functionality
- Error scenarios
- Response format validation
- Status code verification
- Header validation
- Logging verification

## Implementation Details

### Test Plan

#### 1. Environment Setup
- Start server: `npm start`
- Base URL: `http://localhost:3000`
- Tools: curl, Postman, web browser

#### 2. Root Endpoint Tests

**Test Case 1.1: GET / Success**
```bash
curl -i http://localhost:3000/
```
Expected:
- Status: 200 OK
- Body: `{"message":"Hello, World!"}`
- Content-Type: application/json

**Test Case 1.2: Wrong Methods**
```bash
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/
curl -X DELETE http://localhost:3000/
```
Expected: 404 Not Found

#### 3. Health Endpoint Tests

**Test Case 2.1: GET /health Success**
```bash
curl -i http://localhost:3000/health
```
Expected:
- Status: 200 OK
- Body contains: "status": "healthy"
- Valid ISO timestamp

**Test Case 2.2: Timestamp Validation**
```bash
# Run multiple times
for i in {1..3}; do
  curl -s http://localhost:3000/health | jq '.timestamp'
  sleep 1
done
```
Expected: Different timestamps

#### 4. Error Handling Tests

**Test Case 3.1: 404 Errors**
```bash
curl -i http://localhost:3000/nonexistent
curl -i http://localhost:3000/api/v1/users
curl -i http://localhost:3000/test
```
Expected:
- Status: 404 Not Found
- Body: `{"error":"Not Found"}`

**Test Case 3.2: Case Sensitivity**
```bash
curl -i http://localhost:3000/Health
curl -i http://localhost:3000/HEALTH
```
Expected: 404 (routes are case-sensitive)

#### 5. Logging Tests

**Test Case 4.1: Request Logging**
- Make various requests
- Check server console output
- Verify format: `TIMESTAMP - METHOD PATH`

### Postman Collection Structure
```json
{
  "info": {
    "name": "Hello World API Tests",
    "description": "Manual test collection for API endpoints"
  },
  "variable": [
    {
      "key": "baseUrl",
      "value": "http://localhost:3000"
    }
  ],
  "item": [
    {
      "name": "Root Endpoint",
      "item": [
        {
          "name": "GET / - Success",
          "request": {
            "method": "GET",
            "url": "{{baseUrl}}/"
          },
          "test": "pm.test('Status is 200', () => pm.response.to.have.status(200));"
        }
      ]
    },
    {
      "name": "Health Check",
      "item": [
        {
          "name": "GET /health - Success",
          "request": {
            "method": "GET",
            "url": "{{baseUrl}}/health"
          }
        }
      ]
    },
    {
      "name": "Error Handling",
      "item": [
        {
          "name": "404 - Undefined Route",
          "request": {
            "method": "GET",
            "url": "{{baseUrl}}/undefined"
          }
        }
      ]
    }
  ]
}
```

### Test Report Template
```markdown
# API Test Report

Date: [Test Date]
Tester: [Name]
Environment: Local Development

## Summary
- Total Endpoints Tested: 2
- Tests Passed: X
- Tests Failed: X
- Issues Found: X

## Test Results

### 1. Root Endpoint (/)
- [x] GET request returns 200
- [x] Response format correct
- [x] Content-Type is JSON
- [x] Non-GET methods return 404

### 2. Health Endpoint (/health)
- [x] GET request returns 200
- [x] Status field present
- [x] Timestamp is valid ISO
- [x] Timestamp updates

### 3. Error Handling
- [x] Undefined routes return 404
- [x] Error format consistent
- [x] No stack traces exposed

### 4. Request Logging
- [x] All requests logged
- [x] Log format correct
- [x] Timestamps included

## Issues Found
[List any issues]

## Recommendations
[List any improvements]
```

## Dependencies and Requirements

### Prerequisites
- Completed Tasks 9, 10, 11: All endpoints implemented
- Server is running and accessible
- Testing tools available (curl, Postman)

### Testing Tools
- **curl**: Command-line HTTP client
- **Postman**: GUI API testing tool
- **jq**: JSON parsing tool (optional)
- **Web Browser**: For basic GET requests

## Testing Strategy

### Manual Test Execution
1. **Preparation**
   - Start server
   - Open terminal for curl
   - Launch Postman
   - Open server logs

2. **Execution Order**
   - Test happy paths first
   - Test error cases
   - Test edge cases
   - Verify logging throughout

3. **Documentation**
   - Screenshot important results
   - Note any deviations
   - Record response times
   - Document any errors

### Success Criteria
- ✅ All endpoints return expected responses
- ✅ Status codes are correct
- ✅ Response formats match documentation
- ✅ Error handling works properly
- ✅ Request logging is functional
- ✅ No server crashes during testing
- ✅ Postman collection created

## Related Tasks
- **Previous**: Tasks 9, 10, 11 - Implementation
- **Previous**: Task 13 - README documentation
- **Validates**: All implementation tasks

## Notes and Considerations
- Test both positive and negative scenarios
- Include performance observations
- Note any security concerns
- Test with different tools to ensure consistency
- Save Postman collection for regression testing
- Consider automating these tests in the future
- Document any platform-specific behaviors