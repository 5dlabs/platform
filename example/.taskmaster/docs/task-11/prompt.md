# Autonomous Prompt: Initialize Express TypeScript Project

## Task Context
You are an AI assistant tasked with setting up a new Express.js project with TypeScript configuration. This is a foundational task that requires creating the basic project structure and configuration files.

## Objective
Initialize a complete Express.js project with TypeScript support, including proper configuration, dependencies, and project structure.

## Required Actions

### 1. Project Initialization
```bash
# Initialize npm project
npm init -y

# Update package.json with proper configuration
```

### 2. Dependency Installation
```bash
# Install Express.js
npm install express

# Install TypeScript development dependencies
npm install --save-dev typescript @types/express @types/node ts-node nodemon
```

### 3. TypeScript Configuration
Create `tsconfig.json` with the following configuration:
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

### 4. Project Structure Setup
```bash
# Create source directory structure
mkdir -p src/routes src/middleware src/types

# Create main entry point
touch src/index.ts
```

### 5. Package.json Scripts
Update package.json to include these scripts:
```json
{
  "scripts": {
    "start": "node dist/index.js",
    "dev": "nodemon src/index.ts",
    "build": "tsc"
  }
}
```

### 6. Basic Express Server
Create `src/index.ts` with basic Express server setup:
```typescript
import express from 'express';

const app = express();
const port = process.env.PORT || 3000;

app.use(express.json());

app.get('/', (req, res) => {
  res.json({ message: 'Express TypeScript server is running!' });
});

app.listen(port, () => {
  console.log(`Server is running on port ${port}`);
});
```

## Validation Steps
1. Run `npm run build` to verify TypeScript compilation
2. Run `npm run dev` to start development server
3. Verify server responds at http://localhost:3000
4. Check that all dependencies are properly installed
5. Confirm project structure is created correctly

## Success Criteria
- Project builds without TypeScript errors
- Development server starts successfully
- Basic Express server is functional
- All required directories and files are created
- Package.json has proper configuration and scripts

## Error Handling
- If TypeScript compilation fails, check tsconfig.json configuration
- If dependencies are missing, verify package.json and run npm install
- If server fails to start, check port availability and Express configuration
- If nodemon doesn't work, ensure ts-node is properly installed

## Tools Available
- npm/yarn for package management
- TypeScript compiler (tsc)
- File system operations for directory creation
- Text editor for configuration files

## Final Deliverables
- Configured package.json with all dependencies
- TypeScript configuration (tsconfig.json)
- Project directory structure (src/, routes/, middleware/, types/)
- Basic Express server (src/index.ts)
- Working build and development scripts