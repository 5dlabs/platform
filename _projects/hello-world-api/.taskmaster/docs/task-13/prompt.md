# Autonomous AI Agent Prompt: Create README Documentation

## Task Overview
You need to create a comprehensive README.md file for the Hello World API project. The README should include project overview, installation instructions, usage guidelines, API documentation, and troubleshooting information.

## Detailed Instructions

### Step 1: Create README.md File
Create a new file at the project root:
```bash
touch hello-world-api/README.md
```

### Step 2: Add Project Header and Overview
Start with:
```markdown
# Hello World API

A simple Express.js API that demonstrates basic REST endpoints with health checking.

## Table of Contents
[Generate a table of contents with links to all sections]

## Overview

This is a minimal Express.js API that provides a "Hello, World!" endpoint and health check functionality. It serves as a starting point for building more complex APIs with Node.js and Express.
```

### Step 3: Document Features
```markdown
## Features

- ✅ Simple REST API with Express.js
- ✅ Hello World endpoint
- ✅ Health check endpoint with timestamp
- ✅ Request logging middleware
- ✅ Error handling for 404 and 500 errors
- ✅ Environment variable support
- ✅ Development mode with nodemon
```

### Step 4: Add Prerequisites and Installation
```markdown
## Prerequisites

- Node.js 20.x or higher
- npm 10.x or higher
- Git (for cloning the repository)

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd hello-world-api
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Verify installation:
   ```bash
   npm list
   ```
```

### Step 5: Document Usage
```markdown
## Usage

### Starting the Server

**Production mode:**
```bash
npm start
```

**Development mode (with auto-reload):**
```bash
npm run dev
```

**Custom port:**
```bash
PORT=8080 npm start
```

The server will start on port 3000 by default.
```

### Step 6: Create API Documentation
```markdown
## API Documentation

### Base URL
```
http://localhost:3000
```

### Endpoints

#### GET /

Returns a welcome message.

**Request:**
```bash
curl http://localhost:3000/
```

**Response:**
```json
{
  "message": "Hello, World!"
}
```

**Status Code:** 200 OK

---

#### GET /health

Health check endpoint to verify the API is running.

**Request:**
```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-20T15:30:45.123Z"
}
```

**Status Code:** 200 OK
```

### Step 7: Add Error Response Documentation
```markdown
### Error Responses

#### 404 Not Found

Returned when accessing an undefined endpoint.

**Example:**
```bash
curl http://localhost:3000/unknown
```

**Response:**
```json
{
  "error": "Not Found"
}
```

#### 500 Internal Server Error

Returned when a server error occurs.

**Response:**
```json
{
  "error": "Internal Server Error"
}
```
```

### Step 8: Add Development Section
```markdown
## Development

### Project Structure
```
hello-world-api/
├── src/
│   └── index.js        # Main server file
├── package.json
├── package-lock.json
├── .gitignore
└── README.md
```

### Available Scripts

- `npm start` - Start the production server
- `npm run dev` - Start development server with nodemon

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| PORT | Server port | 3000 |
| NODE_ENV | Environment mode | development |
```

### Step 9: Add Troubleshooting Section
```markdown
## Troubleshooting

### Common Issues

**Port already in use:**
```
Error: listen EADDRINUSE: address already in use :::3000
```
**Solution:** Use a different port:
```bash
PORT=3001 npm start
```

**Module not found:**
```
Error: Cannot find module 'express'
```
**Solution:** Install dependencies:
```bash
npm install
```
```

### Step 10: Add License and Contact
```markdown
## License

This project is licensed under the MIT License.

## Contact

For questions or support, please open an issue in the GitHub repository.
```

## Expected Outcomes

### Complete README.md
- Professional documentation
- All sections properly formatted
- Working code examples
- Clear installation instructions
- Comprehensive API documentation

### Documentation Standards
- Consistent markdown formatting
- Proper code block syntax highlighting
- Clear section headers
- Logical information flow

## Validation Steps

1. **File Creation**
   ```bash
   test -f hello-world-api/README.md && echo "README exists" || echo "README missing"
   ```

2. **Content Verification**
   - Check all sections are present
   - Verify code examples use correct syntax
   - Ensure API responses match implementation

3. **Markdown Validation**
   - Headers properly formatted
   - Code blocks have language specified
   - Lists properly indented

4. **Example Testing**
   - All curl commands should work
   - Response formats should match actual API

## Important Notes

### Content Guidelines
- Keep descriptions concise but complete
- Use active voice
- Include practical examples
- Explain "why" not just "how"

### Formatting Requirements
- Use ```bash for shell commands
- Use ```json for JSON responses
- Use ```javascript for code examples
- Include blank lines between sections

### Accuracy Requirements
- Port numbers must match implementation
- Endpoint paths must be exact
- Response formats must match actual responses
- Error messages must be accurate

## Quality Checklist

- [ ] Project overview is clear and concise
- [ ] Installation steps are complete
- [ ] All dependencies are listed
- [ ] Usage instructions include all options
- [ ] API documentation covers all endpoints
- [ ] Response examples are accurate
- [ ] Error responses are documented
- [ ] Troubleshooting covers common issues
- [ ] Project structure is accurate
- [ ] Environment variables are documented
- [ ] All code examples are tested
- [ ] Markdown formatting is consistent
- [ ] Table of contents links work
- [ ] No spelling or grammar errors