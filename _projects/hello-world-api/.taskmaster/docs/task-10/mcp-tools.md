# MCP Tools for Task 10: Implement Health Check Endpoint

## Tool Selection Reasoning
This task involves adding a health check endpoint to the existing Express.js server. Similar to Task 9, the primary operations are:
- Reading the current server file to locate insertion point
- Editing the file to add the health endpoint
- Verifying the implementation

The filesystem tool provides all necessary file manipulation capabilities. No remote services or external APIs are required for implementing the health check endpoint.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Required for reading and modifying the server file to add the health check endpoint. The tool's edit capabilities allow precise insertion of the new route handler.

**Available Operations**:
- `read_file`: Read current server implementation
- `edit_file`: Add health endpoint to existing code
- `write_file`: Rewrite file if major restructuring needed
- `get_file_info`: Verify modifications were saved

**Task-Specific Usage Examples**:

1. **Read Current Implementation**:
```javascript
// Understand current server structure
const serverCode = read_file({ path: "hello-world-api/src/index.js" })
```

2. **Add Basic Health Endpoint**:
```javascript
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    // Insert after root endpoint
    oldText: `// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Start the server`,
    newText: `// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Start the server`
  }]
})
```

3. **Add Enhanced Health Endpoint** (Alternative):
```javascript
// First, add os import at the top
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    oldText: "const express = require('express');",
    newText: "const express = require('express');\nconst os = require('os');"
  }]
})

// Then add enhanced health endpoint
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    oldText: `app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Start the server`,
    newText: `app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Service start time for uptime calculation
const startTime = Date.now();

// Health check endpoint with system information
app.get('/health', (req, res) => {
  const healthCheck = {
    status: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: Math.floor((Date.now() - startTime) / 1000),
    environment: process.env.NODE_ENV || 'development'
  };
  
  // Add system metrics in development
  if (healthCheck.environment !== 'production') {
    healthCheck.system = {
      memory: {
        free: Math.round(os.freemem() / 1024 / 1024),
        total: Math.round(os.totalmem() / 1024 / 1024)
      },
      cpus: os.cpus().length
    };
  }
  
  res.status(200).json(healthCheck);
});

// Start the server`
  }]
})
```

4. **Complete File Rewrite** (If needed):
```javascript
write_file({
  path: "hello-world-api/src/index.js",
  content: `const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(\`\${new Date().toISOString()} - \${req.method} \${req.url}\`);
  next();
});

// Root endpoint - Returns a welcome message to confirm the API is working
app.get('/', (req, res) => {
  res.status(200).json({ message: 'Hello, World!' });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({
    status: 'healthy',
    timestamp: new Date().toISOString()
  });
});

// Start the server
app.listen(PORT, () => {
  console.log(\`Server running on port \${PORT}\`);
});

module.exports = app;`
})
```

5. **Verify Implementation**:
```javascript
// Read updated file
const updatedCode = read_file({ path: "hello-world-api/src/index.js" })

// Verify health endpoint exists
if (updatedCode.includes("app.get('/health'")) {
  console.log("Health endpoint successfully added")
}

// Check for both endpoints
const hasRoot = updatedCode.includes("app.get('/'")
const hasHealth = updatedCode.includes("app.get('/health'")
console.log(`Endpoints: Root=${hasRoot}, Health=${hasHealth}`)
```

## Tool Usage Guidelines for This Task

### Implementation Strategy
1. **Preserve Existing Code**: Don't disrupt the root endpoint or middleware
2. **Maintain Order**: Add health endpoint after root endpoint
3. **Consider Enhancement**: Basic vs. enhanced implementation based on requirements
4. **Verify Completeness**: Ensure both endpoints exist after modification

### Best Practices
1. **Read Before Edit**: Always check current state before modifications
2. **Incremental Changes**: Use precise edits rather than full rewrites when possible
3. **Template Literal Escaping**: Properly escape backticks in code strings
4. **Consistent Formatting**: Match existing code style and indentation

### Common Patterns
1. Locate insertion point → Insert new endpoint → Verify both endpoints exist
2. For enhanced version: Add imports → Add variables → Add endpoint
3. Always verify file after changes to ensure correctness

## Integration Considerations
- Health endpoint should follow same patterns as root endpoint
- Consider whether to add system information (basic vs. enhanced)
- Ensure logging middleware captures health check requests
- Maintain consistent error handling patterns

## Error Prevention
- **Route Order**: Ensure routes are defined before app.listen()
- **JSON Format**: Use res.json() for proper Content-Type headers
- **Dynamic Values**: Ensure timestamp is generated on each request, not cached
- **ISO Format**: Use toISOString() for standard timestamp format