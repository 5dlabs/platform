# MCP Tools for Task 8: Create Main Server File

## Tool Selection Reasoning
This task focuses on creating the main Express.js server file (`src/index.js`). The primary operations involve:
- Writing the server implementation code
- Reading configuration files created in previous tasks
- Verifying file creation
- Potentially updating existing files

The filesystem tool provides all necessary capabilities for these file-based operations. No remote services or external APIs are involved in creating the server file.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Essential for creating the main server file and reading any configuration files that need to be imported. This tool handles all file creation and verification operations required for the task.

**Available Operations**:
- `write_file`: Create the main server file with Express.js code
- `read_file`: Read configuration files if needed
- `get_file_info`: Verify the server file was created correctly
- `edit_file`: Modify the file if corrections are needed

**Task-Specific Usage Examples**:

1. **Create Main Server File**:
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

// Start the server
app.listen(PORT, () => {
  console.log(\`Server running on port \${PORT}\`);
});

module.exports = app;`
})
```

2. **Enhanced Server Implementation with Config**:
```javascript
// First read the config to understand settings
const config = read_file({ path: "hello-world-api/src/config/express.js" })

// Then create server file with config integration
write_file({
  path: "hello-world-api/src/index.js",
  content: `const express = require('express');
const morgan = require('morgan');
const config = require('./config/express');

const app = express();
const PORT = process.env.PORT || config.port;

// Use Morgan for request logging
if (config.env !== 'production') {
  app.use(morgan(config.morganFormat));
}

// Custom request logging middleware
app.use((req, res, next) => {
  console.log(\`\${new Date().toISOString()} - \${req.method} \${req.url}\`);
  next();
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error(\`Error: \${err.message}\`);
  res.status(500).json({
    error: 'Internal Server Error',
    message: config.env === 'development' ? err.message : undefined
  });
});

// Start server
const server = app.listen(PORT, () => {
  console.log(\`✓ Server running on port \${PORT}\`);
  console.log(\`✓ Environment: \${config.env}\`);
}).on('error', (err) => {
  console.error('✗ Server failed to start:', err.message);
  process.exit(1);
});

module.exports = app;`
})
```

3. **Verify File Creation**:
```javascript
// Check that the file was created
get_file_info({ path: "hello-world-api/src/index.js" })

// Read the file to verify content
read_file({ path: "hello-world-api/src/index.js" })
```

4. **Update Server File** (if modifications needed):
```javascript
edit_file({
  path: "hello-world-api/src/index.js",
  edits: [{
    oldText: "const PORT = 3000;",
    newText: "const PORT = process.env.PORT || 3000;"
  }]
})
```

## Tool Usage Guidelines for This Task

### File Creation Best Practices
1. **Template Literals**: Use template literals carefully with proper escaping for nested backticks
2. **Module Exports**: Always export the app for potential testing
3. **Error Handling**: Include basic error handling from the start
4. **Configuration**: Consider importing and using the config file from Task 7

### Code Organization
1. **Import Order**: 
   - Core modules first (express)
   - Installed packages (morgan)
   - Local modules (config)
2. **Middleware Order**:
   - Logging middleware before routes
   - Error handling middleware last
3. **Comments**: Include helpful comments for clarity

### Verification Steps
1. After creating the file, verify it exists
2. Read the file content to ensure proper formatting
3. Check for syntax errors by attempting to parse as JavaScript
4. Ensure all required features are implemented

## Integration Considerations
- The server file should integrate with the configuration created in Task 7
- Consider using installed middleware packages (morgan, helmet, etc.)
- Ensure compatibility with upcoming endpoint implementations
- Export the app instance for potential testing needs

## Common Patterns
1. Create file → Verify creation → Read content to confirm
2. Import config → Use config values → Provide fallbacks
3. Implement basic version → Enhance with additional features
4. Add middleware → Test ordering → Verify functionality