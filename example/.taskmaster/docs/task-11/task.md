# Task 11: Initialize Express TypeScript Project

## Overview
Set up a new Express.js project with TypeScript configuration, including package.json, tsconfig.json, and basic project structure.

## Description
This task involves creating a foundational Express.js application with TypeScript support. It includes configuring the development environment, installing necessary dependencies, and establishing a proper project structure for scalable development.

## Priority
High

## Dependencies
None - This is a foundational task

## Implementation Steps

### 1. Initialize npm project
- Run `npm init -y` to create package.json
- Configure package.json with appropriate metadata including name, version, description, and scripts

### 2. Install dependencies
- **Production dependencies**: Express.js framework
- **Development dependencies**: TypeScript, @types/express, @types/node, ts-node, nodemon
- Use `npm install` for production and `npm install --save-dev` for development dependencies

### 3. Configure TypeScript
- Create `tsconfig.json` with Node.js and Express appropriate settings
- Configure target, module, outDir, rootDir, and strict type checking
- Enable ES6+ features and proper module resolution

### 4. Create project structure
- Set up `src/` directory as the main source folder
- Create subdirectories:
  - `src/routes/` - for route handlers
  - `src/middleware/` - for custom middleware
  - `src/types/` - for TypeScript type definitions
- Create main entry point `src/index.ts`

## Implementation Details

### Package.json Configuration
```json
{
  "name": "express-typescript-api",
  "version": "1.0.0",
  "description": "Express.js API with TypeScript",
  "main": "dist/index.js",
  "scripts": {
    "start": "node dist/index.js",
    "dev": "nodemon src/index.ts",
    "build": "tsc",
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "dependencies": {
    "express": "^4.18.2"
  },
  "devDependencies": {
    "@types/express": "^4.17.17",
    "@types/node": "^20.4.5",
    "nodemon": "^3.0.1",
    "ts-node": "^10.9.1",
    "typescript": "^5.1.6"
  }
}
```

### TypeScript Configuration
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

## Test Strategy
- Verify project builds without errors using `npm run build`
- Ensure development server starts successfully with `npm run dev`
- Confirm TypeScript compilation passes without type errors
- Test that the basic Express server can be instantiated

## Expected Outcomes
- Functional Express.js application with TypeScript support
- Proper development and build scripts configured
- Clean project structure ready for feature development
- All dependencies properly installed and configured

## Common Issues
- **Path resolution**: Ensure tsconfig.json paths are correctly configured
- **Type definitions**: Verify all @types packages are installed
- **Module system**: Confirm CommonJS vs ES modules configuration
- **Build output**: Check that dist/ directory is properly configured

## Next Steps
After completion, this project will be ready for:
- Adding route handlers
- Implementing middleware
- Creating type definitions
- Building API endpoints