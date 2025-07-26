# Autonomous Task Prompt: Install Express.js Dependencies

You need to verify that Express.js is properly installed and consider whether additional middleware packages are needed for the Hello World API.

## Prerequisites
- Task 1 completed (project initialized)
- package.json exists
- Basic Express.js should already be installed

## Task Requirements

### 1. Verify Express Installation
Check that Express.js was properly installed in Task 1:

```bash
# List Express version
npm list express

# Check package.json
grep express package.json
```

Expected output:
- Express version ^4.18.2 or similar
- Listed in dependencies (not devDependencies)

### 2. Review Current Dependencies
```bash
# Show all dependencies
npm list --depth=0

# Check for vulnerabilities
npm audit
```

### 3. Evaluate Additional Packages
For this simple Hello World API, determine if any additional packages are needed:

**Already Implemented:**
- ✅ Express.js (core framework)
- ✅ Request logging (custom middleware in Task 6)
- ✅ Error handling (custom middleware in Task 5)

**Not Needed for Basic API:**
- ❌ Morgan (logging already implemented)
- ❌ Body-parser (included in Express 4.16+)
- ❌ CORS (no cross-origin requirements)
- ❌ Helmet (minimal security needs)

### 4. Verify npm Scripts
Check package.json for proper scripts:
```json
{
  "scripts": {
    "start": "node src/index.js"
  }
}
```

### 5. Optional Development Setup
If you want to add development conveniences:
```bash
# Install nodemon for auto-restart (optional)
npm install --save-dev nodemon

# Add dev script to package.json
"scripts": {
  "start": "node src/index.js",
  "dev": "nodemon src/index.js"
}
```

## Implementation Steps

### Step 1: Audit Current Setup
```bash
# Check Express installation
npm list express

# Verify no missing dependencies
npm install

# Check for security issues
npm audit
```

### Step 2: Fix Any Issues
If npm audit reports vulnerabilities:
```bash
npm audit fix
```

### Step 3: Document Dependencies
Ensure package.json is complete:
- Express in dependencies
- Correct version specified
- Scripts properly configured

### Step 4: Test Installation
```bash
# Test that Express loads
node -e "console.log(require('express').version)"

# Test server starts
npm start
```

## Decision Guide

### Should You Install Additional Packages?

**Morgan Logging Middleware:**
- ❌ Not needed - custom logging already implemented
- Would duplicate existing functionality

**Body-parser:**
- ❌ Not needed - no request body parsing required
- Built into Express 4.16+ anyway

**CORS:**
- ❌ Not needed - no cross-origin requests
- Can add later if needed

**Helmet:**
- ❌ Not needed for basic development API
- Consider for production deployment

**Nodemon:**
- ✓ Optional - useful for development
- Install as devDependency only

## Verification Checklist

### Required Checks
- [ ] Express is installed
- [ ] Express version >= 4.18.0
- [ ] Listed in package.json dependencies
- [ ] No npm vulnerabilities
- [ ] npm start script works

### Optional Checks
- [ ] package-lock.json exists
- [ ] No extraneous packages
- [ ] Dependencies are minimal
- [ ] Dev dependencies separate

## Common Issues

### Issue 1: Express Not Found
```
Error: Cannot find module 'express'
```
**Fix:** Run `npm install express`

### Issue 2: Wrong Dependency Type
Express in devDependencies instead of dependencies
**Fix:** 
```bash
npm uninstall --save-dev express
npm install express
```

### Issue 3: Vulnerabilities Reported
**Fix:** Run `npm audit fix`

### Issue 4: Version Mismatch
**Fix:** Update to latest stable:
```bash
npm install express@latest
```

## Expected package.json
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
  },
  "devDependencies": {
    "nodemon": "^3.0.1"  // Optional
  }
}
```

## Success Criteria
- Express properly installed
- No security vulnerabilities
- Minimal dependencies
- Server starts successfully
- No unnecessary packages

Since Express should already be installed from Task 1, this task is primarily about verification and cleanup.