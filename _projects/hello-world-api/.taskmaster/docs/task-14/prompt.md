# Autonomous Agent Prompt: Manual Testing of API Endpoints

## Context
You are tasked with performing comprehensive manual testing of the Hello World API. All endpoints have been implemented including the root endpoint, health check, and error handling. Now you need to verify that everything works as expected through systematic testing.

## Objective
Execute manual tests for all API endpoints, verify their functionality, document test results, and identify any issues or improvements needed. Create a structured test report with findings and recommendations.

## Task Requirements

### 1. Testing Scope
Test the following:
- **GET /**: Root endpoint returning Hello World message
- **GET /health**: Health check endpoint with status and timestamp
- **404 Handling**: Undefined routes returning proper errors
- **Method Handling**: Wrong HTTP methods returning 404
- **Request Logging**: All requests logged to console
- **Error Responses**: Consistent error format

### 2. Testing Tools
Use multiple approaches:
- **cURL**: Command-line testing with verbose output
- **Browser**: Visual verification of responses
- **Postman**: Organized test collection (if available)
- **Console**: Monitor server logs during testing

### 3. Documentation Requirements
- Test case descriptions
- Expected vs actual results
- Screenshots or output samples
- Issues discovered
- Performance observations
- Recommendations

## Test Execution Plan

### Phase 1: Setup
```bash
# 1. Start the server
npm start

# 2. Verify server is running
# Should see: "Server running on port 3000"

# 3. Open new terminal for testing
```

### Phase 2: Endpoint Testing

#### Test Case 1: Root Endpoint
```bash
# Basic test
curl http://localhost:3000/
# Expected: {"message":"Hello, World!"}

# Verbose test
curl -v http://localhost:3000/
# Check: Status 200, Content-Type: application/json

# Headers only
curl -I http://localhost:3000/
# Verify: HTTP/1.1 200 OK
```

#### Test Case 2: Health Check
```bash
# Basic test
curl http://localhost:3000/health
# Expected: {"status":"healthy","timestamp":"2024-01-15T..."}

# Multiple requests to verify timestamp changes
curl http://localhost:3000/health && echo && sleep 1 && curl http://localhost:3000/health
# Timestamps should be different

# Validate ISO format
curl http://localhost:3000/health | python -m json.tool
# Should format nicely if valid JSON
```

#### Test Case 3: 404 Handling
```bash
# Undefined route
curl http://localhost:3000/undefined
# Expected: {"error":"Not Found"} with 404 status

# Nested undefined route
curl http://localhost:3000/api/v1/users
# Expected: {"error":"Not Found"}

# Check status code
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/undefined
# Expected: 404
```

#### Test Case 4: Method Handling
```bash
# POST to root
curl -X POST http://localhost:3000/
# Expected: {"error":"Not Found"}

# PUT to health
curl -X PUT http://localhost:3000/health
# Expected: {"error":"Not Found"}

# DELETE to root
curl -X DELETE http://localhost:3000/
# Expected: {"error":"Not Found"}

# OPTIONS request
curl -X OPTIONS http://localhost:3000/
# Expected: {"error":"Not Found"}
```

#### Test Case 5: Request Logging
```bash
# Monitor server console while making requests
# Each request should log:
# 2024-01-15T10:30:45.123Z - GET /
# 2024-01-15T10:30:46.456Z - GET /health
# 2024-01-15T10:30:47.789Z - GET /undefined
```

### Phase 3: Edge Cases

```bash
# Query parameters
curl "http://localhost:3000/?test=123"
# Should still return Hello World, params ignored

# Special characters in path
curl "http://localhost:3000/test%20space"
# Expected: 404

# Multiple slashes
curl http://localhost:3000//health
# May work or 404 depending on Express handling

# Case sensitivity
curl http://localhost:3000/HEALTH
# Expected: 404 (routes are case-sensitive)
```

### Phase 4: Performance Testing

```bash
# Response time measurement
time curl http://localhost:3000/
# Should be < 50ms

# Multiple sequential requests
for i in {1..10}; do
  curl -s -o /dev/null -w "Request %{http_code} in %{time_total}s\n" http://localhost:3000/
done

# Concurrent requests
for i in {1..5}; do curl http://localhost:3000/ & done; wait
# All should succeed
```

## Test Report Format

```markdown
# Hello World API - Manual Testing Report

## Test Environment
- **Date**: [Current date]
- **Server**: http://localhost:3000
- **Node Version**: [Version]
- **Test Tools**: cURL, Browser

## Test Summary
| Test Category | Total | Passed | Failed |
|--------------|-------|--------|--------|
| Endpoints    | 2     | X      | X      |
| Error Cases  | 5     | X      | X      |
| Performance  | 3     | X      | X      |

## Detailed Test Results

### 1. Endpoint Tests

#### TC001: GET / - Root Endpoint
- **Status**: PASS/FAIL
- **Expected**: {"message":"Hello, World!"}
- **Actual**: [Actual response]
- **Notes**: [Any observations]

#### TC002: GET /health - Health Check
- **Status**: PASS/FAIL
- **Expected**: {"status":"healthy","timestamp":"<ISO>"}
- **Actual**: [Actual response]
- **Notes**: Timestamp updates correctly

### 2. Error Handling Tests

#### TC003: GET /undefined - 404 Response
- **Status**: PASS/FAIL
- **Expected**: 404 with {"error":"Not Found"}
- **Actual**: [Actual response]

### 3. Performance Tests

#### TC010: Response Time
- **Target**: < 50ms
- **Actual**: [Average time]
- **Status**: PASS/FAIL

## Issues Found

1. **Issue Title**
   - Severity: High/Medium/Low
   - Description: [Details]
   - Steps to Reproduce: [Steps]
   - Recommended Fix: [Suggestion]

## Recommendations

1. **Performance**: [Any performance suggestions]
2. **Security**: [Security observations]
3. **Usability**: [API usability feedback]
4. **Documentation**: [Documentation gaps]

## Conclusion

The API testing revealed [summary of findings]. Overall, the API [meets/does not meet] the functional requirements with [X] issues identified for resolution.
```

## Validation Checklist

- [ ] Server starts without errors
- [ ] Root endpoint returns correct JSON
- [ ] Health check shows valid timestamp
- [ ] 404 errors have consistent format
- [ ] All HTTP methods tested
- [ ] Request logging works
- [ ] No server crashes during testing
- [ ] Performance is acceptable
- [ ] Error messages don't expose sensitive data
- [ ] Console shows appropriate logs

## Important Notes

- Test in a clean environment
- Document exact commands used
- Include timestamps in reports
- Note any unexpected behaviors
- Test both success and failure paths
- Verify JSON formatting is correct
- Check for any security concerns

## Tools Required
- Terminal/Command line access
- cURL installed
- Web browser
- Text editor for report
- Server running locally

Proceed with systematic testing of all endpoints, documenting results carefully and creating a comprehensive test report that validates the API functionality and identifies any issues requiring attention.