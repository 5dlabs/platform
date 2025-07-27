# MCP Tools for Task 11: Add Error Handling

## Tool Selection Reasoning
This task involves adding error handling middleware to the existing Express.js server. I selected:
- **filesystem**: Essential for reading the current server file and editing it to add error handling middleware
- No remote tools needed as this is a code modification task

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files  
**Why Selected**: Required to read the existing server file and make precise edits to add the error handling middleware  
**Task-Specific Usage**: 
- Use `read_file` to examine the current src/index.js structure
- Use `edit_file` to add error handling middleware in the correct location

**Key Operations**:
1. Read current `src/index.js` to locate insertion point
2. Add error handling middleware after all routes
3. Add 404 handler as the very last middleware
4. Verify the changes were applied correctly

## Tool Usage Guidelines for This Task

### Reading and Adding Error Handling
```javascript
// 1. First read the current file to understand its structure
const currentCode = read_file("hello-world-api/src/index.js")

// 2. Add error handling after all routes but before server.listen()
// Look for the end of routes and add both middleware handlers
edit_file("hello-world-api/src/index.js", {
  old: `// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Error handling for server startup`,
  new: `// Health check endpoint
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
  console.error(\`Request path: \${req.path}\`);
  console.error(\`Request method: \${req.method}\`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});

// Error handling for server startup`
})
```

### Key Middleware Components
```javascript
// Error handling middleware (4 parameters - REQUIRED)
const errorHandler = `app.use((err, req, res, next) => {
  console.error(\`Error: \${err.message}\`);
  console.error(\`Stack: \${err.stack}\`);
  console.error(\`Request path: \${req.path}\`);
  console.error(\`Request method: \${req.method}\`);
  res.status(500).json({ error: 'Internal Server Error' });
});`

// 404 handler (2 parameters - must be last)
const notFoundHandler = `app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});`
```

### Validation Steps
```javascript
// 3. Read the file again to verify both handlers were added
read_file("hello-world-api/src/index.js")
// Should now include:
// - Error handling middleware with 4 parameters
// - 404 handler as the last middleware
```

## Best Practices for This Task

1. **Middleware Order**: Ensure error handler comes after routes but before 404
2. **Parameter Count**: Error middleware MUST have exactly 4 parameters
3. **404 Placement**: Must be the very last middleware
4. **Logging Details**: Include comprehensive error information for debugging

## Common Pitfalls to Avoid

1. **Don't forget** the 4-parameter signature for error middleware
2. **Place** 404 handler absolutely last in the middleware chain
3. **Avoid** exposing stack traces in the response (log only)
4. **Ensure** proper escaping of template literals in the edit

## Middleware Integration Notes

The error handling setup:
- Error middleware catches any errors passed with next(err)
- 404 handler catches all requests that don't match any route
- Both return consistent JSON error responses
- Logging provides debugging information without exposing it to clients

This minimal tool selection focuses on the essential file editing operations needed to add comprehensive error handling to the Express.js server.