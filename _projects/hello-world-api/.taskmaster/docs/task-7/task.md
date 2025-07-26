# Task 7: Install Express.js Dependencies

## Overview
This task ensures that Express.js and related middleware packages are properly installed as project dependencies. While basic Express.js installation was covered in Task 1, this task focuses on verifying the installation and considering additional middleware packages that could enhance the API.

## Objectives
- Verify Express.js is properly installed
- Consider additional middleware packages
- Ensure package.json correctly lists dependencies
- Set up proper npm scripts
- Prepare for future enhancements

## Technical Approach

### 1. Core Dependencies
- **Express.js**: Web application framework (already installed in Task 1)
- **Production vs Development**: Understanding dependency categories

### 2. Optional Middleware Considerations
- **Morgan**: HTTP request logger middleware
- **Body-parser**: Parse incoming request bodies (built into Express 4.16+)
- **CORS**: Cross-Origin Resource Sharing support
- **Helmet**: Security headers middleware

### 3. Package Management
Using npm to manage dependencies ensures:
- Version control through package-lock.json
- Easy installation for other developers
- Clear distinction between production and development dependencies

## Implementation Details

### Verify Express Installation
```bash
# Check if Express is installed
npm list express

# Verify in package.json
cat package.json | grep express
```

### Consider Additional Packages (Optional)
```bash
# For production logging (optional)
npm install morgan

# For security headers (optional)
npm install helmet

# For CORS support (optional)
npm install cors
```

### Update npm Scripts
Ensure package.json has appropriate scripts:
```json
{
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js"
  }
}
```

### Development Dependencies (Optional)
```bash
# For auto-restart during development
npm install --save-dev nodemon
```

## Dependencies
- **Task 1**: Initial project setup with basic Express installation
- **npm**: Node package manager

## Success Criteria
- [ ] Express.js is listed in package.json dependencies
- [ ] Version is specified (e.g., "^4.18.2")
- [ ] node_modules contains express package
- [ ] npm start script is configured
- [ ] No npm vulnerabilities reported
- [ ] package-lock.json is present

## Testing Strategy

### Verify Installation
```bash
# Check Express version
npm list express

# Check all dependencies
npm list --depth=0

# Audit for vulnerabilities
npm audit
```

### Test Import
Create a test file to verify Express can be imported:
```javascript
// test-import.js
const express = require('express');
console.log('Express version:', express.version);
```

### Verify Scripts
```bash
# Test start script
npm start

# Test dev script (if configured)
npm run dev
```

## Package Decision Matrix

| Package | Purpose | Required | Justification |
|---------|---------|----------|---------------|
| express | Core framework | Yes | Essential for API |
| morgan | Logging | No | Custom logging already implemented |
| helmet | Security | No | Minimal API, consider for production |
| cors | CORS handling | No | Not needed for basic API |
| body-parser | Body parsing | No | Built into Express 4.16+ |

## Related Tasks
- **Task 1**: Initial Express installation
- **Task 2**: Server implementation using Express
- **Task 6**: Custom logging implementation (alternative to Morgan)
- **Task 9**: May recommend additional packages

## Notes
- Express 4.16+ includes body-parser functionality
- Morgan would duplicate custom logging from Task 6
- Additional packages can be added as needed
- Keep dependencies minimal for simple API
- Always run npm audit after installing packages