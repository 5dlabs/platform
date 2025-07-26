# Acceptance Criteria: Add Error Handling and Documentation

## Definition of Done
Error handling and documentation are considered complete when all the following criteria are met:

## Part 1: Error Handling Implementation

### 1. Error Handler Middleware ✓
- [ ] Error handler function has exactly 4 parameters: (err, req, res, next)
- [ ] Placed after all route handlers
- [ ] Placed before 404 handler
- [ ] Returns status code 500
- [ ] Returns JSON: `{"error":"Something went wrong!"}`
- [ ] Logs error stack to console

### 2. 404 Handler ✓
- [ ] Exists in the application (may be from Task 2)
- [ ] Placed after error handler
- [ ] Returns status code 404
- [ ] Returns JSON: `{"error":"Not found"}`
- [ ] No duplicate 404 handlers

### 3. Middleware Order ✓
Correct order from top to bottom:
1. Request logging
2. Route handlers
3. Error handling (4 params)
4. 404 handler (2 params)
5. app.listen()

## Part 2: README Documentation

### 1. File Location ✓
- [ ] README.md exists in project root
- [ ] File is properly formatted Markdown
- [ ] File is readable and well-structured

### 2. Required Sections ✓
- [ ] Project title: "Hello World API"
- [ ] Project description
- [ ] Installation section with npm command
- [ ] Usage section with start command
- [ ] Endpoints section listing all routes
- [ ] Example responses for both endpoints

### 3. Content Accuracy ✓
- [ ] Installation command is correct: `npm install`
- [ ] Start command is correct: `npm start`
- [ ] Port number matches server: 3000
- [ ] JSON examples match actual responses
- [ ] Endpoint paths are accurate

## Test Cases

### Test Case 1: Error Handler - Synchronous Error
```javascript
// Temporary test route
app.get('/test-error', (req, res) => {
  throw new Error('Test error');
});
```
```bash
curl -i http://localhost:3000/test-error
```
**Expected:**
- Status: 500
- Body: `{"error":"Something went wrong!"}`
- Console: Error stack trace

### Test Case 2: 404 Handler
```bash
curl -i http://localhost:3000/does-not-exist
```
**Expected:**
- Status: 404
- Body: `{"error":"Not found"}`

### Test Case 3: Error Handler - Reference Error
```javascript
// Temporary test route
app.get('/test-null', (req, res) => {
  const data = null;
  res.json(data.property); // Will throw
});
```
**Expected:** Same as Test Case 1

### Test Case 4: Multiple 404 Requests
```bash
curl http://localhost:3000/api/users
curl http://localhost:3000/admin
curl -X POST http://localhost:3000/
```
**Expected:** All return 404 with error message

### Test Case 5: README Rendering
```bash
# If markdown viewer available
mdv README.md

# Or check raw content
cat README.md | grep -E "^#|^##|^\`\`\`"
```
**Expected:** Proper Markdown structure visible

## Edge Cases

### Edge Case 1: Error After Response
```javascript
app.get('/test-late-error', (req, res) => {
  res.json({ok: true});
  throw new Error('Late error'); // After response
});
```
**Expected:** Error logged but no 500 response

### Edge Case 2: Async Errors (Not Handled)
```javascript
app.get('/test-async', async (req, res) => {
  setTimeout(() => {
    throw new Error('Async error');
  }, 100);
});
```
**Expected:** Server might crash (not handled by middleware)

### Edge Case 3: OPTIONS Requests
```bash
curl -X OPTIONS http://localhost:3000/invalid
```
**Expected:** 404 response

## Common Issues & Solutions

### Issue 1: Error Handler Never Triggered
**Symptom**: Errors cause server crash
**Cause**: Handler has wrong number of parameters
**Fix**: Must have exactly (err, req, res, next)

### Issue 2: All Routes Return 404
**Symptom**: Even valid routes return 404
**Cause**: 404 handler placed before routes
**Fix**: Move 404 handler to end

### Issue 3: Double Error Responses
**Symptom**: Client receives two responses
**Cause**: Calling next() after sending response
**Fix**: Don't call next() after res.send/json

### Issue 4: README Not Rendering on GitHub
**Symptom**: Raw Markdown shown
**Cause**: Invalid Markdown syntax
**Fix**: Check code block formatting

## Performance Criteria
- [ ] Error handling adds < 5ms overhead
- [ ] Errors don't cause memory leaks
- [ ] Server recovers after errors
- [ ] Multiple errors handled gracefully

## Security Validation
- [ ] Error messages don't expose internals
- [ ] Stack traces only in console, not response
- [ ] No sensitive data in error responses
- [ ] Generic messages for all 500 errors

## Documentation Quality
- [ ] README is beginner-friendly
- [ ] Examples are copy-pasteable
- [ ] No typos or formatting errors
- [ ] Instructions are OS-agnostic

## Integration Testing

### With Monitoring Tools
```yaml
# Example health check ignoring errors
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s
  retries: 3
```
**Expected:** Health checks continue despite errors

### With Process Managers
```bash
# PM2 example
pm2 start src/index.js
# Trigger error
# Check: pm2 status
```
**Expected:** Process stays "online"

## Sign-off Checklist
- [ ] All error scenarios handled gracefully
- [ ] 404 responses consistent
- [ ] README complete and accurate
- [ ] No regression in existing endpoints
- [ ] Console logging works correctly
- [ ] Server stable under error conditions