# MCP Tools for Task 7: Install Express.js Dependency

## Tool Selection Reasoning
This task involves installing npm packages and creating configuration files. While npm commands would typically be run via shell/bash, the filesystem tool is essential for:
- Creating the config directory structure
- Writing the express.js configuration file
- Reading and verifying package.json after installations
- Checking the project structure

No remote tools are needed as all operations are local package management and file operations.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Required for creating configuration directories and files, as well as verifying the installation results by reading package.json and checking file existence.

**Available Operations**:
- `create_directory`: Create the config directory structure
- `write_file`: Create the express.js configuration file
- `read_file`: Verify package.json updates and configuration
- `list_directory`: Check node_modules and project structure
- `get_file_info`: Verify configuration files exist

**Task-Specific Usage Examples**:

1. **Create Configuration Directory**:
```javascript
// Create config directory
create_directory({ path: "hello-world-api/src/config" })
```

2. **Create Express Configuration File**:
```javascript
write_file({
  path: "hello-world-api/src/config/express.js",
  content: `module.exports = {
  port: process.env.PORT || 3000,
  env: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'dev',
  
  // CORS configuration
  corsOptions: {
    origin: process.env.CORS_ORIGIN || '*',
    optionsSuccessStatus: 200
  },
  
  // Body parser limits
  bodyParserLimit: process.env.BODY_PARSER_LIMIT || '10mb',
  
  // Morgan format based on environment
  morganFormat: process.env.NODE_ENV === 'production' ? 'combined' : 'dev'
};`
})
```

3. **Verify Package Installation**:
```javascript
// Read package.json to verify dependencies
read_file({ path: "hello-world-api/package.json" })

// Check if node_modules exists
list_directory({ path: "hello-world-api/node_modules" })
```

4. **Update Package.json Scripts**:
```javascript
// Read current package.json
const packageJson = JSON.parse(read_file({ path: "hello-world-api/package.json" }))

// Update scripts
packageJson.scripts = {
  ...packageJson.scripts,
  "start": "node src/index.js",
  "dev": "nodemon src/index.js",
  "test": "echo \"Error: no test specified\" && exit 1"
}

// Write updated package.json
write_file({
  path: "hello-world-api/package.json",
  content: JSON.stringify(packageJson, null, 2)
})
```

5. **Verify Installation Results**:
```javascript
// Check that all packages are installed
list_directory({ path: "hello-world-api/node_modules" })

// Verify configuration file
get_file_info({ path: "hello-world-api/src/config/express.js" })
```

## Tool Usage Guidelines for This Task

### npm Command Execution
While filesystem tool doesn't directly run npm commands, the agent should use appropriate shell/bash commands for:
```bash
npm install express --save
npm install morgan --save
npm install body-parser cors helmet --save
npm install nodemon --save-dev
```

### File Operations Best Practices
1. **Directory Creation**: Create parent directories before writing files
2. **File Updates**: Read existing package.json before modifying to preserve other settings
3. **JSON Formatting**: Use 2-space indentation for package.json consistency
4. **Verification**: After npm installs, read package.json to confirm dependencies were added
5. **Error Handling**: Check directory existence before creating files

## Integration Notes
- The filesystem tool works in conjunction with shell commands for npm operations
- After each npm install, use filesystem to verify the changes
- Configuration files should be created after packages are installed
- Always preserve existing package.json content when updating scripts

## Common Patterns
1. Run npm install command → Read package.json to verify
2. Create directory → Write configuration file → Verify file exists
3. Update package.json scripts → Read back to confirm changes