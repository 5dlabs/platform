# Acceptance Criteria for Task 14: Manual Testing of API Endpoints

## Required Outcomes

### 1. Test Environment Setup
- [ ] Server is running successfully
- [ ] Testing tools are available (curl/Postman)
- [ ] Console logging is visible
- [ ] Port 3000 is accessible

### 2. Root Endpoint Testing
- [ ] GET / returns 200 status code
- [ ] Response body is `{"message":"Hello, World!"}`
- [ ] Content-Type header is application/json
- [ ] Response time is under 100ms
- [ ] Wrong HTTP methods return 404

### 3. Health Endpoint Testing
- [ ] GET /health returns 200 status code
- [ ] Response contains "status" field with value "healthy"
- [ ] Response contains "timestamp" field
- [ ] Timestamp is valid ISO 8601 format
- [ ] Timestamp updates on subsequent requests
- [ ] Response time is under 100ms

### 4. Error Handling Testing
- [ ] Undefined routes return 404 status code
- [ ] 404 response body is `{"error":"Not Found"}`
- [ ] Multiple undefined routes tested
- [ ] Path traversal attempts handled safely
- [ ] Consistent error format across all errors

### 5. Request Logging Verification
- [ ] All requests appear in console logs
- [ ] Log format includes timestamp
- [ ] Log includes HTTP method
- [ ] Log includes request path
- [ ] 404 errors are logged appropriately

### 6. Test Documentation
- [ ] Test report created
- [ ] All test cases documented
- [ ] Actual vs expected results recorded
- [ ] Screenshots/output included where relevant
- [ ] Issues and recommendations documented

## Test Cases

### Test Case 1: Root Endpoint Success
```bash
curl -v http://localhost:3000/
```
**Expected:**
- Status: 200 OK
- Body: `{"message":"Hello, World!"}`
- Headers: Content-Type: application/json

### Test Case 2: Health Endpoint Success
```bash
curl -v http://localhost:3000/health
```
**Expected:**
- Status: 200 OK
- Body contains: `"status":"healthy"`
- Body contains valid ISO timestamp

### Test Case 3: 404 Error Handling
```bash
curl -v http://localhost:3000/nonexistent
```
**Expected:**
- Status: 404 Not Found
- Body: `{"error":"Not Found"}`

### Test Case 4: Method Not Allowed
```bash
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/health
```
**Expected:**
- Status: 404 Not Found
- Body: `{"error":"Not Found"}`

### Test Case 5: Multiple Requests
```bash
for i in {1..10}; do curl http://localhost:3000/; done
```
**Expected:**
- All requests succeed
- Consistent responses
- Server remains stable

### Test Case 6: Concurrent Requests
```bash
for i in {1..10}; do curl http://localhost:3000/ & done
```
**Expected:**
- All requests handled
- No errors or timeouts
- Proper logging for all

## Performance Criteria
- [ ] Root endpoint responds in < 50ms
- [ ] Health endpoint responds in < 50ms
- [ ] 404 responses in < 50ms
- [ ] No memory leaks during testing
- [ ] CPU usage remains reasonable

## Test Report Requirements

### Report Structure
- [ ] Executive summary
- [ ] Test environment details
- [ ] Test case results table
- [ ] Detailed findings for each endpoint
- [ ] Performance metrics
- [ ] Issues discovered
- [ ] Recommendations

### Report Content
- [ ] Date and time of testing
- [ ] Tester information
- [ ] Environment specifications
- [ ] Tool versions used
- [ ] Complete test results
- [ ] Evidence (logs, screenshots)

## Quality Checks

### Testing Completeness
- [ ] All endpoints tested
- [ ] All error scenarios tested
- [ ] Edge cases considered
- [ ] Performance measured
- [ ] Logging verified

### Documentation Quality
- [ ] Clear and concise
- [ ] Properly formatted
- [ ] Includes all evidence
- [ ] Actionable recommendations
- [ ] Professional presentation

## Definition of Done
- All test cases executed successfully
- No critical issues found
- Minor issues documented with workarounds
- Comprehensive test report created
- API confirmed ready for use
- Testing can be reproduced

## Common Issues to Check
1. Port conflicts preventing server start
2. Malformed JSON responses
3. Missing Content-Type headers
4. Incorrect status codes
5. Timestamp format issues
6. Logging not working
7. Error responses inconsistent
8. Performance degradation

## Additional Validation
- [ ] Test with different clients (curl, wget, browser)
- [ ] Verify from different terminals
- [ ] Check with different user permissions
- [ ] Test after server restart
- [ ] Validate against API documentation