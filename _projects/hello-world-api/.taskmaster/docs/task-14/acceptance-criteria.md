# Acceptance Criteria: Manual Testing of API Endpoints

## Task Overview
**Task ID**: 14  
**Task Title**: Manual Testing of API Endpoints  
**Purpose**: Verify all API endpoints function correctly through comprehensive manual testing

## Prerequisites
- [ ] Task 9 completed: Root endpoint implemented
- [ ] Task 10 completed: Health check endpoint implemented
- [ ] Task 11 completed: Error handling implemented
- [ ] Server can be started successfully
- [ ] Testing tools available (curl, browser)

## Acceptance Criteria Checklist

### 1. Test Planning
- [ ] **Test cases defined**: All endpoints covered
- [ ] **Test tools ready**: curl/Postman/browser available
- [ ] **Environment setup**: Server running on port 3000
- [ ] **Documentation ready**: Template for results

### 2. Endpoint Testing
- [ ] **Root endpoint tested**: GET / returns Hello World
- [ ] **Health endpoint tested**: GET /health returns status
- [ ] **404 handling tested**: Undefined routes return 404
- [ ] **Method handling tested**: Wrong methods return 404
- [ ] **All tests documented**: Results recorded

### 3. Response Validation
- [ ] **Status codes correct**: 200 for success, 404 for errors
- [ ] **JSON format valid**: Proper JSON structure
- [ ] **Content-Type correct**: application/json
- [ ] **Response times acceptable**: < 50ms average
- [ ] **No errors in console**: Clean server logs

### 4. Error Handling
- [ ] **404 responses consistent**: {"error": "Not Found"}
- [ ] **No stack traces exposed**: Generic error messages
- [ ] **All error paths tested**: Various invalid routes
- [ ] **Error format uniform**: Same structure for all errors

### 5. Test Documentation
- [ ] **Test report created**: Comprehensive documentation
- [ ] **Results recorded**: Pass/fail for each test
- [ ] **Issues documented**: Any problems found
- [ ] **Screenshots included**: Evidence of testing
- [ ] **Recommendations made**: Improvement suggestions

## Test Cases

### Test Case 1: Root Endpoint Success
**ID**: TC001  
**Endpoint**: GET /  
**Steps**:
1. Start server with `npm start`
2. Execute: `curl http://localhost:3000/`
3. Verify response

**Expected**:
- Status: 200 OK
- Body: `{"message":"Hello, World!"}`
- Content-Type: application/json

**Validation**:
- [ ] Status code is 200
- [ ] JSON structure correct
- [ ] Message field present
- [ ] Value is "Hello, World!"

### Test Case 2: Health Check Success
**ID**: TC002  
**Endpoint**: GET /health  
**Steps**:
1. Execute: `curl http://localhost:3000/health`
2. Note timestamp
3. Repeat after 1 second

**Expected**:
- Status: 200 OK
- Body: `{"status":"healthy","timestamp":"<ISO>"}`
- Different timestamps

**Validation**:
- [ ] Status code is 200
- [ ] Status field is "healthy"
- [ ] Timestamp is ISO format
- [ ] Timestamp updates

### Test Case 3: 404 Error Handling
**ID**: TC003  
**Endpoint**: GET /undefined  
**Steps**:
1. Execute: `curl http://localhost:3000/undefined`
2. Try multiple undefined paths

**Expected**:
- Status: 404 Not Found
- Body: `{"error":"Not Found"}`

**Validation**:
- [ ] Status code is 404
- [ ] Error field present
- [ ] Value is "Not Found"
- [ ] Format consistent

### Test Case 4: Method Not Allowed
**ID**: TC004  
**Methods**: POST, PUT, DELETE  
**Steps**:
1. `curl -X POST http://localhost:3000/`
2. `curl -X PUT http://localhost:3000/health`
3. `curl -X DELETE http://localhost:3000/`

**Expected**:
- Status: 404 Not Found
- Body: `{"error":"Not Found"}`

**Validation**:
- [ ] All return 404
- [ ] Same error format
- [ ] No method-specific errors

### Test Case 5: Request Logging
**ID**: TC005  
**Action**: Monitor console  
**Steps**:
1. Make various requests
2. Check server console output

**Expected**:
```
2024-01-15T10:30:45.123Z - GET /
2024-01-15T10:30:46.456Z - GET /health
2024-01-15T10:30:47.789Z - GET /undefined
```

**Validation**:
- [ ] All requests logged
- [ ] ISO timestamp format
- [ ] Method shown
- [ ] Path shown
- [ ] No sensitive data

## Performance Criteria

### Response Time Testing
```bash
# Measure response time
curl -o /dev/null -s -w "Time: %{time_total}s\n" http://localhost:3000/
```

**Acceptance Criteria**:
- [ ] Average < 50ms
- [ ] Max < 100ms
- [ ] Consistent times
- [ ] No timeouts

### Load Testing
```bash
# 100 sequential requests
for i in {1..100}; do
  curl -s http://localhost:3000/ > /dev/null
done
```

**Acceptance Criteria**:
- [ ] All requests succeed
- [ ] No server crashes
- [ ] Memory stable
- [ ] Performance consistent

## Edge Case Testing

### Special Characters
- [ ] `/test%20space` returns 404
- [ ] `/test?param=value` works correctly
- [ ] `/test#anchor` handled properly
- [ ] `//double//slash` normalized

### Case Sensitivity
- [ ] `/HEALTH` returns 404
- [ ] `/Health` returns 404
- [ ] Routes are case-sensitive

### Concurrent Requests
- [ ] Multiple simultaneous requests handled
- [ ] No race conditions
- [ ] Responses don't mix

## Test Report Requirements

### Report Structure
1. **Executive Summary**
   - Test scope
   - Overall results
   - Key findings

2. **Detailed Results**
   - Test case results
   - Pass/fail status
   - Actual vs expected

3. **Issues Found**
   - Issue description
   - Severity level
   - Reproduction steps

4. **Performance Metrics**
   - Response times
   - Load test results
   - Resource usage

5. **Recommendations**
   - Improvements
   - Security concerns
   - Performance optimizations

## Definition of Done

1. **All test cases executed** with results documented
2. **Test report created** with comprehensive findings
3. **Issues logged** with severity and details
4. **Performance validated** against criteria
5. **Console logs verified** for proper formatting
6. **Error handling confirmed** working correctly
7. **Documentation complete** with evidence
8. **Recommendations provided** for improvements

## Success Metrics

- **Test Coverage**: 100% of endpoints tested
- **Pass Rate**: 95%+ tests passing
- **Performance**: All requests < 50ms average
- **Stability**: No crashes during testing
- **Documentation**: Complete test report

## Notes for QA/Review

- Use exact curl commands provided
- Test in clean environment
- Document exact responses
- Include timestamps in report
- Note any platform-specific issues
- Verify against acceptance criteria
- Test both positive and negative cases