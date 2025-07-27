# MCP Tools for Task 7: Install Express.js Dependency

## Tool Selection Reasoning
This task involves installing npm packages and updating configuration files in an existing Node.js project. I selected:
- **filesystem**: Essential for creating the config directory, writing the Express configuration file, and editing package.json
- No remote tools needed as this is a local development setup task using npm commands

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files  
**Why Selected**: Required for creating configuration files and directories, and for reading/editing the package.json file  
**Task-Specific Usage**: 
- Use `create_directory` to create the src/config directory
- Use `write_file` to create the express.js configuration file
- Use `read_file` to check current package.json content
- Use `edit_file` to update package.json scripts section

**Key Operations**:
1. Create `src/config/` directory for configuration files
2. Write `src/config/express.js` with Express configuration
3. Read and update `package.json` to add new npm scripts

## Tool Usage Guidelines for This Task

### Creating Configuration Structure
```javascript
// 1. Create config directory
create_directory("hello-world-api/src/config")

// 2. Create Express configuration file
const expressConfig = `module.exports = {
  port: process.env.PORT || 3000,
  env: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'dev',
  cors: {
    origin: '*',
    credentials: true
  },
  helmet: {
    contentSecurityPolicy: false
  }
};`
write_file("hello-world-api/src/config/express.js", expressConfig)
```

### Updating Package.json Scripts
```javascript
// 3. Read current package.json
const packageJson = read_file("hello-world-api/package.json")

// 4. Update scripts section using edit_file
// Find the scripts section and update it with new commands
edit_file("hello-world-api/package.json", {
  old: '"scripts": {\n    "start": "node src/index.js"\n  }',
  new: '"scripts": {\n    "start": "node src/index.js",\n    "dev": "nodemon src/index.js",\n    "test": "echo \\"Error: no test specified\\" && exit 1"\n  }'
})
```

### Validation Steps
```javascript
// Verify configuration file
read_file("hello-world-api/src/config/express.js")

// Verify updated package.json
read_file("hello-world-api/package.json")
// Should show updated scripts and new dependencies after npm install
```

## Best Practices for This Task

1. **Directory Creation**: Check if config directory exists before creating
2. **Configuration File**: Use proper module.exports syntax for CommonJS
3. **Package.json Updates**: Preserve existing content while adding new scripts
4. **Validation**: Always verify file contents after creation/updates

## Command Line Operations
While the filesystem tools handle file operations, the actual npm installations will be done via command line:
- `npm install express --save`
- `npm install morgan body-parser cors helmet --save`
- `npm install nodemon --save-dev`

## Common Pitfalls to Avoid

1. **Don't overwrite** existing package.json content when updating scripts
2. **Ensure** proper JSON formatting when editing package.json
3. **Create** parent directories before creating files inside them
4. **Use** correct module syntax (CommonJS) for the configuration file

## Integration Notes

The filesystem tools work alongside npm commands:
1. npm handles package installation and dependency management
2. filesystem tools handle configuration file creation and package.json updates
3. Both work together to complete the task successfully

This focused tool selection provides all necessary file operations while relying on npm's native functionality for package management.