# Acceptance Criteria: Install Express.js Dependencies

## Definition of Done
Express.js dependencies are considered properly installed when all the following criteria are met:

## Required Outcomes

### 1. Express.js Installation ✓
- [ ] Express package exists in node_modules
- [ ] Express listed in package.json dependencies
- [ ] Version specified (e.g., "^4.18.2")
- [ ] Not in devDependencies

### 2. Package.json Validation ✓
```json
{
  "dependencies": {
    "express": "^4.18.2"
  }
}
```
- [ ] Dependencies section exists
- [ ] Express version uses caret (^) for minor updates
- [ ] No duplicate Express entries

### 3. Installation Verification ✓
- [ ] `npm list express` shows installed version
- [ ] No peer dependency warnings
- [ ] package-lock.json is present
- [ ] package-lock.json includes Express

### 4. Security Audit ✓
- [ ] `npm audit` runs without errors
- [ ] No high or critical vulnerabilities
- [ ] If vulnerabilities exist, documented why they're acceptable
- [ ] Audit report saved (optional)

## Test Cases

### Test Case 1: Verify Express Installation
```bash
npm list express
```
**Expected Output:**
```
hello-world-api@1.0.0 /path/to/hello-world-api
└── express@4.18.2
```

### Test Case 2: Check Dependencies
```bash
cat package.json | jq .dependencies
```
**Expected Output:**
```json
{
  "express": "^4.18.2"
}
```

### Test Case 3: Import Test
```bash
node -e "const e = require('express'); console.log('Express loaded:', typeof e === 'function')"
```
**Expected Output:**
```
Express loaded: true
```

### Test Case 4: Security Audit
```bash
npm audit
```
**Expected Output:**
```
found 0 vulnerabilities
```

### Test Case 5: Start Script
```bash
npm start
```
**Expected:** Server starts without module errors

## Dependency Analysis

### Required Dependencies
| Package | Version | Purpose | Status |
|---------|---------|---------|---------|
| express | ^4.18.2 | Web framework | Required |

### Evaluated but Not Needed
| Package | Reason Not Included |
|---------|-------------------|
| morgan | Custom logging already implemented |
| body-parser | Built into Express 4.16+ |
| cors | No cross-origin requirements |
| helmet | Over-engineering for simple API |

### Development Dependencies (Optional)
| Package | Version | Purpose | Status |
|---------|---------|---------|---------|
| nodemon | ^3.0.1 | Auto-restart | Optional |

## Common Issues & Solutions

### Issue 1: Multiple Express Versions
**Symptom**: npm list shows multiple versions
**Cause**: Dependency conflicts
**Fix**: 
```bash
npm dedupe
npm install
```

### Issue 2: Express in Wrong Section
**Symptom**: Express in devDependencies
**Fix**:
```bash
npm uninstall --save-dev express
npm install express
```

### Issue 3: Missing package-lock.json
**Symptom**: No lock file present
**Fix**:
```bash
npm install
git add package-lock.json
```

### Issue 4: Vulnerability Warnings
**Symptom**: npm audit shows issues
**Fix**:
```bash
npm audit fix
# If still present, document in README
```

## Package.json Validation

### Required Structure
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

### Validation Checks
- [ ] name is lowercase, no spaces
- [ ] version follows semver
- [ ] main points to correct file
- [ ] start script is defined
- [ ] dependencies object exists

## File System Validation

### Required Files
```
hello-world-api/
├── node_modules/
│   └── express/
├── package.json
├── package-lock.json
└── src/
    └── index.js
```

### Directory Checks
- [ ] node_modules exists
- [ ] node_modules/express exists
- [ ] No .git in node_modules
- [ ] .gitignore excludes node_modules

## Performance Criteria
- [ ] npm install completes in < 30 seconds
- [ ] Total dependencies < 50 packages
- [ ] node_modules size < 50MB
- [ ] No deprecated packages

## Best Practices
- [ ] Using latest stable Express 4.x
- [ ] No unnecessary dependencies
- [ ] Dependencies vs devDependencies correct
- [ ] Security vulnerabilities addressed
- [ ] Lock file committed to git

## Documentation Requirements
- [ ] README mentions Express.js
- [ ] Installation steps documented
- [ ] Required Node.js version noted
- [ ] Dependencies listed in docs

## Sign-off Checklist
- [ ] Express properly installed
- [ ] No security issues
- [ ] Minimal dependency footprint
- [ ] Server starts successfully
- [ ] Documentation updated
- [ ] Ready for development