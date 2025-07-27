# MCP Tools for Task 8: Create Main Server File

## Tool Selection Reasoning
This task involves creating and writing the main Express.js server file. I selected:
- **filesystem**: Essential for creating and writing the index.js file with the server implementation
- No remote tools needed as this is purely local file creation

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations including read, write, and directory management
**Why Selected**: Required for creating the main server file with all the Express.js code
**Task-Specific Usage**: 
- Use `write_file` to create src/index.js with the complete server implementation
- Use `read_file` to verify the file was created correctly if needed

## Tool Usage Guidelines for This Task

### Creating the Server File
1. Use `write_file` to create the complete `src/index.js` file
2. Ensure all code is properly formatted and syntactically correct
3. Include all required components: Express setup, middleware, error handling

### File Content Structure
The file should include:
- Express import and initialization
- PORT configuration with environment variable support
- Request logging middleware
- Server listening with startup message
- Error handling for common server errors
- Graceful shutdown handling
- Module export for testing

## Example Tool Usage

```javascript
// Complete server implementation
const serverCode = `const express = require('express');
const app = express();

// Port configuration with environment variable support
const PORT = process.env.PORT || 3000;

// Middleware for request logging
app.use((req, res, next) => {
  console.log(\`\${new Date().toISOString()} - \${req.method} \${req.url}\`);
  next();
});

// Error handling middleware (for future use)
app.use((err, req, res, next) => {
  console.error(\`\${new Date().toISOString()} - ERROR:\`, err.message);
  res.status(500).json({ error: 'Internal Server Error' });
});

// Start the server with error handling
const server = app.listen(PORT, () => {
  console.log(\`Server running on port \${PORT}\`);
  console.log(\`Environment: \${process.env.NODE_ENV || 'development'}\`);
});

// Handle server startup errors
server.on('error', (error) => {
  if (error.code === 'EADDRINUSE') {
    console.error(\`Port \${PORT} is already in use\`);
  } else if (error.code === 'EACCES') {
    console.error(\`Port \${PORT} requires elevated privileges\`);
  } else {
    console.error('Server error:', error);
  }
  process.exit(1);
});

// Graceful shutdown handling
process.on('SIGTERM', () => {
  console.log('SIGTERM signal received: closing HTTP server');
  server.close(() => {
    console.log('HTTP server closed');
    process.exit(0);
  });
});

module.exports = app; // Export for testing purposes
`;

// Write the server file
await filesystem.write_file({
  path: "hello-world-api/src/index.js",
  content: serverCode
});

// Verify the file was created
const verification = await filesystem.read_file({
  path: "hello-world-api/src/index.js"
});
console.log("Server file created successfully");
```

## Important Notes
- Ensure proper escaping of template literals in the code string
- The middleware order is crucial for proper request handling
- Error handling middleware should be placed after route handlers
- The server export enables future testing capabilities
- All logging uses console.log as specified in requirements
- The file must be syntactically valid JavaScript