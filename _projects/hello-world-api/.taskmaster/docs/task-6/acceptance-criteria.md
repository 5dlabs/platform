# Acceptance Criteria: Request Logging Enhancement

## Definition of Done
Request logging is considered properly implemented when all the following criteria are met:

## Required Outcomes

### 1. Logging Middleware Presence ✓
- [ ] Logging middleware exists in src/index.js
- [ ] Middleware is the first in the chain
- [ ] Middleware properly calls next()
- [ ] No duplicate logging implementations

### 2. Log Format Requirements ✓
Each log entry must contain:
- [ ] Timestamp in ISO 8601 format
- [ ] HTTP method (GET, POST, etc.)
- [ ] Request URL/path
- [ ] Format: `TIMESTAMP - METHOD URL`

Example:
```
2024-01-15T14:32:17.123Z - GET /
2024-01-15T14:32:18.456Z - GET /health
```

### 3. Logging Coverage ✓
- [ ] All valid endpoints are logged
- [ ] 404 requests are logged
- [ ] Error-causing requests are logged
- [ ] All HTTP methods are logged
- [ ] Static file requests logged (if any)

### 4. Timestamp Validation ✓
- [ ] ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
- [ ] Includes milliseconds
- [ ] UTC timezone (Z suffix)
- [ ] Timestamps increment correctly

## Test Cases

### Test Case 1: Basic GET Requests
```bash
curl http://localhost:3000/
curl http://localhost:3000/health
```
**Expected Logs:**
```
2024-01-15T14:32:17.123Z - GET /
2024-01-15T14:32:17.456Z - GET /health
```

### Test Case 2: Various HTTP Methods
```bash
curl -X GET http://localhost:3000/
curl -X POST http://localhost:3000/
curl -X PUT http://localhost:3000/
curl -X DELETE http://localhost:3000/
curl -X PATCH http://localhost:3000/
```
**Expected:** Each method logged correctly

### Test Case 3: 404 Requests
```bash
curl http://localhost:3000/does-not-exist
curl http://localhost:3000/api/users
curl http://localhost:3000/admin/panel
```
**Expected:** All requests logged before 404 response

### Test Case 4: Error-Triggering Requests
```bash
# If test error endpoint exists
curl http://localhost:3000/test-error
```
**Expected:** Request logged even if error occurs

### Test Case 5: Rapid Sequential Requests
```bash
for i in {1..10}; do
  curl -s http://localhost:3000/ &
done
wait
```
**Expected:** All 10 requests logged with unique timestamps

## Performance Testing

### Response Time Impact
```bash
# Measure without logging (comment out middleware)
time curl http://localhost:3000/

# Measure with logging
time curl http://localhost:3000/
```
**Expected:** Difference < 5ms

### High Volume Test
```bash
# Using Apache Bench if available
ab -n 1000 -c 10 http://localhost:3000/
```
**Expected:** 
- No dropped logs
- No server crashes
- Consistent performance

## Edge Cases

### Edge Case 1: Special Characters in URL
```bash
curl "http://localhost:3000/test?param=value&special=!@%23$%25"
curl "http://localhost:3000/path%20with%20spaces"
```
**Expected:** URLs logged correctly, properly encoded

### Edge Case 2: Long URLs
```bash
curl "http://localhost:3000/very/long/path/that/might/cause/issues/with/logging/if/not/handled/properly/123456789"
```
**Expected:** Full URL logged without truncation

### Edge Case 3: Empty Path
```bash
curl http://localhost:3000
```
**Expected:** Logs show "/" (normalized path)

### Edge Case 4: Multiple Slashes
```bash
curl http://localhost:3000///health
```
**Expected:** Path normalized or shown as-is

## Common Issues & Solutions

### Issue 1: Missing Logs
**Symptom**: Some requests not logged
**Cause**: Middleware placement or early response
**Fix**: Ensure logging is first middleware

### Issue 2: Timestamp Format Wrong
**Symptom**: Timestamps not ISO 8601
**Cause**: Using wrong Date method
**Fix**: Use `new Date().toISOString()`

### Issue 3: Logs Out of Order
**Symptom**: Timestamps not sequential
**Cause**: Async operations or system time
**Fix**: Expected behavior for concurrent requests

### Issue 4: Memory Growth
**Symptom**: Memory usage increases over time
**Cause**: Console buffer or memory leak
**Fix**: Implement log rotation or external logging

## Integration Requirements

### With Error Handler
- [ ] Requests logged before errors occur
- [ ] Error handler doesn't prevent logging
- [ ] Stack traces separate from request logs

### With Routes
- [ ] All routes properly logged
- [ ] Route parameters visible in logs
- [ ] Query strings included

### With 404 Handler
- [ ] 404 requests are logged
- [ ] Log appears before 404 response
- [ ] Path information preserved

## Console Output Validation

### Format Consistency
```bash
# Check log format consistency
npm start 2>&1 | grep -E "^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z - [A-Z]+ /"
```
**Expected:** All request logs match pattern

### No Extraneous Output
- [ ] Only request logs during normal operation
- [ ] Server startup message separate
- [ ] Error logs clearly distinguished

## Security Considerations
- [ ] No sensitive data in logs (passwords, tokens)
- [ ] No request bodies logged
- [ ] No authorization headers exposed
- [ ] User data not logged

## Documentation
- [ ] Logging behavior documented
- [ ] Log format explained
- [ ] Timestamp format specified
- [ ] Enhancement options noted

## Sign-off Checklist
- [ ] All endpoints properly logged
- [ ] Format meets requirements
- [ ] Performance impact minimal
- [ ] No security issues
- [ ] Logs useful for debugging
- [ ] No regression in functionality