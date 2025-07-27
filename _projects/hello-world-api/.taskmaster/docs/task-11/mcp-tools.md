# MCP Tools for Task 11: Add Error Handling

## Tool Selection Reasoning
This task involves modifying the existing src/index.js file to add error handling middleware. I selected:
- **filesystem**: Essential for reading the current file and editing it to add error handling middleware
- No remote tools needed as this is purely code modification

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations including read, write, and directory management
**Why Selected**: Required to read the existing server file and edit it to add error handling middleware
**Task-Specific Usage**: 
- Use `read_file` to understand the current file structure and find where to add middleware
- Use `edit_file` to add both the error handler and 404 handler in the correct locations

## Tool Usage Guidelines for This Task

### Adding Error Handling Middleware
1. Read the file to locate where routes end
2. Add error handling middleware after all routes
3. Add 404 handler after error handler
4. Ensure middleware is before app.listen()

### Middleware Implementation
Add these two middleware functions in order:
```javascript
// Error handling middleware
app.use((err, req, res, next) => {
  console.error(`Error: ${err.message}`);
  res.status(500).json({ error: 'Internal Server Error' });
});

// 404 handler for undefined routes
app.use((req, res) => {
  res.status(404).json({ error: 'Not Found' });
});
```

## Example Tool Usage

```javascript
// First, read the current file
const currentContent = await filesystem.read_file({
  path: "hello-world-api/src/index.js"
});

// Find where to insert error handlers
// Look for the last route (likely the health check)
// Insert after routes but before app.listen()

// Example edit to add error handlers
await filesystem.edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    // Find the end of routes, before server startup
    oldText: "});\n\n// Start the server",
    newText: "});\n\n// Error handling middleware\napp.use((err, req, res, next) => {\n  console.error(`Error: ${err.message}`);\n  res.status(500).json({ error: 'Internal Server Error' });\n});\n\n// 404 handler for undefined routes\napp.use((req, res) => {\n  res.status(404).json({ error: 'Not Found' });\n});\n\n// Start the server"
  }]
});
```

### Alternative Pattern for Finding Insert Location
```javascript
// If the exact pattern doesn't match, look for these markers:
// 1. After the last app.get() route
// 2. Before "const server = app.listen"
// 3. Before any existing error handling

await filesystem.edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    // Find where app.listen starts
    oldText: "// Start the server with error handling\nconst server = app.listen",
    newText: "// Error handling middleware\napp.use((err, req, res, next) => {\n  console.error(`Error: ${err.message}`);\n  res.status(500).json({ error: 'Internal Server Error' });\n});\n\n// 404 handler for undefined routes\napp.use((req, res) => {\n  res.status(404).json({ error: 'Not Found' });\n});\n\n// Start the server with error handling\nconst server = app.listen"
  }]
});
```

## Important Notes
- The error handler MUST have exactly 4 parameters: (err, req, res, next)
- The 404 handler MUST have exactly 2 parameters: (req, res)
- Order is critical: routes → error handler → 404 handler → app.listen()
- Do not expose stack traces in the response (security best practice)
- Log errors to console for debugging
- Both handlers must return JSON responses for consistency