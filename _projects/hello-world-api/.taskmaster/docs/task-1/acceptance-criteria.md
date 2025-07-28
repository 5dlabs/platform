# Task 1: Setup Project Structure and Dependencies - Acceptance Criteria

## Acceptance Criteria Checklist

### 1. Project Initialization ✓
- [ ] Directory `hello-world-api` exists
- [ ] File `package.json` exists in the root directory
- [ ] package.json contains valid JSON structure
- [ ] package.json has `name` field set to "hello-world-api"
- [ ] package.json has `main` field set to "src/server.js"
- [ ] package.json has `version` field (e.g., "1.0.0")
- [ ] package.json has `description` field with meaningful text

### 2. Dependencies Installation ✓
- [ ] File `package-lock.json` exists (created after npm install)
- [ ] Directory `node_modules` exists and contains installed packages
- [ ] Core dependencies in package.json:
  - [ ] express@4.18.2
  - [ ] cors@2.8.5
  - [ ] helmet@7.0.0
  - [ ] pino@8.15.0
  - [ ] pino-http@8.5.0
  - [ ] dotenv@16.3.1
- [ ] Dev dependencies in package.json:
  - [ ] jest@29.6.4
  - [ ] supertest@6.3.3
  - [ ] nodemon@3.0.1
  - [ ] eslint@8.48.0
  - [ ] swagger-jsdoc@6.2.8
  - [ ] swagger-ui-express@5.0.0

### 3. Directory Structure ✓
- [ ] Directory `src/` exists
  - [ ] Directory `src/middleware/` exists
  - [ ] Directory `src/routes/` exists
  - [ ] Directory `src/utils/` exists
  - [ ] File `src/app.js` exists
  - [ ] File `src/server.js` exists
- [ ] Directory `tests/` exists
  - [ ] Directory `tests/unit/` exists
  - [ ] Directory `tests/integration/` exists
- [ ] Directory `docs/` exists
  - [ ] File `docs/openapi.yaml` exists

### 4. Configuration Files ✓
- [ ] File `.env` exists with required variables:
  - [ ] PORT=3000
  - [ ] NODE_ENV=development
  - [ ] API_VERSION=1.0.0
  - [ ] LOG_LEVEL=info
- [ ] File `.gitignore` exists with proper patterns:
  - [ ] Contains `node_modules/`
  - [ ] Contains `.env`
  - [ ] Contains `coverage/`
  - [ ] Contains standard IDE patterns
- [ ] File `.dockerignore` exists with proper patterns:
  - [ ] Contains `node_modules/`
  - [ ] Contains `.env`
  - [ ] Contains `.git/`
  - [ ] Contains `tests/`

### 5. Project Files ✓
- [ ] File `README.md` exists
- [ ] File `Dockerfile` exists (can be empty)
- [ ] File `kubernetes.yaml` exists (can be empty)

### 6. NPM Scripts Configuration ✓
- [ ] package.json contains `scripts` section
- [ ] Script `start` is defined as `node src/server.js`
- [ ] Script `dev` is defined as `nodemon src/server.js`
- [ ] Script `test` is defined as `jest --coverage`
- [ ] Script `lint` is defined as `eslint .`

## Test Cases

### Test Case 1: Verify Project Structure
```bash
# Run from project root
test -d src/middleware && echo "✓ src/middleware exists" || echo "✗ src/middleware missing"
test -d src/routes && echo "✓ src/routes exists" || echo "✗ src/routes missing"
test -d src/utils && echo "✓ src/utils exists" || echo "✗ src/utils missing"
test -f src/app.js && echo "✓ src/app.js exists" || echo "✗ src/app.js missing"
test -f src/server.js && echo "✓ src/server.js exists" || echo "✗ src/server.js missing"
```

### Test Case 2: Verify Dependencies
```bash
# Check if all core dependencies are listed
npm list express cors helmet pino pino-http dotenv
```

### Test Case 3: Verify Dev Dependencies
```bash
# Check if all dev dependencies are listed
npm list --dev jest supertest nodemon eslint swagger-jsdoc swagger-ui-express
```

### Test Case 4: Verify Scripts
```bash
# Check if all scripts are defined
npm run | grep -E "(start|dev|test|lint)"
```

### Test Case 5: Verify Environment Configuration
```bash
# Check .env file contents
grep -E "PORT|NODE_ENV|API_VERSION|LOG_LEVEL" .env
```

### Test Case 6: Validate package.json Structure
```bash
# Validate JSON structure
node -e "console.log(JSON.parse(require('fs').readFileSync('package.json', 'utf8')).name)"
# Should output: hello-world-api
```

## Validation Commands

Run these commands to validate the setup:
```bash
# 1. Check all directories exist
find . -type d -name "middleware" -o -name "routes" -o -name "utils" -o -name "unit" -o -name "integration" | wc -l
# Expected output: 5

# 2. Check all required files exist
ls -la .env .gitignore .dockerignore Dockerfile kubernetes.yaml README.md package.json package-lock.json
# All files should be listed

# 3. Verify npm scripts work
npm run --silent 2>&1 | grep -c "Lifecycle scripts included in hello-world-api"
# Expected output: 1

# 4. Check dependency count
npm list --depth=0 2>/dev/null | grep -E "├──|└──" | wc -l
# Should show total number of direct dependencies (12)
```

## Success Indicators
- ✅ All directories created according to specification
- ✅ All dependencies installed with correct versions
- ✅ All configuration files present with required content
- ✅ NPM scripts properly configured
- ✅ No errors when running `npm install`
- ✅ Project ready for development work

## Common Issues and Solutions

### Issue 1: Missing package-lock.json
**Solution:** Run `npm install` to generate the lock file

### Issue 2: Script commands not found
**Solution:** Ensure package.json scripts section is properly formatted with comma separators

### Issue 3: Environment variables not loading
**Solution:** Verify .env file is in root directory and has proper format (KEY=value)

### Issue 4: Dependencies version mismatch
**Solution:** Delete node_modules and package-lock.json, then run `npm install` again with exact versions