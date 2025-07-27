# Task 8: Create Main Server File - Acceptance Criteria

## Definition of Done
The main server file is successfully created and functional when all the following criteria are met:

## Required Deliverables

### 1. File Creation
- [ ] File `src/index.js` exists
- [ ] File has proper JavaScript syntax (no parsing errors)
- [ ] File is in the correct location (src directory)

### 2. Express Server Setup
- [ ] Express is properly imported with `require('express')`
- [ ] Express app is initialized with `express()`
- [ ] PORT constant is defined with environment variable support
- [ ] Default port is 3000 when PORT env var not set

### 3. Request Logging Middleware
- [ ] Middleware function is implemented using `app.use()`
- [ ] Logs include ISO timestamp format
- [ ] Logs include HTTP method (GET, POST, etc.)
- [ ] Logs include request URL
- [ ] Middleware calls `next()` to continue processing

### 4. Server Listening
- [ ] `app.listen()` is called with PORT
- [ ] Callback function logs startup message
- [ ] Server reference is stored for error handling
- [ ] Environment is logged on startup

### 5. Error Handling
- [ ] Server error event handler is implemented
- [ ] EADDRINUSE error is handled with specific message
- [ ] EACCES error is handled with specific message
- [ ] Other errors are handled with generic message
- [ ] Process exits with code 1 on error

### 6. Graceful Shutdown
- [ ] SIGTERM signal handler is implemented
- [ ] Server close is called on SIGTERM
- [ ] Appropriate messages are logged
- [ ] Process exits cleanly with code 0

## Verification Tests

### Test 1: File Syntax Validation
```bash
# Check JavaScript syntax
node -c src/index.js
# Expected: No output (syntax is valid)
```

### Test 2: Module Loading
```bash
# Verify the module can be loaded
node -e "require('./src/index.js'); console.log('✓ Module loads successfully');"
```

### Test 3: Server Startup
```bash
# Start the server
node src/index.js &
SERVER_PID=$!
sleep 2

# Check if server is running
if ps -p $SERVER_PID > /dev/null; then
  echo "✓ Server started successfully"
  kill $SERVER_PID
else
  echo "✗ Server failed to start"
fi
```

### Test 4: Port Configuration
```bash
# Test with custom port
PORT=8080 node src/index.js &
SERVER_PID=$!
sleep 1

# Check output for correct port
if grep -q "8080" <<< "$(ps aux | grep node)"; then
  echo "✓ Custom PORT respected"
else  
  echo "✗ Custom PORT not working"
fi
kill $SERVER_PID 2>/dev/null
```

### Test 5: Request Logging
```bash
# Start server and make request
node src/index.js > server.log 2>&1 &
SERVER_PID=$!
sleep 2

# Make a test request
curl -s http://localhost:3000/ || true
sleep 1

# Check for log entry
if grep -q "GET /" server.log; then
  echo "✓ Request logging works"
else
  echo "✗ Request logging not working"
fi

kill $SERVER_PID 2>/dev/null
rm -f server.log
```

## Edge Cases to Handle

1. **Port Already in Use**
   - Server should detect EADDRINUSE
   - Clear error message should be displayed
   - Process should exit with code 1

2. **Invalid Port Number**
   - Handle non-numeric PORT values
   - Fall back to default 3000

3. **Permission Denied on Port**
   - Detect EACCES error
   - Suggest using higher port number

4. **Missing Express Module**
   - Clear error if Express not found
   - Suggest running npm install

## Success Metrics
- Server starts within 2 seconds
- No unhandled errors during startup
- All requests are logged to console
- Graceful shutdown completes within 5 seconds
- Error messages are descriptive and actionable

## Common Failure Modes

1. **Syntax Errors**
   - Missing semicolons or brackets
   - Incorrect require statements
   - Typos in method names

2. **Middleware Issues**
   - Forgetting to call next()
   - Incorrect middleware signature
   - Middleware defined after routes

3. **Port Configuration**
   - Hardcoded port instead of using PORT env
   - Invalid default port number
   - Not handling port conflicts

4. **Event Handler Problems**
   - Not storing server reference
   - Incorrect event names
   - Missing process.exit calls

## Final Validation Script
```bash
#!/bin/bash
echo "Running comprehensive server validation..."

# Check file exists
if [ ! -f "src/index.js" ]; then
  echo "✗ src/index.js not found"
  exit 1
fi

# Validate syntax
if ! node -c src/index.js 2>/dev/null; then
  echo "✗ Syntax errors in src/index.js"
  exit 1
fi

# Test server startup
node src/index.js > test.log 2>&1 &
SERVER_PID=$!
sleep 2

if ! ps -p $SERVER_PID > /dev/null; then
  echo "✗ Server failed to start"
  cat test.log
  exit 1
fi

# Test request logging
curl -s http://localhost:3000/test-path || true
sleep 1

if grep -q "GET /test-path" test.log; then
  echo "✓ Request logging verified"
else
  echo "✗ Request logging not working"
  cat test.log
  kill $SERVER_PID 2>/dev/null
  exit 1
fi

# Test graceful shutdown
kill -TERM $SERVER_PID
sleep 2

if ! ps -p $SERVER_PID > /dev/null 2>&1; then
  echo "✓ Graceful shutdown works"
else
  echo "✗ Server did not shut down gracefully"
  kill -9 $SERVER_PID 2>/dev/null
fi

rm -f test.log
echo "✅ All server validation tests passed!"
```