# Task 10: Implement Health Check Endpoint - Acceptance Criteria

## Definition of Done
The health check endpoint is successfully implemented when all the following criteria are met:

## Required Deliverables

### 1. Route Implementation
- [ ] GET route handler for path '/health' is defined
- [ ] Route uses `app.get('/health', ...)`
- [ ] Route handler has correct signature `(req, res) => {}`
- [ ] Route is placed after root endpoint

### 2. Response Format
- [ ] Response is sent using `res.json()` method
- [ ] Response body is a JSON object
- [ ] JSON contains "status" field with value "healthy"
- [ ] JSON contains "timestamp" field
- [ ] No additional fields in response

### 3. Timestamp Requirements
- [ ] Timestamp is generated using `new Date().toISOString()`
- [ ] Timestamp is in ISO 8601 format
- [ ] Timestamp is generated fresh on each request
- [ ] Timestamp reflects current time

### 4. HTTP Status
- [ ] HTTP status code is explicitly set to 200
- [ ] Status is set using `res.status(200)`
- [ ] Status is set before JSON response

### 5. Code Quality
- [ ] Descriptive comment above route handler
- [ ] Proper indentation and formatting
- [ ] No syntax errors
- [ ] Integrated into existing server file

## Verification Tests

### Test 1: Endpoint Accessibility
```bash
# Start server and test health endpoint
npm start &
SERVER_PID=$!
sleep 2

# Test the endpoint
STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health)

if [ "$STATUS" = "200" ]; then
  echo "✓ Health endpoint returns 200"
else
  echo "✗ Wrong status code: $STATUS"
fi

kill $SERVER_PID
```

### Test 2: Response Structure
```bash
# Check JSON structure
npm start &
SERVER_PID=$!
sleep 2

RESPONSE=$(curl -s http://localhost:3000/health)

# Validate JSON fields
node -e "
  try {
    const data = JSON.parse('$RESPONSE');
    let valid = true;
    
    if (data.status !== 'healthy') {
      console.log('✗ Wrong status value:', data.status);
      valid = false;
    }
    
    if (!data.timestamp) {
      console.log('✗ Missing timestamp');
      valid = false;
    }
    
    if (Object.keys(data).length !== 2) {
      console.log('✗ Extra fields in response');
      valid = false;
    }
    
    if (valid) {
      console.log('✓ Response structure is correct');
    }
  } catch (e) {
    console.log('✗ Invalid JSON response');
  }
"

kill $SERVER_PID
```

### Test 3: Timestamp Validation
```bash
# Verify timestamp format and freshness
npm start &
SERVER_PID=$!
sleep 2

RESPONSE=$(curl -s http://localhost:3000/health)

node -e "
  const data = JSON.parse('$RESPONSE');
  const timestamp = new Date(data.timestamp);
  
  // Check valid date
  if (isNaN(timestamp.getTime())) {
    console.log('✗ Invalid timestamp format');
  } else {
    console.log('✓ Valid ISO timestamp');
  }
  
  // Check freshness (within 2 seconds)
  const age = Date.now() - timestamp.getTime();
  if (age < 2000) {
    console.log('✓ Timestamp is current');
  } else {
    console.log('✗ Timestamp is stale:', age, 'ms old');
  }
"

kill $SERVER_PID
```

### Test 4: Dynamic Timestamp
```bash
# Verify timestamp changes between requests
npm start &
SERVER_PID=$!
sleep 2

TIME1=$(curl -s http://localhost:3000/health | jq -r .timestamp)
sleep 1
TIME2=$(curl -s http://localhost:3000/health | jq -r .timestamp)

if [ "$TIME1" != "$TIME2" ]; then
  echo "✓ Timestamp changes between requests"
else
  echo "✗ Timestamp is static"
fi

kill $SERVER_PID
```

