# Acceptance Criteria: Create Main Server File (Verification)

## Definition of Done
The main server file implementation is considered complete when all the following criteria are met:

## Required Outcomes

### 1. File Structure ✓
- [ ] src/index.js exists
- [ ] File has proper JavaScript syntax
- [ ] No syntax errors when parsed

### 2. Express.js Setup ✓
- [ ] Express imported correctly
- [ ] App instance created
- [ ] PORT constant defined
- [ ] Server listening configuration present

### 3. Port Configuration ✓
- [ ] Default port is 3000
- [ ] Environment variable support: `process.env.PORT || 3000`
- [ ] Port value used consistently

### 4. Request Logging ✓
- [ ] Logging middleware implemented
- [ ] Placed before route handlers
- [ ] Logs timestamp (ISO format)
- [ ] Logs HTTP method
- [ ] Logs URL path
- [ ] Calls next() to continue

### 5. Server Startup ✓
- [ ] app.listen() implemented
- [ ] Startup message logged
- [ ] Message includes port number
- [ ] Server actually listens on specified port

### 6. Error Handling (Enhancement) ✓
- [ ] Server error event listener (optional)
- [ ] Port conflict handling (optional)
- [ ] Graceful error messages (optional)

## Test Cases

### Test Case 1: Basic Server Start
```bash
npm start
```
**Expected Output:**
```
Server running on http://localhost:3000
```

### Test Case 2: Custom Port
```bash
PORT=4000 npm start
```
**Expected Output:**
```
Server running on http://localhost:4000
```

### Test Case 3: Request Logging
```bash
# Start server, then in another terminal:
curl http://localhost:3000/test
```
**Expected Log:**
```
2024-01-15T14:32:17.123Z - GET /test
```

### Test Case 4: Port Conflict (if error handling added)
```bash
# Terminal 1
npm start

# Terminal 2
npm start
```
**Expected:** Second instance shows port conflict error

### Test Case 5: File Validation
```bash
node -c src/index.js
```
**Expected:** No syntax errors

## Code Review Checklist

### Structure
- [ ] Imports at top
- [ ] Constants defined clearly
- [ ] Middleware before routes
- [ ] Server listen at bottom

### Best Practices
- [ ] Consistent indentation
- [ ] Meaningful variable names
- [ ] Comments where helpful
- [ ] No hardcoded values (except defaults)

### Security
- [ ] No exposed sensitive data
- [ ] Port validation (if implemented)
- [ ] Safe error messages

## Comparison with Task 2

### Task 2 Requirements Met
- [x] Express server created
- [x] Request logging implemented
- [x] Server listens on port 3000
- [x] Basic functionality complete

### Task 8 Additional Requirements
- [ ] Environment variable support for PORT
- [ ] Error handling for server startup
- [ ] Enhanced startup logging

## Common Issues & Solutions

### Issue 1: PORT Not Using Environment
**Current**: `const PORT = 3000;`
**Fix**: `const PORT = process.env.PORT || 3000;`

### Issue 2: No Error Handling
**Symptom**: Server crashes on port conflict
**Fix**: Add error event listener

### Issue 3: Duplicate Implementation
**Symptom**: Task 2 already did this
**Fix**: Focus on verification and enhancement

## Performance Criteria
- [ ] Server starts in < 2 seconds
- [ ] No memory leaks on startup
- [ ] Handles immediate shutdown gracefully
- [ ] Port binding is reliable

## Integration Points

### With Routes (Tasks 3 & 4)
- [ ] Routes can be added after middleware
- [ ] Server structure supports routing
- [ ] No conflicts with route setup

### With Error Handling (Task 5)
- [ ] Error middleware can be added
- [ ] Server supports error handling
- [ ] Logging works with errors

## Sign-off Requirements
- [ ] Core server functionality verified
- [ ] Any gaps from Task 2 addressed
- [ ] Enhancements implemented if needed
- [ ] All tests pass
- [ ] Ready for route implementation