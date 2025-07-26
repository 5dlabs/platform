# Acceptance Criteria: Initialize Node.js Project

## Definition of Done
The Node.js project is considered successfully initialized when all the following criteria are met:

## Required Outcomes

### 1. Project Directory Structure ✓
- [ ] Directory `hello-world-api` exists
- [ ] Directory `src` exists within project root
- [ ] File `src/index.js` exists with basic Express server code
- [ ] File `package.json` exists with correct configuration
- [ ] File `package-lock.json` exists (auto-generated)
- [ ] File `.gitignore` exists with proper entries
- [ ] File `README.md` exists with basic content

### 2. Package.json Configuration ✓
```json
{
  "name": "hello-world-api",
  "version": "1.0.0",
  "description": "A simple Hello World API",
  "main": "src/index.js",
  "scripts": {
    "start": "node src/index.js"
  },
  "dependencies": {
    "express": "^4.18.2"
  }
}
```

### 3. Dependencies Installed ✓
- [ ] Express.js is listed in dependencies
- [ ] node_modules directory exists
- [ ] Running `npm list express` shows installed version

### 4. Git Configuration ✓
- [ ] .gitignore contains `node_modules/`
- [ ] .gitignore contains `.env`

### 5. Server Functionality ✓
- [ ] Running `npm start` starts the server without errors
- [ ] Server listens on port 3000 (or PORT env variable)
- [ ] Console shows "Server running on http://localhost:3000"
- [ ] Request logging middleware is configured
- [ ] Making a request logs to console with timestamp

## Test Cases

### Test Case 1: Verify Project Structure
```bash
# Expected: All directories and files exist
ls -la hello-world-api/
ls -la hello-world-api/src/
```

### Test Case 2: Verify Dependencies
```bash
cd hello-world-api
npm list
# Expected: express@4.x.x listed
```

### Test Case 3: Start Server
```bash
npm start
# Expected: "Server running on http://localhost:3000"
```

### Test Case 4: Test Request Logging
```bash
# In another terminal:
curl http://localhost:3000
# Expected: Console shows logged request with timestamp
```

### Test Case 5: Verify Package Scripts
```bash
npm run start
# Expected: Server starts successfully
```

## Validation Checklist

### Pre-Implementation
- [ ] Node.js version 20+ installed
- [ ] npm is available
- [ ] No existing hello-world-api directory

### Post-Implementation
- [ ] All files created as specified
- [ ] No npm errors during installation
- [ ] Server starts without errors
- [ ] Port 3000 is available (no conflicts)
- [ ] All test cases pass

## Common Issues & Solutions

### Issue 1: Port Already in Use
**Error**: `Error: listen EADDRINUSE: address already in use :::3000`
**Solution**: 
- Kill process using port 3000
- Or set PORT environment variable: `PORT=3001 npm start`

### Issue 2: npm Install Fails
**Error**: Network or permission errors
**Solution**:
- Check internet connection
- Run with sudo if permission denied
- Clear npm cache: `npm cache clean --force`

### Issue 3: Module Not Found
**Error**: `Error: Cannot find module 'express'`
**Solution**:
- Ensure npm install completed successfully
- Delete node_modules and reinstall: `rm -rf node_modules && npm install`

## Sign-off Criteria
- [ ] Developer: All acceptance criteria met
- [ ] Code Review: Project structure follows standards
- [ ] Testing: All test cases pass
- [ ] Documentation: README.md is present and accurate