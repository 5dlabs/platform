# Task 11: Add Error Handling - Acceptance Criteria

## Definition of Done
Error handling is successfully implemented when all the following criteria are met:

## Required Deliverables

### 1. Error Handling Middleware
- [ ] Error handler middleware is implemented
- [ ] Has exactly 4 parameters: `(err, req, res, next)`
- [ ] Returns 500 status code
- [ ] Returns JSON with "error" field
- [ ] Error message is "Internal Server Error"
- [ ] Logs error details to console

### 2. 404 Handler Middleware
- [ ] 404 handler is implemented
- [ ] Has exactly 2 parameters: `(req, res)`
- [ ] Returns 404 status code
- [ ] Returns JSON with "error" field
- [ ] Error message is "Not Found"
- [ ] Catches all undefined routes

### 3. Middleware Order
- [ ] Error handler is placed after all routes
- [ ] 404 handler is placed after error handler
- [ ] Both are before app.listen()
- [ ] Request logging remains first

### 4. Response Format
- [ ] All error responses are JSON
- [ ] Consistent error field name
- [ ] No stack traces in responses
- [ ] No sensitive information exposed

### 5. Error Logging
- [ ] Errors are logged with message
- [ ] Console.error is used for errors
- [ ] Timestamp included in logs
- [ ] No client data logged

## Verification Tests

### Test 1: 404 Handler Basic Test
```bash
# Start server and test 404
npm start &
SERVER_PID=$!
sleep 2

# Test undefined route
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/undefined)
BODY=$(echo "$RESPONSE" | head -n -1)
STATUS=$(echo "$RESPONSE" | tail -n 1)

if [ "$STATUS" = "404" ]; then
  echo "✓ 404 status returned"
else
  echo "✗ Wrong status: $STATUS"
fi

if echo "$BODY" | jq -e '.error == "Not Found"' > /dev/null 2>&1; then
  echo "✓ Correct error message"
else
  echo "✗ Wrong error response"
fi

kill $SERVER_PID
```

### Test 2: Multiple 404 Scenarios
```bash
# Test various undefined routes
npm start &
SERVER_PID=$!
sleep 2

ROUTES=("/api" "/users" "/test" "/api/v1/data" "/health/check")

for route in "${ROUTES[@]}"; do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3000$route")
  if [ "$STATUS" = "404" ]; then
    echo "✓ $route returns 404"
  else
    echo "✗ $route returns $STATUS"
  fi
done

kill $SERVER_PID
```

### Test 3: HTTP Method 404s
```bash
# Test wrong methods on defined routes
npm start &
SERVER_PID=$!
sleep 2

# POST to GET-only routes
POST_ROOT=$(curl -s -X POST -w "\n%{http_code}" http://localhost:3000/ | tail -n 1)
POST_HEALTH=$(curl -s -X POST -w "\n%{http_code}" http://localhost:3000/health | tail -n 1)

if [ "$POST_ROOT" = "404" ] && [ "$POST_HEALTH" = "404" ]; then
  echo "✓ POST methods return 404"
else
  echo "✗ POST methods not handled correctly"
fi

kill $SERVER_PID
```

### Test 4: Error Handler (with test route)
```bash
# Add test error route temporarily
cat > test-error.js << 'EOF'
const src = require('fs').readFileSync('src/index.js', 'utf8');
const modified = src.replace(
  "app.get('/health'",
  "app.get('/test-error', (req, res, next) => next(new Error('Test')));\n\napp.get('/health'"
);
require('fs').writeFileSync('src/index.js', modified);
EOF

node test-error.js

# Test error handling
npm start > server.log 2>&1 &
SERVER_PID=$!
sleep 2

ERROR_RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/test-error)
ERROR_STATUS=$(echo "$ERROR_RESPONSE" | tail -n 1)

if [ "$ERROR_STATUS" = "500" ]; then
  echo "✓ Error returns 500"
else
  echo "✗ Error returns $ERROR_STATUS"
fi

# Check error was logged
if grep -q "Error: Test" server.log; then
  echo "✓ Error was logged"
else
  echo "✗ Error not logged"
fi

kill $SERVER_PID

# Restore original file
git checkout src/index.js 2>/dev/null || echo "Note: Restore src/index.js manually"
rm -f test-error.js server.log
```

### Test 5: Middleware Order Verification
```bash
# Check that error handlers are in correct position
if grep -A20 "app.get.*health" src/index.js | grep -q "app.use.*err.*req.*res.*next"; then
  echo "✓ Error handler after routes"
else
  echo "✗ Error handler not found after routes"
fi

if grep -A5 "err.*req.*res.*next" src/index.js | grep -q "app.use.*req.*res.*{$"; then
  echo "✓ 404 handler after error handler"
else
  echo "✗ 404 handler not in correct position"
fi
```

## Edge Cases to Handle

1. **Async Route Errors**
   - Errors in async functions need proper handling
   - Use try/catch or error handling middleware

2. **JSON Parsing Errors**
   - Invalid JSON in request body
   - Should return 400 or 500, not crash

3. **Large Request Bodies**
   - Server should handle gracefully
   - Not cause memory issues

4. **Special Characters in URLs**
   - URL encoding issues
   - Should return 404, not crash

## Success Metrics
- Zero unhandled errors
- All routes tested return expected status
- Server remains stable after errors
- Response time under 50ms for errors
- Memory usage stable

## Common Failure Modes

1. **Wrong Parameter Count**
   ```javascript
   // Wrong - only 3 parameters
   app.use((err, req, res) => { });
   
   // Wrong - too many parameters
   app.use((err, req, res, next, extra) => { });
   ```

2. **Wrong Middleware Order**
   ```javascript
   // Wrong - error handlers before routes
   app.use((err, req, res, next) => { });
   app.get('/', handler);
   ```

3. **Not Calling res.send/json**
   ```javascript
   // Wrong - no response sent
   app.use((req, res) => {
     console.log('404');
     // Missing: res.status(404).json(...)
   });
   ```

4. **Exposing Stack Traces**
   ```javascript
   // Wrong - exposes internal details
   app.use((err, req, res, next) => {
     res.status(500).json({ 
       error: err.message,
       stack: err.stack  // Security risk!
     });
   });
   ```

## Final Validation Script
```bash
#!/bin/bash
echo "Testing Error Handling Implementation..."

# Start server
npm start > test.log 2>&1 &
SERVER_PID=$!
sleep 3

# Cleanup function
cleanup() {
  kill $SERVER_PID 2>/dev/null
  rm -f test.log
}

# Test existing routes still work
ROOT_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/)
HEALTH_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health)

if [ "$ROOT_STATUS" != "200" ] || [ "$HEALTH_STATUS" != "200" ]; then
  echo "✗ Existing routes broken"
  cleanup
  exit 1
fi

# Test 404 handling
NOTFOUND_RESPONSE=$(curl -s http://localhost:3000/notfound)
if ! echo "$NOTFOUND_RESPONSE" | jq -e '.error == "Not Found"' > /dev/null 2>&1; then
  echo "✗ 404 handler not working"
  cleanup
  exit 1
fi

# Test various 404s
for method in POST PUT DELETE PATCH; do
  STATUS=$(curl -s -X $method -o /dev/null -w "%{http_code}" http://localhost:3000/)
  if [ "$STATUS" != "404" ]; then
    echo "✗ $method to / should return 404, got $STATUS"
    cleanup
    exit 1
  fi
done

echo "✓ Error handling middleware working correctly"
echo "✅ All error handling tests passed!"
cleanup
```