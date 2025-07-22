# Acceptance Criteria: Initialize Project and Environment Configuration

## Core Requirements

### 1. NPM Project Initialization
- [ ] `package.json` file exists in project root
- [ ] Package name is set appropriately
- [ ] Version starts at "1.0.0"
- [ ] Main entry point is "src/index.js"
- [ ] License is specified (MIT or similar)

### 2. Dependencies Installation
- [ ] **Production dependencies**:
  - [ ] express@5.x.x installed
  - [ ] dotenv@16.x.x installed
- [ ] **Development dependencies**:
  - [ ] nodemon@3.x.x installed as devDependency
- [ ] All dependencies have exact versions specified in package.json

### 3. Directory Structure
- [ ] `/src` directory exists
- [ ] `/src/routes` directory exists
- [ ] `/src/controllers` directory exists
- [ ] `/src/middleware` directory exists
- [ ] `/src/utils` directory exists
- [ ] All directories are empty but ready for use

### 4. Environment Configuration
- [ ] `.env` file exists in project root
- [ ] `.env` contains `PORT=3000`
- [ ] `.env` contains `NODE_ENV=development`
- [ ] `.env.example` file created with same structure (values can be placeholders)

### 5. Git Configuration
- [ ] `.gitignore` file exists
- [ ] `.gitignore` includes:
  - [ ] `node_modules/`
  - [ ] `.env`
  - [ ] `.DS_Store`
  - [ ] `*.log`
  - [ ] `npm-debug.log*`

### 6. Documentation
- [ ] `README.md` file exists
- [ ] README includes:
  - [ ] Project title
  - [ ] Brief description
  - [ ] Prerequisites section mentioning Node.js 18+
  - [ ] Installation instructions
  - [ ] Environment setup instructions
  - [ ] Available npm scripts
  - [ ] Basic project structure overview

### 7. Package.json Scripts
- [ ] `"start"` script defined (will be implemented in Task 2)
- [ ] `"dev"` script defined (will be implemented in Task 2)
- [ ] Scripts section exists and is properly formatted

## Test Cases

### Test 1: Verify Installation
```bash
# Should complete without errors
npm install
```

### Test 2: Check Directory Structure
```bash
# Should show all required directories
ls -la src/
```

### Test 3: Environment File
```bash
# Should contain PORT and NODE_ENV
cat .env
```

### Test 4: Git Ignore
```bash
# Verify .env is not tracked
git status
```

## Definition of Done
- All checkboxes above are checked
- No errors when running `npm install`
- Project structure matches specification exactly
- Environment variables are properly configured
- Documentation is complete and accurate