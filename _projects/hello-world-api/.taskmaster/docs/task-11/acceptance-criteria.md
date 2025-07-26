# Acceptance Criteria for Task 11: Add Error Handling

## Required Outcomes

### 1. Error Handling Middleware
- [ ] Error middleware function has exactly 4 parameters
- [ ] Placed after all route definitions
- [ ] Placed before 404 handler
- [ ] Logs error message to console
- [ ] Returns 500 status code
- [ ] Returns JSON with "error" property

### 2. 404 Not Found Handler
- [ ] Middleware function has 2 parameters (req, res)
- [ ] Placed as the last middleware
- [ ] Placed before app.listen()
- [ ] Returns 404 status code
- [ ] Returns JSON with "error" property
- [ ] Catches all undefined routes

### 3. Error Logging
- [ ] Error message is logged
- [ ] Error timestamp is logged
- [ ] Request method and path are logged
- [ ] Stack trace is available for debugging
- [ ] Logs are formatted consistently

### 4. Response Standards
- [ ] All error responses are JSON
- [ ] Consistent error property in responses
- [ ] Appropriate HTTP status codes
- [ ] No sensitive information exposed
- [ ] Response time remains fast

## Test Cases

### Test 1: 404 for Undefined GET Route
```bash
curl -v http://localhost:3000/undefined
# Expected response:
# HTTP/1.1 404 Not Found
# {"error":"Not Found"}
```

### Test 2: 404 for Undefined POST Route
```bash
curl -X POST http://localhost:3000/api/users
# Expected response:
# {"error":"Not Found"}
```

### Test 3: 404 for Various Paths
```bash
# Test multiple undefined paths
for path in "/admin" "/api" "/users/123" "/../../etc/passwd"; do
  echo "Testing: $path"
  curl -s -o /dev/null -w "%{http_code}" http://localhost:3000$path
done
# Expected: All return 404
```

### Test 4: Existing Routes Still Work
```bash
# Root endpoint
curl http://localhost:3000/
# Expected: {"message":"Hello, World!"}

# Health endpoint
curl http://localhost:3000/health
# Expected: {"status":"healthy","timestamp":"..."}
```

### Test 5: Error Handler Test
Create temporary test route:
```javascript
app.get('/test-error', (req, res, next) => {
  next(new Error('Test error'));
});
```

Then test:
```bash
curl -v http://localhost:3000/test-error
# Expected:
# HTTP/1.1 500 Internal Server Error
# {"error":"Internal Server Error"}

# Check server logs for error details
```

### Test 6: Malformed Request Handling
```bash
# Send malformed JSON (if body parsing is added)
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{"invalid json'
# Should handle gracefully (404 or appropriate error)
```

## Code Quality Checks

### Middleware Order Verification
```javascript
// Correct order:
// 1. Request logging middleware
// 2. Routes (/, /health)
// 3. Error handling middleware (4 params)
// 4. 404 handler (2 params)
// 5. app.listen()
```

### Error Middleware Signature
- [ ] Has exactly 4 parameters: (err, req, res, next)
- [ ] Parameters are in correct order
- [ ] Even if 'next' unused, it must be present

### 404 Handler Signature
- [ ] Has exactly 2 parameters: (req, res)
- [ ] No 'next' parameter (it's terminal)

## Logging Requirements
- [ ] All errors include timestamp
- [ ] Error messages are descriptive
- [ ] Request context is logged
- [ ] No passwords or tokens in logs
- [ ] Consistent log format

## Definition of Done
- Error handling middleware implemented correctly
- 404 handler catches all undefined routes
- All existing endpoints still function
- Comprehensive error logging in place
- Server remains stable under error conditions
- No unhandled promise rejections
- Clean, consistent error responses

## Common Issues to Avoid
1. Wrong parameter count in error middleware
2. Error handlers placed before routes
3. Missing error logging information
4. Exposing sensitive error details
5. Not testing with actual errors
6. 404 handler with 'next' parameter
7. Inconsistent response formats

## Security Considerations
- [ ] Stack traces hidden in production
- [ ] Generic error messages for 500s
- [ ] No path traversal information exposed
- [ ] Rate limiting considerations for errors
- [ ] No internal system details revealed