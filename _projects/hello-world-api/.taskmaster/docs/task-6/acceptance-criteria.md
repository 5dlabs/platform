# Task 6: Initialize Node.js Project - Acceptance Criteria

## Definition of Done
The Node.js project is successfully initialized when all the following criteria are met:

## Required Deliverables

### 1. Project Directory Structure
- [ ] Root directory `hello-world-api` exists
- [ ] Subdirectory `src` exists within the project root
- [ ] File `src/index.js` exists (can be empty)
- [ ] No extraneous directories or files are created

### 2. Package.json Configuration
- [ ] File `package.json` exists in the project root
- [ ] Valid JSON syntax (no parsing errors)
- [ ] Contains the following required fields:
  - [ ] `name` is set to "hello-world-api"
  - [ ] `version` is set to "1.0.0"
  - [ ] `description` includes meaningful project description
  - [ ] `main` points to "src/index.js"
  - [ ] `private` is set to `true`
  - [ ] `license` field is present (MIT or other valid license)

### 3. NPM Scripts
- [ ] `scripts` section exists in package.json
- [ ] `start` script is defined as "node src/index.js"
- [ ] `dev` script is optionally defined for nodemon usage
- [ ] Running `npm run` displays available scripts

### 4. Git Configuration
- [ ] File `.gitignore` exists in the project root
- [ ] Contains entry for `node_modules/`
- [ ] Contains entries for environment files (`.env`)
- [ ] Contains entries for log files (`*.log`)
- [ ] Contains OS-specific exclusions (`.DS_Store`, `Thumbs.db`)

## Verification Tests

### Test 1: Directory Structure
```bash
# Run from project parent directory
test -d hello-world-api && echo "✓ Project directory exists" || echo "✗ Project directory missing"
test -d hello-world-api/src && echo "✓ src directory exists" || echo "✗ src directory missing"
test -f hello-world-api/src/index.js && echo "✓ index.js exists" || echo "✗ index.js missing"
```

### Test 2: Package.json Validation
```bash
# Run from project directory
node -e "try { require('./package.json'); console.log('✓ Valid JSON'); } catch(e) { console.log('✗ Invalid JSON'); }"
node -e "const p = require('./package.json'); console.log(p.name === 'hello-world-api' ? '✓ Correct name' : '✗ Incorrect name');"
node -e "const p = require('./package.json'); console.log(p.scripts && p.scripts.start ? '✓ Start script exists' : '✗ Start script missing');"
```

### Test 3: NPM Scripts
```bash
# Run from project directory
npm run | grep -q "start" && echo "✓ Start script available" || echo "✗ Start script not found"
```

### Test 4: Git Ignore
```bash
# Run from project directory
test -f .gitignore && echo "✓ .gitignore exists" || echo "✗ .gitignore missing"
grep -q "node_modules" .gitignore && echo "✓ node_modules excluded" || echo "✗ node_modules not excluded"
```

## Edge Cases to Handle

1. **Existing Project Directory**
   - If directory already exists, work within it
   - Don't delete existing files unless necessary
   - Merge configurations if files already exist

2. **Existing package.json**
   - Update existing file rather than overwriting
   - Preserve custom fields not specified in requirements
   - Ensure required fields are updated

3. **Permission Issues**
   - Handle cases where directory creation might fail
   - Provide clear error messages for permission denied

## Success Metrics
- All checklist items above are marked as complete
- All verification tests pass successfully
- No errors occur during npm script execution
- Project structure is ready for Express.js installation

## Common Failure Modes
1. **Invalid JSON in package.json**: Missing commas, quotes, or brackets
2. **Wrong directory structure**: Files created in wrong locations
3. **Missing required fields**: package.json lacks essential configuration
4. **Incorrect script paths**: Start script points to wrong file location
5. **Missing .gitignore**: Version control not properly configured

## Final Validation
Run this command to perform a final check:
```bash
cd hello-world-api && \
test -f package.json && \
test -f .gitignore && \
test -d src && \
test -f src/index.js && \
npm run | grep -q "start" && \
echo "✅ All acceptance criteria met!" || \
echo "❌ Some criteria not met"
```