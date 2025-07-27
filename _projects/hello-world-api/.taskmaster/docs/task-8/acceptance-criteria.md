# Acceptance Criteria: Create Main Server File

## Task Overview
**Task ID**: 8  
**Task Title**: Create Main Server File  
**Purpose**: Implement the main Express.js server file with basic middleware and startup configuration

## Prerequisites
- [ ] Task 7 completed: Express.js and dependencies installed
- [ ] `src/` directory exists in project
- [ ] Express.js available in node_modules
- [ ] package.json has "start" script configured

## Acceptance Criteria Checklist

### 1. File Creation and Structure
- [ ] **File exists**: `src/index.js` created
- [ ] **Proper imports**: Express required correctly
- [ ] **App initialized**: Express app instance created
- [ ] **Module exported**: App exported for testing

### 2. Port Configuration
- [ ] **Environment support**: Uses process.env.PORT
- [ ] **Default port**: Falls back to 3000
- [ ] **Port stored**: PORT constant defined
- [ ] **Configuration flexible**: Works with different ports

### 3. Request Logging Middleware
- [ ] **Middleware defined**: Custom logging function
- [ ] **Timestamp format**: ISO 8601 format used
- [ ] **Request details**: Logs method and URL
- [ ] **Middleware chain**: Calls next() properly
- [ ] **Global application**: Applied with app.use()

### 4. Server Startup
- [ ] **Listen configured**: app.listen() implemented
- [ ] **Port specified**: Uses PORT constant
- [ ] **Callback provided**: Success message logged
- [ ] **Server instance**: Stored for error handling

### 5. Error Handling
- [ ] **Error listener**: server.on('error') implemented
- [ ] **EACCES handled**: Permission error message
- [ ] **EADDRINUSE handled**: Port conflict message
- [ ] **Exit codes**: Proper process.exit() calls
- [ ] **Default case**: Other errors re-thrown

## Test Cases

### Test Case 1: Server Startup
**Steps**:
1. Run `npm start`
2. Observe console output

**Expected Result**:
```
Server running on port 3000
```
- No errors
- Process continues running
- Can be stopped with Ctrl+C

### Test Case 2: Request Logging
**Steps**:
1. Start server with `npm start`
2. Make HTTP request: `curl http://localhost:3000`
3. Check console output

**Expected Result**:
```
2024-01-15T10:30:45.123Z - GET /
```
- Timestamp in ISO format
- Correct HTTP method
- Correct URL path

### Test Case 3: Environment Port
**Steps**:
1. Run `PORT=5000 npm start`
2. Check startup message

**Expected Result**:
```
Server running on port 5000
```
- Server uses environment variable
- Not hardcoded to 3000

### Test Case 4: Port Already in Use
**Steps**:
1. Start server: `npm start`
2. In new terminal, start again: `npm start`
3. Observe error message

**Expected Result**:
```
Port 3000 is already in use
```
- Clear error message
- Process exits with code 1
- First server continues running

### Test Case 5: Multiple Requests
**Steps**:
1. Start server
2. Make multiple requests:
   - GET /
   - GET /test
   - POST /data

**Expected Result**:
- Each request logged separately
- Correct method for each
- Timestamps increase chronologically

## Edge Cases to Consider

### 1. Invalid PORT Environment Variable
- **Scenario**: PORT="abc" npm start
- **Expected**: Falls back to 3000 or shows error
- **Validation**: Server should handle gracefully

### 2. Privileged Port Access
- **Scenario**: PORT=80 npm start (without sudo)
- **Expected**: "Port 80 requires elevated privileges"
- **Exit Code**: 1

### 3. Rapid Requests
- **Scenario**: Multiple simultaneous requests
- **Expected**: All requests logged
- **Order**: Timestamps maintain chronological order

### 4. Special Characters in URL
- **Scenario**: Request to /test?param=value&other=123
- **Expected**: Full URL with query string logged

## Code Quality Criteria

### Structure
- [ ] **Clean imports**: Required modules at top
- [ ] **Logical flow**: Configuration, middleware, startup
- [ ] **Consistent style**: Proper indentation and formatting
- [ ] **No dead code**: All code serves a purpose

### Best Practices
- [ ] **Error handling**: Graceful failure modes
- [ ] **Logging**: Informative console messages
- [ ] **Configuration**: Environment-aware setup
- [ ] **Exports**: Module pattern for testability

### Performance
- [ ] **Lightweight middleware**: Minimal overhead
- [ ] **Synchronous logging**: Acceptable for development
- [ ] **No blocking operations**: Server remains responsive

## Security Validation

- [ ] **No sensitive data logged**: URLs only, no bodies/headers
- [ ] **No hardcoded secrets**: Port from environment
- [ ] **Safe error messages**: No system details exposed
- [ ] **Proper exits**: Clean shutdown on errors

## Definition of Done

1. **Server file created** at src/index.js
2. **All middleware functioning** correctly
3. **Server starts successfully** on configured port
4. **Request logging operational** with proper format
5. **Error handling complete** for common scenarios
6. **Code follows standards** and best practices
7. **Module exports app** for testing purposes

## Success Metrics

- **Startup Time**: < 1 second
- **Request Logging**: 100% of requests logged
- **Error Handling**: All specified errors handled
- **Code Coverage**: All paths executable

## Notes for QA/Review

- Verify ISO timestamp format is correct
- Test with various HTTP methods (GET, POST, PUT, DELETE)
- Confirm middleware ordering (logging before routes)
- Check that server doesn't crash on errors
- Validate process exit codes are correct
- Ensure no memory leaks in logging middleware
- Test concurrent request handling