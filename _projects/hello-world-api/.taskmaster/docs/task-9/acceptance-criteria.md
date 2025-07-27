# Task 9: Implement Root Endpoint - Acceptance Criteria

## Definition of Done
The root endpoint is successfully implemented when all the following criteria are met:

## Required Deliverables

### 1. Route Implementation
- [ ] GET route handler for path '/' is defined
- [ ] Route is defined using `app.get('/', ...)`
- [ ] Route handler function has correct signature `(req, res) => {}`
- [ ] Route is placed after middleware, before error handlers

### 2. Response Format
- [ ] Response is sent using `res.json()` method
- [ ] Response body is a JSON object
- [ ] JSON object contains "message" field
- [ ] Message value is exactly "Hello, World!"
- [ ] No extra fields in the JSON response

### 3. HTTP Status
- [ ] HTTP status code is explicitly set to 200
- [ ] Status is set before sending response
- [ ] Uses `res.status(200)` method

### 4. Code Quality
- [ ] Descriptive comment above route handler
- [ ] Proper indentation and formatting
- [ ] No syntax errors
- [ ] Route integrated into existing server file

## Verification Tests

### Test 1: Basic Endpoint Access
```bash
# Start server and test endpoint
npm start &
SERVER_PID=$!
sleep 2

# Test the endpoint
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/)
BODY=$(echo "$RESPONSE" | head -n -1)
STATUS=$(echo "$RESPONSE" | tail -n 1)

if [ "$STATUS" = "200" ]; then
  echo "✓ Status code is 200"
else
  echo "✗ Wrong status code: $STATUS"
fi

kill $SERVER_PID
```

### Test 2: JSON Response Validation
```bash
# Check JSON structure
npm start &
SERVER_PID=$!
sleep 2

RESPONSE=$(curl -s http://localhost:3000/)

# Validate JSON and message
node -e "
  try {
    const data = JSON.parse('$RESPONSE');
    if (data.message === 'Hello, World!') {
      console.log('✓ Correct JSON response');
    } else {
      console.log('✗ Wrong message:', data.message);
    }
  } catch (e) {
    console.log('✗ Invalid JSON response');
  }
"

kill $SERVER_PID
```

### Test 3: Content-Type Header
```bash
# Check response headers
npm start &
SERVER_PID=$!
sleep 2

HEADERS=$(curl -s -I http://localhost:3000/)

if echo "$HEADERS" | grep -qi "content-type.*application/json"; then
  echo "✓ Content-Type is application/json"
else
  echo "✗ Wrong Content-Type header"
fi

kill $SERVER_PID
```

### Test 4: Request Logging
```bash
# Verify request is logged
npm start > server.log 2>&1 &
SERVER_PID=$!
sleep 2

curl -s http://localhost:3000/ > /dev/null
sleep 1

if grep -q "GET /" server.log; then
  echo "✓ Request was logged"
else
  echo "✗ Request not logged"
fi

kill $SERVER_PID
rm -f server.log
```

### Test 5: Route Placement
```bash
# Check route is defined in correct location
if grep -A5 -B5 "app.get('/'" src/index.js | grep -q "app.use"; then
  echo "✓ Route appears to be correctly placed"
else
  echo "✗ Route may be in wrong location"
fi
```

## Edge Cases to Handle

1. **Multiple GET requests**
   - Endpoint should handle repeated requests
   - Each request should return same response
   - No state changes between requests

2. **Different HTTP methods**
   - Only GET should work
   - POST, PUT, DELETE should return 404
   - OPTIONS might return due to CORS

3. **Query parameters**
   - GET /?param=value should still work
   - Response should ignore query parameters
   - Same response regardless of parameters

4. **Request headers**
   - Should work with any headers
   - Accept header shouldn't affect JSON response
   - User-Agent shouldn't matter

## Success Metrics
- Response time under 100ms for local requests
- Consistent response format across all requests
- No errors in server logs
- Memory usage remains stable

## Common Failure Modes

1. **Wrong Response Format**
   ```javascript
   // Wrong:
   res.send("Hello, World!");  // Not JSON
   res.json("Hello, World!");   // Not an object
   res.json({ msg: "Hello, World!" });  // Wrong field name
   ```

2. **Missing Status Code**
   ```javascript
   // Wrong:
   res.json({ message: "Hello, World!" });  // No explicit status
   ```

3. **Route Placement Issues**
   ```javascript
   // Wrong: After app.listen()
   app.listen(PORT, () => {});
   app.get('/', handler);  // Too late!
   ```

4. **Syntax Errors**
   - Missing closing brackets
   - Missing semicolons
   - Incorrect arrow function syntax

## Final Validation Script
```bash
#!/bin/bash
echo "Testing Root Endpoint Implementation..."

# Start server
npm start > test.log 2>&1 &
SERVER_PID=$!
sleep 3

# Function to cleanup
cleanup() {
  kill $SERVER_PID 2>/dev/null
  rm -f test.log response.json
}

# Test endpoint
curl -s -o response.json -w "%{http_code}" http://localhost:3000/ > status.txt

STATUS=$(cat status.txt)
if [ "$STATUS" != "200" ]; then
  echo "✗ Wrong status code: $STATUS"
  cleanup
  exit 1
fi

# Validate JSON
if ! jq . response.json > /dev/null 2>&1; then
  echo "✗ Response is not valid JSON"
  cleanup
  exit 1
fi

# Check message
MESSAGE=$(jq -r .message response.json)
if [ "$MESSAGE" != "Hello, World!" ]; then
  echo "✗ Wrong message: $MESSAGE"
  cleanup
  exit 1
fi

# Check logging
if ! grep -q "GET /" test.log; then
  echo "✗ Request not logged"
  cleanup
  exit 1
fi

echo "✅ All root endpoint tests passed!"
cleanup
rm -f status.txt
```