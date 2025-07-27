# Task 7: Install Express.js Dependency - Acceptance Criteria

## Definition of Done
Express.js and all required middleware packages are successfully installed when all the following criteria are met:

## Required Deliverables

### 1. Core Dependencies Installed
- [ ] Express.js package is installed
- [ ] Morgan logging middleware is installed
- [ ] CORS middleware is installed
- [ ] Helmet security middleware is installed
- [ ] All packages appear in node_modules directory

### 2. Development Dependencies
- [ ] Nodemon is installed as a dev dependency
- [ ] Appears in devDependencies section of package.json

### 3. Package.json Updates
- [ ] Dependencies section includes:
  - [ ] "express": version ^4.0.0 or higher
  - [ ] "morgan": version ^1.10.0 or higher
  - [ ] "cors": version ^2.8.0 or higher
  - [ ] "helmet": version ^7.0.0 or higher
- [ ] DevDependencies section includes:
  - [ ] "nodemon": version ^3.0.0 or higher

### 4. Configuration Files
- [ ] Directory `src/config` exists
- [ ] File `src/config/express.js` exists
- [ ] Configuration file exports valid JavaScript object
- [ ] Configuration includes port, env, and logLevel settings

### 5. Package Lock File
- [ ] `package-lock.json` file is created/updated
- [ ] Lock file is consistent with package.json

## Verification Tests

### Test 1: Verify Express Installation
```bash
# Check Express is installed and loadable
node -e "try { require('express'); console.log('✓ Express installed'); process.exit(0); } catch(e) { console.log('✗ Express not found'); process.exit(1); }"
```

### Test 2: Verify Middleware Installation
```bash
# Check all middleware packages
node -e "
  const packages = ['morgan', 'cors', 'helmet'];
  let missing = [];
  packages.forEach(pkg => {
    try { require(pkg); } catch(e) { missing.push(pkg); }
  });
  if (missing.length === 0) {
    console.log('✓ All middleware installed');
    process.exit(0);
  } else {
    console.log('✗ Missing packages:', missing.join(', '));
    process.exit(1);
  }
"
```

### Test 3: Verify Package.json Dependencies
```bash
# Check package.json has correct dependencies
node -e "
  const pkg = require('./package.json');
  const required = ['express', 'morgan', 'cors', 'helmet'];
  const missing = required.filter(dep => !pkg.dependencies[dep]);
  if (missing.length === 0) {
    console.log('✓ All dependencies in package.json');
  } else {
    console.log('✗ Missing from package.json:', missing.join(', '));
  }
"
```

### Test 4: Verify Development Dependencies
```bash
# Check nodemon is in devDependencies
node -e "
  const pkg = require('./package.json');
  if (pkg.devDependencies && pkg.devDependencies.nodemon) {
    console.log('✓ Nodemon in devDependencies');
  } else {
    console.log('✗ Nodemon not in devDependencies');
  }
"
```

### Test 5: Verify Configuration File
```bash
# Check configuration file exists and is valid
node -e "
  try {
    const config = require('./src/config/express.js');
    if (config.port && config.env && config.logLevel) {
      console.log('✓ Configuration file valid');
    } else {
      console.log('✗ Configuration missing required fields');
    }
  } catch(e) {
    console.log('✗ Configuration file not found or invalid');
  }
"
```

## Edge Cases to Handle

1. **Network Issues During Installation**
   - Retry npm install if network timeout occurs
   - Use npm cache if available
   - Provide clear error messages

2. **Version Conflicts**
   - Ensure compatible versions are installed
   - Handle peer dependency warnings
   - Document any version constraints

3. **Existing Dependencies**
   - Don't downgrade existing packages
   - Preserve custom configurations
   - Update only specified packages

4. **Permission Issues**
   - Handle npm permission errors gracefully
   - Suggest solutions for common permission problems

## Success Metrics
- All verification tests pass (return 0 exit code)
- No npm vulnerabilities at high or critical level
- package-lock.json is generated and valid
- All packages are actually usable (can be required)

## Common Failure Modes

1. **Network Timeout**: npm install fails due to network issues
2. **Permission Denied**: Cannot write to node_modules or package.json
3. **Version Mismatch**: Incompatible versions between packages
4. **Missing Configuration**: Config file not created or malformed
5. **Corrupted Cache**: npm cache causes installation issues

## Final Validation
Run this comprehensive check:
```bash
# Final validation script
node -e "
  let success = true;
  const checks = [];
  
  // Check Express
  try { require('express'); checks.push('✓ Express'); } 
  catch(e) { checks.push('✗ Express'); success = false; }
  
  // Check middleware
  ['morgan', 'cors', 'helmet'].forEach(pkg => {
    try { require(pkg); checks.push('✓ ' + pkg); } 
    catch(e) { checks.push('✗ ' + pkg); success = false; }
  });
  
  // Check config
  try {
    const config = require('./src/config/express.js');
    if (config.port) checks.push('✓ Config file');
    else { checks.push('✗ Config invalid'); success = false; }
  } catch(e) { checks.push('✗ Config missing'); success = false; }
  
  // Report
  console.log('Validation Results:');
  checks.forEach(check => console.log('  ' + check));
  console.log('\nOverall:', success ? '✅ PASSED' : '❌ FAILED');
  process.exit(success ? 0 : 1);
"
```