# Acceptance Criteria: Implement Hello Endpoint

## Overview
This document defines the acceptance criteria for Task 3: Implement Hello Endpoint. All criteria must be met for the task to be considered complete.

## Acceptance Criteria

### 1. Endpoint Configuration
- [ ] GET route is defined for path `/`
- [ ] Route handler uses `app.get()` method
- [ ] Route is defined after middleware but before 404 handler
- [ ] Previous placeholder route is completely removed

### 2. Response Format
- [ ] Response is in JSON format
- [ ] Response uses `res.json()` method
- [ ] JSON structure matches: `{ "message": "Hello, World!" }`
- [ ] Message text is exactly "Hello, World!" (with comma and exclamation)
- [ ] No additional fields in JSON response

### 3. HTTP Status
- [ ] Response status code is 200 OK
- [ ] Status is explicitly set using `res.status(200)`
- [ ] No redirects or other status codes

### 4. Content Headers
- [ ] Content-Type header is `application/json`
- [ ] Character encoding is UTF-8
- [ ] Headers are automatically set by Express

### 5. HTTP Method Support
- [ ] Only GET method is supported
- [ ] POST to `/` returns 404
- [ ] PUT to `/` returns 404  
- [ ] DELETE to `/` returns 404
- [ ] PATCH to `/` returns 404

## Test Cases

### Test Case 1: Basic GET Request
```bash
curl http://localhost:3000/
```
**Expected Response:**
```json
{"message":"Hello, World!"}
```
**Expected Status:** 200 OK

### Test Case 2: Response Headers
```bash
curl -I http://localhost:3000/
```
**Expected Headers Include:**
```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
```

### Test Case 3: JSON Validation
```bash
curl -s http://localhost:3000/ | python -m json.tool
```
**Expected Output:**
```json
{
    "message": "Hello, World!"
}
```
**Note:** Should parse without errors

### Test Case 4: Exact Message Format
```bash
curl -s http://localhost:3000/ | jq -r '.message'
```
**Expected Output:**
```
Hello, World!
```
**Validation Points:**
- Capital H in Hello
- Comma after Hello
- Space after comma
- Capital W in World
- Exclamation mark at end

### Test Case 5: POST Method Rejection
```bash
curl -X POST http://localhost:3000/
```
**Expected Response:**
```json
{"error":"Not found"}
```
**Expected Status:** 404 Not Found

### Test Case 6: Request Logging
```bash
curl http://localhost:3000/
```
**Expected Server Log:**
```
2024-01-15T10:30:45.123Z - GET /
```

### Test Case 7: Response Time
```bash
time curl -s http://localhost:3000/
```
**Expected:** Response time < 50ms

### Test Case 8: Concurrent Requests
```bash
# Run 10 concurrent requests
for i in {1..10}; do curl -s http://localhost:3000/ & done; wait
```
**Expected:** All requests return identical correct response

## Validation Script

Save as `validate-hello-endpoint.js`:

```javascript
const http = require('http');
const assert = require('assert');

function validateHelloEndpoint() {
  const options = {
    hostname: 'localhost',
    port: 3000,
    path: '/',
    method: 'GET'
  };

  const req = http.request(options, (res) => {
    let data = '';

    // Check status code
    assert.strictEqual(res.statusCode, 200, 'Status code should be 200');
    
    // Check content type
    assert(res.headers['content-type'].includes('application/json'), 
           'Content-Type should be application/json');

    res.on('data', (chunk) => {
      data += chunk;
    });

    res.on('end', () => {
      try {
        // Parse JSON
        const json = JSON.parse(data);
        
        // Validate structure
        assert.strictEqual(Object.keys(json).length, 1, 
                          'Response should have exactly one field');
        assert(json.hasOwnProperty('message'), 
               'Response should have "message" field');
        assert.strictEqual(json.message, 'Hello, World!', 
                          'Message should be exactly "Hello, World!"');
        
        console.log('✓ All tests passed!');
        console.log('Response:', json);
      } catch (error) {
        console.error('✗ Test failed:', error.message);
        process.exit(1);
      }
    });
  });

  req.on('error', (error) => {
    console.error('✗ Request failed:', error.message);
    process.exit(1);
  });

  req.end();
}

// Run validation
console.log('Validating Hello endpoint...\n');
validateHelloEndpoint();
```

Run with: `node validate-hello-endpoint.js`

## Performance Criteria

- [ ] Response time < 50ms for single request
- [ ] Can handle 100 requests/second
- [ ] No memory leaks during extended operation
- [ ] CPU usage remains low during normal load

## Security Criteria

- [ ] No user input is processed (prevents injection)
- [ ] No sensitive information in response
- [ ] Standard Express security headers present
- [ ] No CORS headers (unless specifically required)

## Code Quality Criteria

- [ ] Route handler is concise and readable
- [ ] Proper indentation maintained
- [ ] Descriptive comment above route
- [ ] No console.log statements in route handler
- [ ] No unused variables or parameters

## Definition of Done

The task is complete when:

1. All checkbox criteria are met
2. All test cases pass
3. Validation script runs successfully
4. Endpoint has been tested manually via browser
5. Code review confirms proper implementation
6. No regression in existing functionality (logging, 404 handler)

## Regression Tests

Ensure existing functionality still works:

1. **Logging still functions**:
   ```bash
   curl http://localhost:3000/
   # Check server console for log entry
   ```

2. **404 handler still works**:
   ```bash
   curl http://localhost:3000/nonexistent
   # Should return {"error":"Not found"}
   ```

3. **Server starts without errors**:
   ```bash
   npm start
   # No errors in console
   ```

## Browser Testing

1. Open http://localhost:3000/ in:
   - Chrome
   - Firefox
   - Safari (if available)
   - Edge

2. Verify JSON is displayed (may be formatted by browser)

3. Open Developer Tools > Network tab:
   - Check response headers
   - Verify 200 status
   - Confirm application/json content type