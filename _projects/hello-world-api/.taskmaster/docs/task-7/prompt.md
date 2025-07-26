# Autonomous Agent Prompt for Task 7: Install Express.js Dependency

## Task Context
You need to install Express.js and essential middleware packages for the Hello World API project. The Node.js project has already been initialized in Task 6.

## Your Mission
Install and configure Express.js with commonly used middleware packages to create a robust foundation for the API server.

## Step-by-Step Instructions

### 1. Navigate to Project Directory
```bash
cd hello-world-api
```

### 2. Install Express.js Core
```bash
npm install express --save
```

### 3. Install Logging Middleware
```bash
npm install morgan --save
```

### 4. Install Essential Middleware Packages
```bash
npm install body-parser cors helmet --save
```

### 5. Install Development Dependencies
```bash
npm install nodemon --save-dev
```

### 6. Create Configuration Directory and File
```bash
mkdir -p src/config
```

Create `src/config/express.js` with the following content:
```javascript
module.exports = {
  port: process.env.PORT || 3000,
  env: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'dev',
  
  // CORS configuration
  corsOptions: {
    origin: process.env.CORS_ORIGIN || '*',
    optionsSuccessStatus: 200
  },
  
  // Body parser limits
  bodyParserLimit: process.env.BODY_PARSER_LIMIT || '10mb',
  
  // Morgan format based on environment
  morganFormat: process.env.NODE_ENV === 'production' ? 'combined' : 'dev'
};
```

### 7. Update package.json Scripts
Ensure package.json includes these scripts:
```json
{
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js",
    "test": "echo \"Error: no test specified\" && exit 1"
  }
}
```

## Validation Steps

### 1. Verify Package Installation
```bash
# Check package.json dependencies
cat package.json | grep -E '"express"|"morgan"|"body-parser"|"cors"|"helmet"'

# Verify node_modules
ls node_modules | grep -E '^(express|morgan|body-parser|cors|helmet)$'
```

### 2. Check Dev Dependencies
```bash
# Verify nodemon in devDependencies
cat package.json | grep '"nodemon"'
```

### 3. Verify Configuration File
```bash
# Check config file exists
ls src/config/express.js
```

### 4. Test Scripts
```bash
# Test that scripts are properly configured
npm run dev -- --version
```

## Expected Result
After completion:
- `package.json` contains all required dependencies
- `node_modules` folder contains Express.js and middleware
- Configuration file exists at `src/config/express.js`
- npm scripts are updated for development workflow
- Project is ready for server implementation

## Important Notes
- Use `--save` flag to ensure dependencies are added to package.json
- Use `--save-dev` for development-only dependencies like nodemon
- Don't commit node_modules to version control
- Configuration file should use environment variables with sensible defaults
- All packages should be latest stable versions unless specified otherwise