# Task 14 Prompt: Manual Testing of API Endpoints

## Testing Assignment

Perform comprehensive manual testing of all implemented API endpoints to verify they work correctly according to specifications.

## Testing Requirements

### 1. Test GET / Endpoint
- Send GET request to root endpoint
- Verify response: `{"message": "Hello, World!"}`
- Confirm status code is 200
- Check Content-Type header is `application/json`
- Test with wrong HTTP methods (POST, PUT, DELETE) to ensure they're rejected

### 2. Test GET /health Endpoint
- Send GET request to /health
- Verify response contains:
  - `status` field with value "healthy"
  - `timestamp` field with valid ISO 8601 timestamp
- Confirm status code is 200
- Make multiple requests and verify timestamp changes
- Validate timestamp format

### 3. Test Error Handling
- Access undefined routes (e.g., /api/test, /users, /invalid)
- Verify 404 status code is returned
- Check error response format
- Ensure consistent error structure across different invalid routes

### 4. Verify Request Logging
- Monitor console output during testing
- Confirm each request is logged with:
  - Timestamp
  - HTTP method
  - Request path
- Verify log format consistency

## Testing Tools

Use one or more of these tools:

### Option 1: curl Commands
```bash
# Test root endpoint
curl -v http://localhost:3000/

# Test health endpoint
curl -v http://localhost:3000/health

# Test 404 error
curl -v http://localhost:3000/nonexistent
```

### Option 2: Postman
- Create a new collection called "Hello World API Tests"
- Add requests for each endpoint
- Save expected responses
- Use Postman's test features to validate responses

### Option 3: Browser Testing
- Navigate to endpoints in a web browser
- Use developer tools to inspect responses
- Check network tab for headers and status codes

## Documentation Requirements

### 1. Create Test Results Summary
Document the following for each test:
- Test description
- Expected result
- Actual result
- Pass/Fail status
- Any issues found

### 2. Capture Evidence
- Take screenshots of successful responses
- Document any error messages
- Save console logs showing request logging

### 3. Issue Reporting
If any issues are found:
- Describe the issue clearly
- Provide steps to reproduce
- Include expected vs actual behavior
- Suggest priority level

## Test Execution Steps

1. **Preparation**
   - Start the server with `npm start`
   - Verify server is running on port 3000
   - Open testing tool of choice

2. **Execute Tests**
   - Run through all test cases systematically
   - Document results as you go
   - Take screenshots for evidence

3. **Analysis**
   - Review all test results
   - Identify any patterns in failures
   - Determine if API meets requirements

4. **Reporting**
   - Compile comprehensive test report
   - Include recommendations
   - Export Postman collection if used

## Expected Outcome

A complete test report confirming:
- All endpoints work as specified
- Error handling is properly implemented
- Request logging is functional
- API is ready for use

Or, if issues are found:
- Clear documentation of problems
- Reproducible test cases
- Recommendations for fixes