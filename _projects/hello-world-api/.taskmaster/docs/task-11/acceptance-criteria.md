# Acceptance Criteria: Add Error Handling

## Task Overview
**Task ID**: 11  
**Task Title**: Add Error Handling  
**Purpose**: Implement middleware to handle server errors and undefined routes with appropriate status codes

## Prerequisites
- [ ] Task 9 completed: Root endpoint implemented
- [ ] Task 10 completed: Health check endpoint implemented
- [ ] Express server has existing routes
- [ ] Middleware chain is established

## Acceptance Criteria Checklist

### 1. Error Handling Middleware
- [ ] **Middleware created**: Error handler with 4 parameters
- [ ] **Parameter signature**: (err, req, res, next)
- [ ] **Placement correct**: After all routes
- [ ] **Returns JSON**: Error response in JSON format
- [ ] **Status code 500**: Internal server errors

### 2. Error Logging
- [ ] **Error message logged**: console.error with message
- [ ] **Stack trace logged**: Full stack trace to console
- [ ] **Request path logged**: Path that caused error
- [ ] **Request method logged**: HTTP method logged
- [ ] **No client exposure**: Stack not sent to client

### 3. 404 Handler
- [ ] **Handler created**: Catch-all middleware
- [ ] **Placement correct**: Very last middleware
- [ ] **Returns JSON**: {"error": "Not Found"}
- [ ] **Status code 404**: For undefined routes
- [ ] **Catches all methods**: GET, POST, PUT, DELETE, etc.

### 4. Response Formats
- [ ] **500 response format**: {"error": "Internal Server Error"}
- [ ] **404 response format**: {"error": "Not Found"}
- [ ] **Content-Type**: application/json
- [ ] **Consistent structure**: Both use "error" field

### 5. Middleware Order
- [ ] **Correct sequence**: 
  1. Request logging
  2. Routes (/, /health)
  3. Error handler
  4. 404 handler
- [ ] **No middleware after 404**: It's the last one
- [ ] **Routes still accessible**: Existing endpoints work

## Test Cases

### Test Case 1: 404 for Undefined Route
**Steps**:
1. Start server
2. Request: `curl http://localhost:3000/undefined`

**Expected Result**:
```json
{"error":"Not Found"}
```
**Status**: 404

### Test Case 2: 404 for Wrong Method
**Steps**:
1. Request: `curl -X POST http://localhost:3000/`
2. Request: `curl -X DELETE http://localhost:3000/health`

**Expected Result**:
- Both return: {"error":"Not Found"}
- Status: 404

### Test Case 3: Multiple Undefined Paths
**Steps**:
1. Test various paths:
   ```bash
   curl http://localhost:3000/api
   curl http://localhost:3000/users
   curl http://localhost:3000/test/nested/path
   ```

**Expected Result**:
- All return 404 with {"error":"Not Found"}

### Test Case 4: Error Handler (with test route)
**Setup**: Add temporary test route
```javascript
app.get('/test-error', (req, res, next) => {
  next(new Error('Test error'));
});
```

**Steps**:
1. Request: `curl http://localhost:3000/test-error`

**Expected Result**:
- Response: {"error":"Internal Server Error"}
- Status: 500
- Console shows error details

### Test Case 5: Existing Routes Still Work
**Steps**:
1. Test root: `curl http://localhost:3000/`
2. Test health: `curl http://localhost:3000/health`

**Expected Result**:
- Root returns: {"message":"Hello, World!"}
- Health returns: {"status":"healthy","timestamp":"..."}
- Both return 200 status

## Edge Cases to Consider

### 1. Case Sensitivity
- **Test**: /HEALTH vs /health
- **Expected**: /HEALTH returns 404
- **Reason**: Express routes are case-sensitive

### 2. Trailing Slashes
- **Test**: /health/ vs /health
- **Expected**: May differ based on Express config
- **Default**: Both should work

### 3. Query Parameters on 404
- **Test**: /undefined?param=value
- **Expected**: Still returns 404
- **Logging**: Full path with query logged

### 4. Special Characters
- **Test**: /test%20space or /test/../../etc
- **Expected**: 404 for all invalid paths
- **Security**: No path traversal

## Console Output Validation

### For 500 Errors
**Expected Console Output**:
```
Error: Test error message
Stack: Error: Test error message
    at app.get (/path/to/index.js:45:10)
    at Layer.handle [as handle_request]...
Request path: /test-error
Request method: GET
```

### For 404 Errors
**Expected Console Output**:
```
2024-01-15T10:30:45.123Z - GET /undefined
```
(Only request logging, no error logs)

## Performance Criteria

- **Error handling overhead**: < 1ms
- **404 checking**: < 1ms
- **No memory leaks**: Errors properly handled
- **Logging performance**: Synchronous but fast

## Security Validation

- [ ] **No stack traces to client**: Generic messages only
- [ ] **No system paths exposed**: No file paths in responses
- [ ] **No sensitive data**: Error details stay server-side
- [ ] **Consistent responses**: Same format for all errors

## Integration Testing

### Middleware Chain Test
1. **Valid request**: Should pass through logging → route → response
2. **Error request**: Should pass through logging → route → error handler
3. **404 request**: Should pass through logging → 404 handler

### Load Testing
- **Scenario**: 1000 requests to undefined routes
- **Expected**: All return 404 without crashes
- **Performance**: No degradation

## Definition of Done

1. **Error middleware implemented** with 4 parameters
2. **404 handler implemented** as last middleware
3. **Both return JSON** with appropriate status codes
4. **Error logging works** with detailed information
5. **Existing routes unaffected** and still functional
6. **Middleware order correct** as specified
7. **All test cases pass** including edge cases
8. **No security issues** with error exposure

## Success Metrics

- **Error handling**: 100% of errors caught
- **404 coverage**: All undefined routes handled
- **Response consistency**: All errors return JSON
- **Logging completeness**: All errors logged with details

## Notes for QA/Review

- Verify 4-parameter signature for error middleware
- Check middleware order carefully
- Test with various error scenarios
- Confirm no sensitive data in responses
- Validate JSON response format
- Test concurrent error requests
- Ensure graceful error handling