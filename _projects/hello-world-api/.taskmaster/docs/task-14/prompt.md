# Autonomous AI Agent Prompt: Manual Testing of API Endpoints

## Task Overview
You need to perform comprehensive manual testing of all implemented API endpoints. Test using curl commands, create a Postman collection for organized testing, verify all responses match expectations, and document the results.

## Detailed Instructions

### Step 1: Start the Server
```bash
cd hello-world-api
npm start
```
Verify server starts on port 3000.

### Step 2: Test Root Endpoint

**Test 2.1: Successful GET Request**
```bash
curl -i http://localhost:3000/
```
Verify:
- Status: 200 OK
- Content-Type: application/json
- Body: `{"message":"Hello, World!"}`

**Test 2.2: Invalid Methods**
```bash
curl -X POST -i http://localhost:3000/
curl -X PUT -i http://localhost:3000/
curl -X DELETE -i http://localhost:3000/
curl -X PATCH -i http://localhost:3000/
```
Verify all return 404 Not Found.

### Step 3: Test Health Endpoint

**Test 3.1: Successful GET Request**
```bash
curl -i http://localhost:3000/health
```
Verify:
- Status: 200 OK
- Body contains "status": "healthy"
- Valid ISO timestamp present

**Test 3.2: Timestamp Freshness**
```bash
# Test timestamps are different
curl -s http://localhost:3000/health | grep timestamp
sleep 2
curl -s http://localhost:3000/health | grep timestamp
```
Verify timestamps are different.

**Test 3.3: Invalid Methods**
```bash
curl -X POST -i http://localhost:3000/health
```
Verify returns 404.

### Step 4: Test Error Handling

**Test 4.1: Various 404 Scenarios**
```bash
curl -i http://localhost:3000/api
curl -i http://localhost:3000/users
curl -i http://localhost:3000/api/v1/test
curl -i http://localhost:3000/Health  # Case sensitive
```
Verify all return:
- Status: 404 Not Found
- Body: `{"error":"Not Found"}`

**Test 4.2: Special Characters**
```bash
curl -i "http://localhost:3000/test%20space"
curl -i "http://localhost:3000/../../etc/passwd"
```
Verify proper 404 handling.

### Step 5: Verify Request Logging

Check server console for log entries:
```
2024-01-20T10:30:00.000Z - GET /
2024-01-20T10:30:01.000Z - POST /
2024-01-20T10:30:02.000Z - GET /health
2024-01-20T10:30:03.000Z - GET /api
```

### Step 6: Create Postman Collection

**Collection Structure:**
```json
{
  "info": {
    "name": "Hello World API Tests",
    "description": "Complete test suite for Hello World API"
  },
  "variable": [
    {
      "key": "baseUrl",
      "value": "http://localhost:3000",
      "type": "string"
    }
  ],
  "item": [
    {
      "name": "Root Endpoint Tests",
      "item": [
        {
          "name": "GET / - Success",
          "request": {
            "method": "GET",
            "url": "{{baseUrl}}/"
          },
          "response": []
        },
        {
          "name": "POST / - 404 Error",
          "request": {
            "method": "POST",
            "url": "{{baseUrl}}/"
          }
        }
      ]
    },
    {
      "name": "Health Check Tests",
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
      "name": "Error Handling Tests",
      "item": [
        {
          "name": "GET /undefined - 404",
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

### Step 7: Create Test Report

Create `test-report.md`:
```markdown
# API Endpoint Test Report

## Test Information
- Date: [Current Date]
- Environment: Local Development
- Base URL: http://localhost:3000
- Tools Used: curl, Postman

## Test Summary
- Total Tests: 15
- Passed: X
- Failed: X
- Blocked: X

## Detailed Results

### 1. Root Endpoint (/)

| Test Case | Method | Expected | Actual | Status |
|-----------|--------|----------|--------|--------|
| Valid GET | GET | 200, Hello World | 200, Hello World | PASS |
| Invalid POST | POST | 404 | 404 | PASS |
| Invalid PUT | PUT | 404 | 404 | PASS |
| Invalid DELETE | DELETE | 404 | 404 | PASS |

### 2. Health Endpoint (/health)

| Test Case | Method | Expected | Actual | Status |
|-----------|--------|----------|--------|--------|
| Valid GET | GET | 200, healthy | 200, healthy | PASS |
| Timestamp Fresh | GET | Different times | Different | PASS |
| Invalid POST | POST | 404 | 404 | PASS |

### 3. Error Handling

| Test Case | Path | Expected | Actual | Status |
|-----------|------|----------|--------|--------|
| Undefined Route | /api | 404 | 404 | PASS |
| Deep Path | /api/v1/test | 404 | 404 | PASS |
| Case Sensitive | /Health | 404 | 404 | PASS |

### 4. Request Logging

- [x] All requests logged to console
- [x] Correct timestamp format
- [x] Method and path included

## Issues Found

[Document any issues]

## Recommendations

1. Consider adding request ID for tracking
2. Add response time to logs
3. Consider rate limiting for production

## Conclusion

All endpoints functioning as expected. API is ready for use.
```

## Expected Outcomes

### Successful Tests
- All endpoints return correct status codes
- Response formats match specifications
- Error handling works consistently
- Logging captures all requests

### Deliverables
1. Complete test execution results
2. Postman collection file
3. Test report document
4. List of any issues found

## Common Issues and Solutions

### Issue: Connection Refused
**Solution**: Ensure server is running on correct port

### Issue: Unexpected Response Format
**Solution**: Check implementation matches documentation

### Issue: Missing Logs
**Solution**: Verify logging middleware is properly configured

## Important Notes

### Testing Best Practices
- Test each endpoint thoroughly
- Include both positive and negative tests
- Verify exact response formats
- Check headers, not just body
- Test edge cases

### Documentation Requirements
- Record actual vs expected results
- Include response times if notable
- Screenshot any unusual behaviors
- Note platform-specific issues

### Postman Collection Tips
- Use environment variables for base URL
- Add tests to validate responses
- Include example responses
- Organize tests logically
- Export collection for sharing

## Validation Checklist

- [ ] Server starts successfully
- [ ] GET / returns Hello World
- [ ] GET /health returns status and timestamp
- [ ] Invalid routes return 404
- [ ] Wrong HTTP methods return 404
- [ ] All requests are logged
- [ ] No server crashes during testing
- [ ] Postman collection covers all cases
- [ ] Test report is complete
- [ ] Any issues are documented