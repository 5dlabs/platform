# Autonomous Agent Prompt: Initialize Node.js Project

## Context
You are an expert Node.js developer tasked with initializing a new Node.js project for a Hello World API. This is the first task in building a simple Express.js API that will serve "Hello, World!" responses.

## Objective
Create a properly structured Node.js project with all necessary configuration files and directory structure to support the development of a basic Express.js API.

## Task Requirements

### 1. Create Project Structure
- Create a new directory named `hello-world-api`
- Inside the project directory, create a `src` folder for source code
- Create an empty `src/index.js` file as a placeholder for the main application

### 2. Initialize Node.js Project
- Navigate to the project directory
- Run `npm init -y` to create a basic package.json file
- The package.json should be created with default values

### 3. Configure Package.json
Update the generated package.json with the following:
- **name**: "hello-world-api"
- **version**: "1.0.0"
- **description**: "A simple Hello World API built with Node.js"
- **main**: "src/index.js"
- **private**: true
- **scripts**: 
  - "start": "node src/index.js"
- **keywords**: ["api", "express", "hello-world"]
- **author**: (keep default or update as needed)
- **license**: "ISC"

### 4. Create Version Control Configuration
Create a `.gitignore` file in the project root with the following content:
```
# Dependencies
node_modules/

# Environment variables
.env
.env.local
.env.*.local

# Logs
logs/
*.log
npm-debug.log*

# OS files
.DS_Store
Thumbs.db

# IDE files
.vscode/
.idea/
*.swp
*.swo

# Temporary files
*.tmp
*.temp
```

## Step-by-Step Execution

1. **Create the project directory structure**:
   ```bash
   mkdir -p hello-world-api/src
   touch hello-world-api/src/index.js
   ```

2. **Navigate to project directory and initialize npm**:
   ```bash
   cd hello-world-api
   npm init -y
   ```

3. **Update package.json** with the required configuration

4. **Create .gitignore file** with the specified content

## Validation Criteria

### Success Indicators
- [ ] Directory `hello-world-api` exists with `src` subdirectory
- [ ] File `src/index.js` exists (can be empty)
- [ ] Valid `package.json` file exists with correct metadata
- [ ] The `npm start` script is defined in package.json
- [ ] `.gitignore` file exists with appropriate exclusions

### Quality Checks
- Ensure package.json has valid JSON syntax
- Verify all required fields are present in package.json
- Confirm the project structure follows Node.js conventions

## Error Handling

### Common Issues to Handle
1. **Directory already exists**: Check if directory exists before creating
2. **npm not installed**: Provide clear error message about Node.js/npm requirement
3. **Invalid JSON in package.json**: Validate JSON syntax before saving
4. **File permissions**: Handle permission errors gracefully

## Expected Output

After successful completion, the project structure should be:
```
hello-world-api/
├── src/
│   └── index.js      # Empty file (placeholder)
├── package.json      # Node.js configuration
└── .gitignore        # Version control exclusions
```

The package.json should contain proper project metadata and npm scripts configuration.

## Important Notes

- This is a foundational task - ensure everything is set up correctly
- The project should follow Node.js best practices and conventions
- All files should use appropriate formatting and structure
- The setup should be ready for the next task: installing Express.js

## Tools Required
- File system access to create directories and files
- Command execution capability for npm commands
- Text editing capability to modify JSON files

Proceed with implementing this task, ensuring all requirements are met and the project is properly initialized for subsequent development tasks.