# Acceptance Criteria for Task 6: Initialize Node.js Project

## Required Outcomes

### 1. Directory Structure
- [ ] Project root directory `hello-world-api` exists
- [ ] Source directory `src/` exists within project root
- [ ] Main entry file `src/index.js` exists (can be empty or with placeholder comment)

### 2. Package.json Configuration
- [ ] `package.json` file exists in project root
- [ ] Package name is set to `"hello-world-api"`
- [ ] Version is set to `"1.0.0"`
- [ ] Description includes "Hello World API" or similar
- [ ] Main entry point is set to `"src/index.js"`
- [ ] Private flag is set to `true`
- [ ] Scripts section includes:
  - [ ] `"start": "node src/index.js"`
  - [ ] Optionally: `"dev": "nodemon src/index.js"`

### 3. Version Control Setup
- [ ] `.gitignore` file exists in project root
- [ ] `.gitignore` includes:
  - [ ] `node_modules/`
  - [ ] `.env`
  - [ ] `npm-debug.log`
  - [ ] `.DS_Store`
  - [ ] Common IDE directories (`.vscode/`, `.idea/`)

### 4. Project Validation
- [ ] Running `npm run start` doesn't produce syntax errors
- [ ] Project structure follows Node.js conventions
- [ ] All paths in package.json are correct and relative

## Test Cases

### Test 1: Verify Project Structure
```bash
# Check directory structure
ls -la hello-world-api/
ls -la hello-world-api/src/
# Expected: Both directories exist with proper permissions
```

### Test 2: Validate package.json
```bash
# Check package.json content
cat hello-world-api/package.json | grep '"name"'
cat hello-world-api/package.json | grep '"start"'
# Expected: Correct project name and start script
```

### Test 3: Verify .gitignore
```bash
# Check .gitignore exists and contains key entries
grep "node_modules" hello-world-api/.gitignore
grep ".env" hello-world-api/.gitignore
# Expected: Both patterns found in .gitignore
```

### Test 4: NPM Script Execution
```bash
cd hello-world-api
npm run start
# Expected: Command runs without syntax errors (may exit immediately)
```

## Definition of Done
- All directory structure requirements met
- package.json properly configured with all required fields
- .gitignore contains all necessary exclusions
- Project follows Node.js best practices
- Ready for Express.js installation (Task 7)

## Common Issues to Avoid
1. Incorrect paths in package.json
2. Missing .gitignore file
3. Improper directory structure
4. Missing or incorrect npm scripts
5. Using relative paths that break when run from different directories