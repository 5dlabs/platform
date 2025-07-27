# MCP Tools for Task 8: Create Main Server File

## Tool Selection Reasoning
This task involves creating the main Express.js server file with middleware and startup configuration. I selected:
- **filesystem**: Essential for creating and writing the main server file (src/index.js)
- No remote tools needed as this is purely a code implementation task

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files  
**Why Selected**: Required to create the main server file and potentially verify directory structure  
**Task-Specific Usage**: 
- Use `create_directory` to ensure src/ directory exists
- Use `write_file` to create src/index.js with the Express server code
- Use `read_file` to verify the created file if needed

**Key Operations**:
1. Ensure `src/` directory exists
2. Create `src/index.js` with complete Express server implementation
3. Verify file was created successfully

## Tool Usage Guidelines for This Task

### Creating the Server File
```javascript
// 1. Ensure src directory exists (may already exist from previous tasks)
create_directory("hello-world-api/src")

// 2. Create the main server file with complete implementation
const serverCode = `const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(\`\${new Date().toISOString()} - \${req.method} \${req.url}\`);
  next();
});

// Error handling for server startup
const server = app.listen(PORT, () => {
  console.log(\`Server running on port \${PORT}\`);
});

server.on('error', (error) => {
  if (error.syscall !== 'listen') {
    throw error;
  }

  switch (error.code) {
    case 'EACCES':
      console.error(\`Port \${PORT} requires elevated privileges\`);
      process.exit(1);
      break;
    case 'EADDRINUSE':
      console.error(\`Port \${PORT} is already in use\`);
      process.exit(1);
      break;
    default:
      throw error;
  }
});

module.exports = app;`

write_file("hello-world-api/src/index.js", serverCode)
```

### Validation Steps
```javascript
// 3. Verify the file was created
read_file("hello-world-api/src/index.js")
// Should display the complete server code
```

## Best Practices for This Task

1. **File Creation**: Ensure parent directory exists before creating file
2. **Code Formatting**: Maintain proper indentation and formatting in the generated code
3. **Error Handling**: Include comprehensive error handling in the server code
4. **Module Pattern**: Export the app instance for testing purposes

## Common Pitfalls to Avoid

1. **Don't forget** to escape template literals properly in the code string
2. **Ensure** the middleware calls next() to continue the request chain
3. **Remember** to handle both EACCES and EADDRINUSE errors
4. **Include** module.exports for testability

## Code Generation Notes

The generated server file includes:
- Express app initialization
- Environment-aware port configuration
- Request logging middleware with ISO timestamps
- Server startup with success logging
- Comprehensive error handling for common startup issues
- Module export for testing

This minimal tool selection focuses on the essential file creation operation needed to implement the Express.js server.