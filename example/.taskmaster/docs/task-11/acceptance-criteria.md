# Acceptance Criteria: Initialize Express TypeScript Project

## Test Cases and Validation

### 1. Project Structure Validation

#### Test Case 1.1: Directory Structure
**Given**: A new project directory
**When**: Project initialization is completed
**Then**: The following directory structure exists:
```
project-root/
├── src/
│   ├── routes/
│   ├── middleware/
│   └── types/
├── node_modules/
└── dist/ (after build)
```

**Verification Commands**:
```bash
ls -la src/
ls -la src/routes/
ls -la src/middleware/
ls -la src/types/
```

#### Test Case 1.2: Configuration Files
**Given**: Project initialization is completed
**When**: Checking for configuration files
**Then**: The following files exist and are valid:
- `package.json` - Valid JSON with required fields
- `tsconfig.json` - Valid TypeScript configuration
- `src/index.ts` - Main application entry point

**Verification Commands**:
```bash
cat package.json | jq '.'
cat tsconfig.json | jq '.'
ls -la src/index.ts
```

### 2. Dependency Management

#### Test Case 2.1: Production Dependencies
**Given**: Project dependencies are installed
**When**: Checking package.json and node_modules
**Then**: The following production dependencies are installed:
- express (^4.18.2)

**Verification Commands**:
```bash
npm list express
grep -A 5 '"dependencies"' package.json
```

#### Test Case 2.2: Development Dependencies
**Given**: Development dependencies are installed
**When**: Checking package.json and node_modules
**Then**: The following dev dependencies are installed:
- typescript (^5.1.6)
- @types/express (^4.17.17)
- @types/node (^20.4.5)
- ts-node (^10.9.1)
- nodemon (^3.0.1)

**Verification Commands**:
```bash
npm list --depth=0 --dev
grep -A 10 '"devDependencies"' package.json
```

### 3. TypeScript Configuration

#### Test Case 3.1: TypeScript Compilation
**Given**: TypeScript configuration is set up
**When**: Running TypeScript compiler
**Then**: Code compiles without errors

**Verification Commands**:
```bash
npm run build
echo $?  # Should return 0 (success)
ls -la dist/
```

#### Test Case 3.2: Type Checking
**Given**: TypeScript strict mode is enabled
**When**: Running type checker
**Then**: No type errors are reported

**Verification Commands**:
```bash
npm run type-check
npx tsc --noEmit --strict
```

### 4. Build Process

#### Test Case 4.1: Build Script
**Given**: Build script is configured
**When**: Running `npm run build`
**Then**: 
- TypeScript files are compiled to JavaScript
- Output is placed in `dist/` directory
- Process exits with code 0

**Verification Commands**:
```bash
npm run build
ls -la dist/
file dist/index.js
```

#### Test Case 4.2: Build Output Validation
**Given**: Build is completed
**When**: Checking compiled output
**Then**: 
- `dist/index.js` exists and is valid JavaScript
- Source maps are generated (if configured)
- No TypeScript files in output directory

**Verification Commands**:
```bash
node -c dist/index.js  # Check syntax
ls -la dist/*.map      # Check source maps
find dist -name "*.ts" # Should find nothing
```

### 5. Development Server

#### Test Case 5.1: Development Server Start
**Given**: Development dependencies are installed
**When**: Running `npm run dev`
**Then**: 
- Server starts without errors
- Listens on default port (3000)
- Responds to HTTP requests

**Verification Commands**:
```bash
timeout 10s npm run dev &
sleep 5
curl -s http://localhost:3000
```

#### Test Case 5.2: Hot Reload
**Given**: Development server is running
**When**: Modifying a TypeScript file
**Then**: 
- Server automatically restarts
- Changes are reflected without manual restart

**Manual Test**: 
1. Start `npm run dev`
2. Modify `src/index.ts`
3. Verify server restarts automatically
4. Confirm changes are reflected

### 6. Production Server

#### Test Case 6.1: Production Build Execution
**Given**: Production build is completed
**When**: Running `npm start`
**Then**: 
- Server starts from compiled JavaScript
- Listens on configured port
- Responds to HTTP requests

**Verification Commands**:
```bash
npm run build
timeout 10s npm start &
sleep 5
curl -s http://localhost:3000
```

### 7. Package.json Configuration

#### Test Case 7.1: Required Scripts
**Given**: Package.json is configured
**When**: Checking available scripts
**Then**: The following scripts are available:
- `start` - Production server
- `dev` - Development server  
- `build` - TypeScript compilation

**Verification Commands**:
```bash
npm run-script | grep -E "(start|dev|build)"
```

#### Test Case 7.2: Metadata Validation
**Given**: Package.json is created
**When**: Checking package metadata
**Then**: Required fields are present:
- name
- version
- description
- main
- scripts

**Verification Commands**:
```bash
cat package.json | jq '.name, .version, .description, .main, .scripts'
```

### 8. Error Handling

#### Test Case 8.1: Missing Dependencies
**Given**: Dependencies are removed
**When**: Running build or dev scripts
**Then**: Clear error messages are shown

**Test Commands**:
```bash
rm -rf node_modules
npm run build  # Should fail with clear message
npm run dev    # Should fail with clear message
```

#### Test Case 8.2: TypeScript Errors
**Given**: Invalid TypeScript code is added
**When**: Running build
**Then**: TypeScript errors are reported clearly

**Test**: Add invalid TypeScript to `src/index.ts` and run `npm run build`

### 9. Integration Tests

#### Test Case 9.1: End-to-End Setup
**Given**: Fresh project directory
**When**: Following all setup steps
**Then**: Complete working Express TypeScript application

**Steps**:
1. Initialize npm project
2. Install dependencies
3. Configure TypeScript
4. Create project structure
5. Run build
6. Start server
7. Test HTTP endpoint

#### Test Case 9.2: Clean Build
**Given**: Existing build artifacts
**When**: Running clean build
**Then**: 
- Old build artifacts are removed
- Fresh build is created
- No stale files remain

**Verification Commands**:
```bash
npm run build
touch dist/stale-file.js
npm run clean
npm run build
ls -la dist/stale-file.js  # Should not exist
```

## Acceptance Checklist

- [ ] Project structure created correctly
- [ ] All dependencies installed
- [ ] TypeScript configuration valid
- [ ] Build process works without errors
- [ ] Development server starts and responds
- [ ] Production server starts and responds
- [ ] Package.json scripts are functional
- [ ] TypeScript compilation is error-free
- [ ] Hot reload works in development
- [ ] Error handling provides clear messages
- [ ] Clean build process works
- [ ] All configuration files are valid JSON/TypeScript

## Performance Benchmarks

- Build time: < 5 seconds for initial build
- Development server startup: < 3 seconds
- Hot reload response: < 2 seconds
- Memory usage: < 50MB for basic server

## Rollback Plan

If any acceptance criteria fail:
1. Remove node_modules: `rm -rf node_modules`
2. Remove dist directory: `rm -rf dist`
3. Reset package.json to minimal state
4. Restart initialization process
5. Verify each step individually