### Test 5: Request Logging
```bash
# Verify health checks are logged
npm start > server.log 2>&1 &
SERVER_PID=$!
sleep 2

curl -s http://localhost:3000/health > /dev/null
sleep 1

if grep -q "GET /health" server.log; then
  echo "✓ Health check request was logged"
else
  echo "✗ Health check not logged"
fi

kill $SERVER_PID
rm -f server.log
```

## Edge Cases to Handle

1. **Rapid Sequential Requests**
   - Each request should have unique timestamp
   - No caching of responses
   - Consistent performance

2. **Concurrent Requests**
   - Handle multiple simultaneous health checks
   - Each gets its own timestamp
   - No race conditions

3. **Long Running Server**
   - Health check should work after hours/days
   - No memory leaks
   - Consistent response time

4. **Time Zone Considerations**
   - Always use UTC (via toISOString())
   - No local time zone issues

## Success Metrics
- Response time consistently under 10ms
- Zero errors across 1000 requests
- Timestamp accuracy within 1ms
- Memory usage stable over time

## Common Failure Modes

1. **Static Timestamp**
   ```javascript
   // Wrong - timestamp generated once
   const timestamp = new Date().toISOString();
   app.get('/health', (req, res) => {
     res.json({ status: 'healthy', timestamp });
   });
   ```

2. **Wrong Status Value**
   ```javascript
   // Wrong - incorrect status values
   res.json({ status: 'ok' });        // Should be 'healthy'
   res.json({ status: 'running' });   // Should be 'healthy'
   res.json({ status: true });        // Should be 'healthy'
   ```

3. **Extra Fields**
   ```javascript
   // Wrong - only status and timestamp allowed
   res.json({
     status: 'healthy',
     timestamp: new Date().toISOString(),
     version: '1.0.0',  // Not in requirements
     uptime: 12345      // Not in requirements
   });
   ```

4. **Wrong Timestamp Format**
   ```javascript
   // Wrong formats
   timestamp: Date.now()              // Unix timestamp
   timestamp: new Date().toString()   // Local string format
   timestamp: new Date().getTime()    // Milliseconds
   ```

## Final Validation Script
```bash
#!/bin/bash
echo "Testing Health Check Endpoint..."

# Start server
npm start > test.log 2>&1 &
SERVER_PID=$!
sleep 3

# Function to cleanup
cleanup() {
  kill $SERVER_PID 2>/dev/null
  rm -f test.log response.json
}

# Test 1: Status code
STATUS=$(curl -s -o response.json -w "%{http_code}" http://localhost:3000/health)
if [ "$STATUS" != "200" ]; then
  echo "✗ Wrong status code: $STATUS"
  cleanup
  exit 1
fi

# Test 2: Valid JSON
if ! jq . response.json > /dev/null 2>&1; then
  echo "✗ Response is not valid JSON"
  cleanup
  exit 1
fi

# Test 3: Check fields
STATUS_VAL=$(jq -r .status response.json)
TIMESTAMP=$(jq -r .timestamp response.json)
FIELD_COUNT=$(jq 'keys | length' response.json)

if [ "$STATUS_VAL" != "healthy" ]; then
  echo "✗ Wrong status value: $STATUS_VAL"
  cleanup
  exit 1
fi

if [ "$FIELD_COUNT" != "2" ]; then
  echo "✗ Wrong number of fields: $FIELD_COUNT"
  cleanup
  exit 1
fi

# Test 4: Timestamp format
if ! date -d "$TIMESTAMP" > /dev/null 2>&1; then
  echo "✗ Invalid timestamp format"
  cleanup
  exit 1
fi

# Test 5: Dynamic timestamp
sleep 1
TIME2=$(curl -s http://localhost:3000/health | jq -r .timestamp)
if [ "$TIMESTAMP" = "$TIME2" ]; then
  echo "✗ Timestamp not updating"
  cleanup
  exit 1
fi

echo "✅ All health check tests passed!"
cleanup
```