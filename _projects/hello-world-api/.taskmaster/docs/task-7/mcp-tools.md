# MCP Tools for Task 7: Install Express.js Dependency

## Tool Selection Reasoning
This task involves installing npm packages and creating configuration files. I selected:
- **filesystem**: Essential for creating the configuration directory and files, and verifying the installation results
- No remote tools needed as npm commands are executed via shell/terminal, not through MCP tools

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations including read, write, and directory management
**Why Selected**: Required for creating the configuration structure and files after packages are installed
**Task-Specific Usage**: 
- Use `create_directory` to create the src/config directory
- Use `write_file` to create the express.js configuration file
- Use `read_file` to verify package.json updates if needed
- Use `list_directory` to confirm node_modules contains installed packages

## Tool Usage Guidelines for This Task

### Package Installation Process
1. **Note**: The actual npm install commands are executed via shell/terminal, not through MCP tools
2. The filesystem tool is used for post-installation configuration

### Creating Configuration Structure
1. Use `create_directory` to create `hello-world-api/src/config` directory
2. Use `write_file` to create the Express configuration file

### Configuration File Creation
```javascript
// Create the express.js configuration file
const configContent = `module.exports = {
  port: process.env.PORT || 3000,
  env: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'dev',
  corsOptions: {
    origin: process.env.CORS_ORIGIN || '*',
    credentials: true
  }
};
`;

await filesystem.write_file({
  path: "hello-world-api/src/config/express.js",
  content: configContent
});
```

### Verification Steps
1. Use `read_file` to check updated package.json for dependencies
2. Use `list_directory` on node_modules to verify package installation
3. Use `read_file` to verify the configuration file content

## Example Tool Usage

```javascript
// Create config directory
await filesystem.create_directory({ 
  path: "hello-world-api/src/config" 
});

// Write configuration file
await filesystem.write_file({
  path: "hello-world-api/src/config/express.js",
  content: `module.exports = {
  port: process.env.PORT || 3000,
  env: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'dev',
  corsOptions: {
    origin: process.env.CORS_ORIGIN || '*',
    credentials: true
  }
};
`
});

// Verify package.json was updated
const packageJson = await filesystem.read_file({ 
  path: "hello-world-api/package.json" 
});
console.log("Updated dependencies:", JSON.parse(packageJson).dependencies);

// Check if node_modules exists and contains express
const nodeModules = await filesystem.list_directory({ 
  path: "hello-world-api/node_modules" 
});
console.log("Express installed:", nodeModules.includes("express"));
```

## Important Notes
- The filesystem tool handles configuration file creation but not npm operations
- npm install commands must be run through shell/terminal access
- Always verify package.json updates after installation
- The configuration file sets up environment-based settings for flexibility
- No remote tools needed as this task is purely local development setup