# Task 3: Implement API Endpoints - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. File Structure ✓
- [ ] File `src/routes/index.js` updated with all route imports
- [ ] File `src/routes/health.js` exists
- [ ] File `src/routes/hello.js` exists
- [ ] File `src/routes/echo.js` exists
- [ ] File `src/routes/info.js` exists

### 2. Health Check Endpoint (/health) ✓
- [ ] GET method is implemented
- [ ] Returns 200 status code
- [ ] Response includes `status: 'up'` in data
- [ ] Uses standardized success response format
- [ ] Includes Swagger documentation comment
- [ ] Correlation ID appears in response header

### 3. Basic Greeting Endpoint (/hello) ✓
- [ ] GET method is implemented
- [ ] Returns 200 status code
- [ ] Response message is "Hello, World!"
- [ ] Response data includes `greeting: 'Hello, World!'`
- [ ] Uses standardized success response format
- [ ] Includes Swagger documentation comment

### 4. Personalized Greeting Endpoint (/hello/:name) ✓
- [ ] GET method with path parameter is implemented
- [ ] Returns 200 for valid names
- [ ] Returns 400 for empty/invalid names
- [ ] Name is sanitized (special characters removed)
- [ ] Response includes personalized greeting
- [ ] Error handling uses standardized format
- [ ] Includes Swagger documentation comment

### 5. Echo Service Endpoint (/echo) ✓
- [ ] POST method is implemented
- [ ] Returns 200 for valid JSON body
- [ ] Returns 400 for empty/missing body
- [ ] Response data matches request body exactly
- [ ] Uses standardized response format
- [ ] Includes Swagger documentation comment

### 6. Service Info Endpoint (/info) ✓
- [ ] GET method is implemented
- [ ] Returns 200 status code
- [ ] Response includes all required fields:
  - [ ] version (from package.json)
  - [ ] name (from package.json)
  - [ ] uptime (in seconds)
  - [ ] environment
  - [ ] hostname
  - [ ] platform
  - [ ] memory.total (in MB)
  - [ ] memory.free (in MB)
- [ ] Uptime increases on subsequent calls
- [ ] Uses standardized success response format
- [ ] Includes Swagger documentation comment

## Test Cases

### Test Case 1: Health Check
```bash
curl -i http://localhost:3000/health
```
**Expected Response:**
```json
{
  "status": "success",
  "message": "Service is healthy",
  "data": {
    "status": "up"
  },
  "timestamp": "2024-01-15T..."
}
```
**Expected Headers:** Contains `x-correlation-id`

### Test Case 2: Basic Greeting
```bash
curl http://localhost:3000/hello
```
**Expected Response:**
```json
{
  "status": "success",
  "message": "Hello, World!",
  "data": {
    "greeting": "Hello, World!"
  },
  "timestamp": "2024-01-15T..."
}
```

### Test Case 3: Personalized Greeting (Valid)
```bash
curl http://localhost:3000/hello/Alice
```
**Expected Response:**
```json
{
  "status": "success",
  "message": "Hello, Alice!",
  "data": {
    "greeting": "Hello, Alice!"
  },
  "timestamp": "2024-01-15T..."
}
```

### Test Case 4: Personalized Greeting (With Special Characters)
```bash
curl http://localhost:3000/hello/Alice%40%23%24
```
**Expected:** Special characters removed, response shows "Hello, Alice!"

### Test Case 5: Personalized Greeting (Empty)
```bash
curl http://localhost:3000/hello/%20
```
**Expected Response:**
```json
{
  "status": "error",
  "message": "Name parameter is required",
  "data": null,
  "timestamp": "2024-01-15T..."
}
```
**Expected Status:** 400

### Test Case 6: Echo Service (Valid Body)
```bash
curl -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "test", "number": 123}'
```
**Expected Response:**
```json
{
  "status": "success",
  "message": "Echo response",
  "data": {
    "message": "test",
    "number": 123
  },
  "timestamp": "2024-01-15T..."
}
```

### Test Case 7: Echo Service (Empty Body)
```bash
curl -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" \
  -d '{}'
```
**Expected Status:** 400
**Expected Message:** "Request body is required"

### Test Case 8: Service Info
```bash
curl http://localhost:3000/info
```
**Expected Response Structure:**
```json
{
  "status": "success",
  "message": "Service information",
  "data": {
    "version": "1.0.0",
    "name": "hello-world-api",
    "uptime": "10 seconds",
    "environment": "development",
    "hostname": "...",
    "platform": "...",
    "memory": {
      "total": "... MB",
      "free": "... MB"
    }
  },
  "timestamp": "2024-01-15T..."
}
```

## Validation Commands

### Comprehensive Endpoint Test
```bash
# Run all endpoint tests
echo "Testing /health..."
curl -s http://localhost:3000/health | jq .

echo -e "\nTesting /hello..."
curl -s http://localhost:3000/hello | jq .

echo -e "\nTesting /hello/World..."
curl -s http://localhost:3000/hello/World | jq .

echo -e "\nTesting /echo..."
curl -s -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" \
  -d '{"test": true}' | jq .

echo -e "\nTesting /info..."
curl -s http://localhost:3000/info | jq .
```

### Error Handling Test
```bash
# Test error cases
echo "Testing empty name..."
curl -s http://localhost:3000/hello/%20 | jq .

echo -e "\nTesting empty echo body..."
curl -s -X POST http://localhost:3000/echo \
  -H "Content-Type: application/json" | jq .

echo -e "\nTesting undefined route..."
curl -s http://localhost:3000/undefined | jq .
```

### Uptime Verification Test
```bash
# Check uptime increases
echo "First call:"
curl -s http://localhost:3000/info | jq .data.uptime
sleep 2
echo "Second call (after 2 seconds):"
curl -s http://localhost:3000/info | jq .data.uptime
```

## Success Indicators
- ✅ All endpoints return correct status codes
- ✅ Response format is consistent across all endpoints
- ✅ Correlation ID present in all responses
- ✅ Input validation works correctly
- ✅ Name sanitization removes special characters
- ✅ Echo returns exact input
- ✅ Info endpoint shows accurate system data
- ✅ Uptime increases over time
- ✅ Error responses use standard format

## Common Issues and Solutions

### Issue 1: "Cannot find module './health'"
**Solution:** Ensure all route files are created in src/routes/ directory

### Issue 2: Name parameter not sanitized
**Solution:** Use `.replace(/[^a-zA-Z0-9 ]/g, '')` on the name parameter

### Issue 3: Echo endpoint not checking for empty body
**Solution:** Add check: `!req.body || Object.keys(req.body).length === 0`

### Issue 4: Info uptime not updating
**Solution:** Ensure startTime is declared at module level, not inside route handler

### Issue 5: Package.json import fails
**Solution:** Use correct relative path: `require('../../package.json')`

## Performance Benchmarks
- Health check response time: < 5ms
- Hello endpoints response time: < 10ms
- Echo endpoint response time: < 10ms
- Info endpoint response time: < 15ms
- Memory usage remains stable under load