# MCP Tools for Task 11: Add Error Handling

## Tool Selection Reasoning
This task involves adding error handling middleware to the Express.js server. The required operations are:
- Reading the current server file to understand its structure
- Adding error handling middleware after all routes
- Adding 404 handler as the last middleware
- Ensuring proper middleware ordering

The filesystem tool provides all necessary file manipulation capabilities. No remote services are needed for implementing error handling.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Essential for reading and modifying the server file to add error handling middleware. The tool's edit capabilities allow precise placement of middleware in the correct order.

**Available Operations**:
- `read_file`: Read current server implementation
- `edit_file`: Add error handling middleware
- `write_file`: Rewrite file if restructuring needed
- `get_file_info`: Verify file modifications

**Task-Specific Usage Examples**:

1. **Read Current Server Structure**:
```javascript
// Understand current implementation
const serverCode = read_file({ path: "hello-world-api/src/index.js" })
```

2. **Add Error Handling Middleware**:
```javascript
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    // Insert after health endpoint but before app.listen
    oldText: `// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Start the server`,
    newText: `// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error(\`Error: \${err.message}\`);
  console.error(\`Stack: \${err.stack}\`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// Start the server`
  }]
})
```

3. **Enhanced Error Handling Implementation**:
```javascript
// Add comprehensive error handling
const enhancedErrorHandling = `
// Error handling middleware
app.use((err, req, res, next) => {
  const timestamp = new Date().toISOString();
  
  // Comprehensive error logging
  console.error(\`[\${timestamp}] Error occurred:\`);
  console.error(\`  Message: \${err.message}\`);
  console.error(\`  Stack: \${err.stack}\`);
  console.error(\`  Request: \${req.method} \${req.originalUrl}\`);
  console.error(\`  IP: \${req.ip}\`);
  console.error(\`  Headers: \${JSON.stringify(req.headers)}\`);
  
  // Determine appropriate status code
  const statusCode = err.statusCode || err.status || 500;
  
  // Build error response
  const errorResponse = {
    error: statusCode === 500 ? 'Internal Server Error' : err.message,
    timestamp: timestamp
  };
  
  // Add details in development mode
  if (process.env.NODE_ENV === 'development') {
    errorResponse.debug = {
      message: err.message,
      stack: err.stack,
      code: err.code
    };
  }
  
  res.status(statusCode).json(errorResponse);
});

// 404 Not Found handler - must be last
app.use((req, res) => {
  const timestamp = new Date().toISOString();
  console.log(\`[\${timestamp}] 404 Not Found: \${req.method} \${req.originalUrl}\`);
  
  res.status(404).json({
    error: 'Not Found',
    message: \`Cannot \${req.method} \${req.originalUrl}\`,
    timestamp: timestamp
  });
});`;

// Apply the enhanced implementation
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    oldText: /* previous basic implementation */,
    newText: enhancedErrorHandling
  }]
})
```

4. **Add Test Error Route** (for validation):
```javascript
// Temporarily add test route before error handlers
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    oldText: "// Error handling middleware",
    newText: `// Test error route (remove after testing)
app.get('/test-error', (req, res, next) => {
  next(new Error('Test error for validation'));
});

// Error handling middleware`
  }]
})
```

5. **Verify Implementation**:
```javascript
// Read updated file
const updatedCode = read_file({ path: "hello-world-api/src/index.js" })

// Verify middleware order
const errorHandlerIndex = updatedCode.indexOf('app.use((err, req, res, next)')
const notFoundIndex = updatedCode.indexOf('app.use((req, res) =>')
const listenIndex = updatedCode.indexOf('app.listen(')

if (errorHandlerIndex > 0 && notFoundIndex > errorHandlerIndex && listenIndex > notFoundIndex) {
  console.log("Middleware order is correct")
} else {
  console.log("WARNING: Middleware order may be incorrect")
}
```

## Tool Usage Guidelines for This Task

### Middleware Ordering Rules
1. **Routes First**: All routes must be defined before error handlers
2. **Error Handler**: 4-parameter middleware comes after routes
3. **404 Handler**: 2-parameter catch-all comes last
4. **Before Listen**: All middleware before app.listen()

### Implementation Best Practices
1. **Parameter Count**: Error middleware must have exactly 4 parameters
2. **Logging Detail**: Include timestamp, method, path in logs
3. **Environment Aware**: Hide sensitive details in production
4. **Status Codes**: Use appropriate HTTP status codes

### Common Patterns
1. Read file → Locate insertion point → Add both handlers → Verify order
2. Use multi-line edits to add both handlers together
3. Test with temporary error route, then remove
4. Always verify middleware ordering after changes

## Error Prevention
- **Parameter Count**: Ensure error handler has (err, req, res, next)
- **Middleware Order**: Place after all routes but before listen
- **Logging Security**: Don't log sensitive headers or body data
- **Response Format**: Consistent JSON structure for all errors

## Integration Considerations
- Error handlers must not interfere with normal routes
- Logging should follow existing app patterns
- Consider adding request ID for tracking
- Plan for future error types (validation, auth)
- Ensure compatibility with monitoring tools