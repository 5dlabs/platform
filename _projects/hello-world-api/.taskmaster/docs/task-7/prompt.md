# Autonomous Agent Prompt: Install Express.js Dependency

## Context
You are continuing the development of a Hello World API. The Node.js project has been initialized with a basic structure and package.json. Your task is to install Express.js and essential middleware packages that will form the foundation of the API server.

## Objective
Install and configure Express.js along with commonly used middleware packages to create a robust foundation for the API development. Set up proper dependency management and create a configuration structure for Express settings.

## Task Requirements

### 1. Install Express.js
- Navigate to the hello-world-api project directory
- Install Express.js as a production dependency
- Ensure it's properly listed in package.json

### 2. Install Essential Middleware
Install the following middleware packages as production dependencies:
- **morgan**: HTTP request logger middleware
- **body-parser**: Parse incoming request bodies
- **cors**: Enable CORS (Cross-Origin Resource Sharing)
- **helmet**: Security middleware that sets various HTTP headers

### 3. Install Development Tools
- Install **nodemon** as a development dependency for auto-restarting the server during development

### 4. Create Configuration Structure
- Create a `src/config/` directory
- Create `src/config/express.js` with the following configuration:
```javascript
module.exports = {
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
};
```

### 5. Update npm Scripts
Update the package.json scripts section to include:
```json
"scripts": {
  "start": "node src/index.js",
  "dev": "nodemon src/index.js",
  "test": "echo \"Error: no test specified\" && exit 1"
}
```

## Step-by-Step Execution

1. **Navigate to project directory**:
   ```bash
   cd hello-world-api
   ```

2. **Install Express.js**:
   ```bash
   npm install express --save
   ```

3. **Install middleware packages**:
   ```bash
   npm install morgan body-parser cors helmet --save
   ```

4. **Install development dependencies**:
   ```bash
   npm install nodemon --save-dev
   ```

5. **Create configuration directory and file**:
   ```bash
   mkdir -p src/config
   # Then create src/config/express.js with the configuration content
   ```

6. **Update package.json** with the new scripts

## Validation Criteria

### Success Indicators
- [ ] Express.js is listed in package.json dependencies
- [ ] All middleware packages are installed and listed
- [ ] nodemon is in devDependencies
- [ ] Configuration file exists at `src/config/express.js`
- [ ] npm scripts are updated with start and dev commands
- [ ] node_modules directory contains all installed packages

### Quality Checks
1. Run `npm list` to verify all packages are installed correctly
2. Check that no errors or warnings appear during installation
3. Verify package.json has clean formatting
4. Ensure configuration file has valid JavaScript syntax

## Error Handling

### Common Issues to Handle
1. **Network errors during installation**: Retry or check connection
2. **Version conflicts**: Use `npm audit fix` if needed
3. **Missing directories**: Create config directory before writing file
4. **Invalid JSON in package.json**: Validate after editing

## Expected Output

### Updated package.json structure:
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple Hello World API built with Node.js",
  "main": "src/index.js",
  "private": true,
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js",
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "dependencies": {
    "express": "^4.18.0",
    "morgan": "^1.10.0",
    "body-parser": "^1.20.0",
    "cors": "^2.8.5",
    "helmet": "^7.0.0"
  },
  "devDependencies": {
    "nodemon": "^3.0.0"
  }
}
```

### Project structure after completion:
```
hello-world-api/
├── node_modules/
├── src/
│   ├── config/
│   │   └── express.js
│   └── index.js
├── package.json
├── package-lock.json
└── .gitignore
```

## Important Notes

- Always use `--save` for production dependencies and `--save-dev` for development dependencies
- The configuration file sets up a flexible structure for future expansion
- CORS is set to allow all origins (*) for development; this should be restricted in production
- Helmet's CSP is disabled for simplicity; enable it in production applications
- Version numbers may vary; use the latest stable versions

## Tools Required
- File system access for creating directories and files
- Command execution capability for npm commands
- Text editing capability for updating JSON and creating JavaScript files

Proceed with implementing this task, ensuring all dependencies are properly installed and configured for the subsequent API development tasks.