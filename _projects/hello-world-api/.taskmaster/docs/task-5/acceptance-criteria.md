# Acceptance Criteria: Add Error Handling and Documentation

## Required Deliverables

### 1. Error Handling Middleware ✓
- [ ] Error middleware function exists
- [ ] Has exactly 4 parameters: (err, req, res, next)
- [ ] Logs error stack to console
- [ ] Returns 500 status code
- [ ] Returns JSON error response
- [ ] Generic error message (no sensitive info)

### 2. 404 Handler Middleware ✓
- [ ] 404 handler function exists
- [ ] Placed after all routes
- [ ] Placed before app.listen()
- [ ] Returns 404 status code
- [ ] Returns JSON error response
- [ ] Message says "Not found"

### 3. Middleware Order ✓
- [ ] Request logging (first)
- [ ] Route handlers
- [ ] Error handling middleware
- [ ] 404 handler
- [ ] app.listen() (last)

### 4. README.md File ✓
- [ ] File exists in project root
- [ ] Contains project title
- [ ] Contains description
- [ ] Installation instructions
- [ ] Usage instructions
- [ ] Endpoint documentation
- [ ] Example responses

## Test Cases

### Test Case 1: 404 Response
**Steps:**
1. Start server
2. Execute: `curl -i http://localhost:3000/undefined-route`

**Expected:**
```
HTTP/1.1 404 Not Found
Content-Type: application/json

{"error":"Not found"}
```

### Test Case 2: Method Not Allowed
**Steps:**
1. Start server
2. Execute: `curl -X DELETE http://localhost:3000/`

**Expected:**
- Status: 404 Not Found
- Body: `{"error":"Not found"}`

### Test Case 3: Error Handler (Simulated)
**Steps:**
1. Add test route that throws error:
```javascript
app.get('/test-error', (req, res) => {
  throw new Error('Test error');
});
```
2. Execute: `curl -i http://localhost:3000/test-error`

**Expected:**
```
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{"error":"Something went wrong!"}
```
**Console:** Error stack trace visible

### Test Case 4: Multiple 404s
**Steps:**
1. Execute multiple undefined routes:
```bash
curl http://localhost:3000/abc
curl http://localhost:3000/xyz/123
curl http://localhost:3000/test/test/test
```

**Expected:** All return 404 with "Not found"

### Test Case 5: README Content Validation
**Steps:**
1. Open README.md
2. Verify all sections present

**Expected Sections:**
- [ ] # Hello World API
- [ ] ## Installation
- [ ] ## Usage
- [ ] ## Endpoints
- [ ] ## Example Responses

### Test Case 6: README Code Blocks
**Steps:**
1. Check README.md formatting
2. Verify code blocks use triple backticks

**Expected:**
- Installation commands in code block
- Usage commands in code block
- JSON examples in code blocks with syntax highlighting

## Error Handling Validation

### Test Case 7: Async Error Handling
**Steps:**
1. Add async route with error:
```javascript
app.get('/async-error', async (req, res) => {
  throw new Error('Async error');
});
```
2. Test the endpoint

**Expected:** 500 error response (requires express-async-errors or try/catch)

### Test Case 8: Syntax Error Protection
**Steps:**
1. Verify server doesn't crash on errors
2. Make multiple requests after an error occurs

**Expected:**
- Server continues running
- Other endpoints still work

### Test Case 9: Error Logging
**Steps:**
1. Trigger an error
2. Check console output

**Expected:**
- Error message logged
- Stack trace visible
- Timestamp included

## Documentation Quality

### Test Case 10: Installation Test
**Steps:**
1. Follow README installation steps
2. Run `npm install`

**Expected:**
- Clear instructions
- Commands work as documented
- No missing steps

### Test Case 11: Usage Test
**Steps:**
1. Follow README usage instructions
2. Run `npm start`

**Expected:**
- Server starts successfully
- Port information correct
- No confusion

### Test Case 12: Example Accuracy
**Steps:**
1. Copy example requests from README
2. Execute them

**Expected:**
- Examples match actual responses
- JSON formatting identical
- Timestamps shown as examples

## Performance Requirements

### Error Handling Performance
- [ ] Error responses < 100ms
- [ ] No memory leaks on errors
- [ ] Stack trace logging doesn't block

### Documentation Load Time
- [ ] README renders quickly
- [ ] No broken links
- [ ] Images load (if any)

## Security Validation

### Test Case 13: Error Information Disclosure
**Steps:**
1. Trigger various errors
2. Check response content

**Expected:**
- [ ] No stack traces in response
- [ ] No file paths exposed
- [ ] No internal variable names
- [ ] Generic error messages only

### Test Case 14: 404 Information Disclosure
**Steps:**
1. Try various malformed URLs
2. Check 404 responses

**Expected:**
- [ ] No path hints
- [ ] No directory listing
- [ ] Consistent error format

## Edge Cases

### Test Case 15: Long URL 404
**Steps:**
1. Request URL with 1000+ characters

**Expected:**
- Proper 404 response
- No buffer overflow
- No timeout

### Test Case 16: Special Characters
**Steps:**
1. Request: `/test<script>alert(1)</script>`
2. Check response

**Expected:**
- Safe 404 response
- No XSS vulnerability
- Proper escaping

## Integration Testing

### Test Case 17: Full API Test
**Script:**
```bash
#!/bin/bash
# Full integration test

# Test all endpoints
curl -s http://localhost:3000/ | jq .
curl -s http://localhost:3000/health | jq .
curl -s http://localhost:3000/notfound | jq .

# Test methods
curl -s -X GET http://localhost:3000/
curl -s -X POST http://localhost:3000/
curl -s -X PUT http://localhost:3000/
curl -s -X DELETE http://localhost:3000/
```

**Expected:**
- GET requests work
- Other methods return 404
- All responses are JSON

## Definition of Done
- [ ] All error handling tests pass
- [ ] 404 handler works for all cases
- [ ] README.md complete and accurate
- [ ] No sensitive information exposed
- [ ] Server remains stable
- [ ] Documentation helpful and clear
- [ ] Code follows Express best practices
- [ ] All middleware in correct order

## Maintenance Checklist
- [ ] Error messages are user-friendly
- [ ] Logging is production-ready
- [ ] README stays updated with code
- [ ] Examples remain accurate
- [ ] Security considerations documented