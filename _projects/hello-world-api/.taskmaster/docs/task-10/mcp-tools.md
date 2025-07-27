# MCP Tools for Task 10: Implement Health Check Endpoint

## Tool Selection Reasoning
This task involves modifying the existing src/index.js file to add a health check route. I selected:
- **filesystem**: Essential for reading the current file and editing it to add the new health endpoint
- No remote tools needed as this is purely code modification

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations including read, write, and directory management
**Why Selected**: Required to read the existing server file and edit it to add the health check route
**Task-Specific Usage**: 
- Use `read_file` to read the current src/index.js content and locate where to add the route
- Use `edit_file` to add the health check endpoint after the root endpoint

## Tool Usage Guidelines for This Task

### Adding the Health Check Route
1. First read the file to understand current structure
2. Locate the root endpoint (GET /)
3. Add the health check endpoint immediately after
4. Ensure proper formatting and placement

### Route Implementation
The health check route should be added after the root endpoint:
```javascript
// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});
```

## Example Tool Usage

```javascript
// First, read the current file
const currentContent = await filesystem.read_file({
  path: "hello-world-api/src/index.js"
});

// Find where the root endpoint ends
// Look for the pattern:
// app.get('/', (req, res) => {
//   res.status(200).json({ message: 'Hello, World!' });
// });

// Add the health check endpoint after it
await filesystem.edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    oldText: "// Root endpoint - Returns a welcome message to confirm the API is working\napp.get('/', (req, res) => {\n  res.status(200).json({ message: 'Hello, World!' });\n});",
    newText: "// Root endpoint - Returns a welcome message to confirm the API is working\napp.get('/', (req, res) => {\n  res.status(200).json({ message: 'Hello, World!' });\n});\n\n// Health check endpoint\napp.get('/health', (req, res) => {\n  res.status(200).json({\n    status: 'healthy',\n    timestamp: new Date().toISOString()\n  });\n});"
  }]
});
```

### Alternative Pattern Matching
If the exact text doesn't match, look for these patterns:
1. After any existing routes
2. Before error handling middleware
3. Before app.listen()

```javascript
// Alternative: Insert at a specific location
await filesystem.edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    // Find a good insertion point
    oldText: "});\n\n// Error handling middleware",
    newText: "});\n\n// Health check endpoint\napp.get('/health', (req, res) => {\n  res.status(200).json({\n    status: 'healthy',\n    timestamp: new Date().toISOString()\n  });\n});\n\n// Error handling middleware"
  }]
});
```

## Important Notes
- The health check must return exactly two fields: status and timestamp
- Status must always be "healthy" per requirements
- Timestamp must be ISO format using toISOString()
- The route must be added in the correct location
- Do not modify existing routes or middleware
- Ensure the timestamp is generated fresh on each request (inside the handler)