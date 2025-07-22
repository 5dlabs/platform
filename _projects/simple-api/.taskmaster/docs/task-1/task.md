# Task 1: Initialize Project and Environment Configuration

## Overview
Set up the project repository, initialize npm, configure environment variables, and establish the base project structure for the Simple Express API.

## Task Details
- **Priority**: High
- **Dependencies**: None (first task)
- **Status**: Pending

## Implementation Guide

### 1. Initialize NPM Project
```bash
npm init -y
```
This creates a `package.json` file with default values.

### 2. Install Core Dependencies
```bash
npm install express@5 dotenv@16
npm install --save-dev nodemon@3
```

### 3. Create Project Structure
```
simple-api/
├── src/
│   ├── index.js          # Main server file
│   ├── routes/           # API route definitions
│   ├── controllers/      # Business logic handlers
│   ├── middleware/       # Express middleware
│   └── utils/            # Utility functions
├── .env                  # Environment variables
├── .gitignore           # Git ignore file
├── package.json         # Project dependencies
└── README.md            # Project documentation
```

### 4. Configure Environment Variables
Create `.env` file:
```env
PORT=3000
NODE_ENV=development
```

### 5. Create .gitignore
```
node_modules/
.env
.DS_Store
*.log
```

### 6. Initialize README.md
Create basic README with:
- Project title and description
- Prerequisites (Node.js 18+)
- Installation instructions
- Available scripts
- API endpoints (to be updated)

## Acceptance Criteria
- [ ] NPM initialized with package.json
- [ ] Core dependencies installed (express@5, dotenv@16, nodemon@3)
- [ ] Project directory structure created
- [ ] .env file configured with PORT variable
- [ ] .gitignore file created
- [ ] Basic README.md with setup instructions
- [ ] Node.js version 18+ requirement documented

## Test Strategy
Verify that running `npm start` and `npm run dev` starts the server and loads environment variables correctly:
1. Check package.json exists with correct dependencies
2. Verify directory structure is created
3. Confirm .env file is properly configured
4. Test that environment variables load correctly