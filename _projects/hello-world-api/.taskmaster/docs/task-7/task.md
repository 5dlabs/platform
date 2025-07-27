# Task 7: Install Express.js Dependency

## Overview
This task adds Express.js and essential middleware packages to the Node.js project, establishing the foundation for building the Hello World API. It includes installing the core Express framework along with commonly used middleware for logging, security, and request handling.

## Purpose and Objectives
- Install Express.js as the main web framework for the API
- Add essential middleware packages for production-ready development
- Configure logging with Morgan for request monitoring
- Set up security headers with Helmet
- Enable CORS support for cross-origin requests
- Prepare the project for Express server implementation

## Technical Approach

### Package Selection Strategy
1. **Core Framework**: Express.js for routing and middleware support
2. **Logging**: Morgan for HTTP request logging
3. **Security**: Helmet for setting security headers
4. **CORS**: Enable cross-origin resource sharing
5. **Body Parsing**: Built-in Express middleware for JSON parsing

### Key Technical Decisions
- Use Express 4.x for stability and wide ecosystem support
- Install production dependencies with `--save` flag
- Include security middleware from the start
- Use Morgan in 'dev' format for readable logs
- Prepare configuration structure for scalability

## Implementation Details

### Step 1: Install Express.js Core
```bash
npm install express --save
```

### Step 2: Install Middleware Packages
```bash
npm install morgan cors helmet --save
```

### Step 3: Install Development Dependencies
```bash
npm install nodemon --save-dev
```

### Step 4: Create Configuration Structure
```bash
mkdir -p src/config
touch src/config/express.js
```

### Step 5: Express Configuration File
```javascript
// src/config/express.js
module.exports = {
  port: process.env.PORT || 3000,
  env: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'dev',
  corsOptions: {
    origin: process.env.CORS_ORIGIN || '*',
    credentials: true
  }
};
```

### Step 6: Updated Package.json
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple Hello World API built with Node.js and Express",
  "main": "src/index.js",
  "private": true,
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js"
  },
  "dependencies": {
    "express": "^4.19.2",
    "morgan": "^1.10.0",
    "cors": "^2.8.5",
    "helmet": "^7.1.0"
  },
  "devDependencies": {
    "nodemon": "^3.0.0"
  }
}
```

## Dependencies and Requirements

### Prerequisites
- Completed Task 6: Node.js project initialized
- Valid package.json file exists
- npm is available and configured

### Runtime Dependencies
- **express**: ^4.19.2 - Web application framework
- **morgan**: ^1.10.0 - HTTP request logger middleware
- **cors**: ^2.8.5 - Cross-origin resource sharing
- **helmet**: ^7.1.0 - Security headers middleware

### Development Dependencies
- **nodemon**: ^3.0.0 - Auto-restart server on file changes

## Testing Strategy

### Verification Steps
1. **Package Installation**
   ```bash
   # Check if packages are installed
   npm list express morgan cors helmet
   ```

2. **Dependencies Verification**
   ```bash
   # Verify package.json contains all dependencies
   cat package.json | grep -E "express|morgan|cors|helmet"
   ```

3. **Node Modules Check**
   ```bash
   # Confirm packages exist in node_modules
   ls node_modules | grep -E "^(express|morgan|cors|helmet)$"
   ```

4. **Configuration File Test**
   ```bash
   # Test configuration file syntax
   node -e "console.log(require('./src/config/express.js'))"
   ```

### Success Criteria
- ✅ Express.js installed and listed in dependencies
- ✅ All middleware packages installed successfully
- ✅ Nodemon installed as dev dependency
- ✅ Configuration file created and valid
- ✅ npm scripts updated for development workflow
- ✅ No npm vulnerabilities reported

## Related Tasks
- **Previous**: Task 6 - Initialize Node.js Project
- **Next**: Task 8 - Create Main Server File
- **Enables**: All subsequent Express.js implementation tasks

## Notes and Considerations
- Express 4.x includes body-parser functionality built-in
- Helmet should be configured appropriately for development vs production
- CORS settings may need adjustment based on client requirements
- Morgan logs should be configured differently for production
- Consider adding compression middleware for production
- Monitor for security updates to dependencies regularly