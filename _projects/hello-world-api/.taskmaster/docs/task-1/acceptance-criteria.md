# Acceptance Criteria: Initialize Node.js Project

## Required Deliverables

### 1. Project Structure ✓
- [ ] Directory `hello-world-api` exists
- [ ] Subdirectory `src/` exists
- [ ] File `src/index.js` exists
- [ ] File `package.json` exists
- [ ] File `.gitignore` exists
- [ ] File `README.md` exists

### 2. Package Configuration ✓
- [ ] package.json contains:
  - [ ] name: "hello-world-api"
  - [ ] version: "1.0.0"
  - [ ] description: "A simple Hello World API"
  - [ ] main: "src/index.js"
  - [ ] scripts.start: "node src/index.js"
  - [ ] dependencies includes "express"

### 3. Dependencies ✓
- [ ] Express.js is installed (verify with `npm list express`)
- [ ] node_modules directory exists
- [ ] package-lock.json exists

### 4. Server Implementation ✓
- [ ] src/index.js contains:
  - [ ] Express import statement
  - [ ] Express app initialization
  - [ ] PORT configuration (default 3000)
  - [ ] Request logging middleware
  - [ ] Server listen statement

### 5. Git Configuration ✓
- [ ] .gitignore includes:
  - [ ] node_modules/
  - [ ] .env
  - [ ] *.log
  - [ ] .DS_Store

### 6. Documentation ✓
- [ ] README.md includes:
  - [ ] Project title
  - [ ] Brief description
  - [ ] Installation instructions
  - [ ] Running instructions

## Test Cases

### Test Case 1: Project Initialization
**Steps:**
1. Navigate to project directory
2. Run `npm list`
**Expected:** Shows express in dependency tree

### Test Case 2: Server Startup
**Steps:**
1. Run `npm start`
**Expected:** 
- Console shows "Server running on http://localhost:3000"
- No errors displayed
- Server remains running

### Test Case 3: Request Logging
**Steps:**
1. Start server with `npm start`
2. Open browser to http://localhost:3000
**Expected:** Console displays timestamp, method (GET), and URL (/)

### Test Case 4: Port Configuration
**Steps:**
1. Set PORT=8080 environment variable
2. Run `npm start`
**Expected:** Server starts on port 8080

### Test Case 5: Package Scripts
**Steps:**
1. Run `npm run start`
**Expected:** Server starts successfully

## Non-Functional Requirements

### Performance
- [ ] Server starts within 2 seconds
- [ ] Request logging adds minimal latency (<5ms)

### Security
- [ ] No sensitive information in logs
- [ ] .gitignore properly configured

### Maintainability
- [ ] Code follows standard Express.js patterns
- [ ] Clear file organization
- [ ] Descriptive variable names

## Definition of Done
- [ ] All deliverables created
- [ ] All test cases pass
- [ ] Code is clean and follows conventions
- [ ] No console errors or warnings
- [ ] Project can be cloned and run with just `npm install && npm start`

## Edge Cases to Verify
1. **Port already in use**: Server should fail gracefully with clear error
2. **Missing node_modules**: Running `npm install` should restore functionality
3. **Invalid PORT value**: Should default to 3000
4. **Ctrl+C shutdown**: Server should stop cleanly