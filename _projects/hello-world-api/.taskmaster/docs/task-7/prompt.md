# Autonomous AI Agent Prompt: Install Express.js Dependency

## Task Overview
You need to install Express.js and essential middleware packages for the Hello World API project. This includes the core Express framework, logging middleware, security headers, and CORS support.

## Detailed Instructions

### Step 1: Navigate to Project Directory
1. Change to the `hello-world-api` directory
2. Ensure you're in the project root (where package.json is located)

### Step 2: Install Core Express.js
1. Run the following command to install Express.js:
   ```bash
   npm install express --save
   ```
2. Verify Express appears in package.json dependencies

### Step 3: Install Essential Middleware
1. Install logging, CORS, and security middleware:
   ```bash
   npm install morgan cors helmet --save
   ```
2. These packages provide:
   - morgan: HTTP request logging
   - cors: Cross-origin resource sharing support
   - helmet: Security headers middleware

### Step 4: Install Development Dependencies
1. Install nodemon for automatic server restarts during development:
   ```bash
   npm install nodemon --save-dev
   ```

### Step 5: Create Configuration Structure
1. Create a config directory:
   ```bash
   mkdir -p src/config
   ```

2. Create Express configuration file at `src/config/express.js`:
   ```javascript
   module.exports = {
     port: process.env.PORT || 3000,
     env: process.env.NODE_ENV || 'development',
     logLevel: process.env.LOG_LEVEL || 'dev',
     corsOptions: {
       origin: process.env.CORS_ORIGIN || '*',
       credentials: true
     }
   };
   ```

### Step 6: Verify Installation
1. Check that all packages are installed:
   ```bash
   npm list --depth=0
   ```

2. Ensure no vulnerabilities:
   ```bash
   npm audit
   ```

## Expected Outcomes

1. **Updated package.json** with dependencies:
   ```json
   {
     "dependencies": {
       "express": "^4.19.2",
       "morgan": "^1.10.0",
       "cors": "^2.8.5",
       "helmet": "^7.1.0"
     },
     "devDependencies": {
       "nodemon": "^3.0.0"
     }
   }
   ```

2. **Project structure** after completion:
   ```
   hello-world-api/
   ├── node_modules/
   ├── src/
   │   ├── config/
   │   │   └── express.js
   │   └── index.js
   ├── package.json
   ├── package-lock.json
   └── .gitignore
   ```

## Validation Steps
1. Verify Express is installed:
   ```bash
   node -e "require('express'); console.log('Express loaded successfully');"
   ```

2. Check all middleware packages:
   ```bash
   node -e "require('morgan'); require('cors'); require('helmet'); console.log('All middleware loaded');"
   ```

3. Test configuration file:
   ```bash
   node -e "console.log(require('./src/config/express.js'));"
   ```

## Common Issues and Solutions

### Issue: npm install fails with permissions error
**Solution**: Ensure you have write permissions in the project directory

### Issue: Package vulnerabilities reported
**Solution**: Run `npm audit fix` to automatically fix vulnerabilities

### Issue: Wrong version of Express installed
**Solution**: Specify exact version: `npm install express@4.19.2`

### Issue: Configuration file not found
**Solution**: Ensure you created the src/config directory before the file

## Notes
- Always use `--save` for production dependencies
- Use `--save-dev` for development-only dependencies
- The configuration file prepares for environment-based settings
- Helmet will add security headers automatically when used
- CORS origin '*' is permissive; restrict in production