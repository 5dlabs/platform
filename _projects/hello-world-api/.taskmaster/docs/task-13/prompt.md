# Autonomous Agent Prompt: Create README Documentation

## Context
You are creating comprehensive documentation for the Hello World API project. The API has been implemented with two endpoints (root and health check) and includes error handling. Now you need to create a professional README.md file that will serve as the primary documentation for users and developers.

## Objective
Create a complete README.md file in the project root directory with clear documentation covering project overview, installation instructions, usage guidelines, API endpoint documentation, and helpful additional sections.

## Task Requirements

### 1. Create README.md Structure
The README should include these sections:
- Project title and description
- Table of contents
- Overview
- Features
- Prerequisites
- Installation
- Usage
- API Documentation
- Development
- Error Handling
- Troubleshooting
- License

### 2. Content Requirements
Each section should contain:
- **Overview**: What the project is and its purpose
- **Installation**: Step-by-step setup instructions
- **Usage**: How to start and configure the server
- **API Docs**: Detailed endpoint documentation with examples
- **Development**: Development scripts and workflow
- **Troubleshooting**: Common issues and solutions

### 3. Documentation Standards
- Use proper markdown formatting
- Include code examples with syntax highlighting
- Provide curl examples for API testing
- Show example responses
- Be clear and concise

## Complete README Template

```markdown
# Hello World API

A simple Express.js API that provides basic REST endpoints with health monitoring.

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Error Handling](#error-handling)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Overview

This is a minimal Express.js API that demonstrates basic REST endpoint implementation with proper error handling and health monitoring. It serves as a starting point for building more complex APIs or as a simple microservice.

## Features

- ✅ Simple REST API with JSON responses
- ✅ Health check endpoint for monitoring
- ✅ Request logging middleware
- ✅ Comprehensive error handling
- ✅ Environment-based configuration
- ✅ Development mode with auto-reload

## Prerequisites

Before you begin, ensure you have the following installed:
- Node.js (v20.0.0 or higher)
- npm (v9.0.0 or higher)

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

## Usage

### Starting the Server

1. **Production mode:**
   ```bash
   npm start
   ```
   The server will start on port 3000 by default.

2. **Development mode (with auto-reload):**
   ```bash
   npm run dev
   ```

3. **Custom port:**
   ```bash
   PORT=8080 npm start
   ```

### Configuration

The application can be configured using environment variables:

- `PORT` - Server port (default: 3000)
- `NODE_ENV` - Environment mode (default: development)

## API Documentation

### Base URL
```
http://localhost:3000
```

### Endpoints

#### GET /

Returns a welcome message.

**Response:**
- Status: `200 OK`
- Content-Type: `application/json`

```json
{
  "message": "Hello, World!"
}
```

**Example Request:**
```bash
curl http://localhost:3000/
```

---

#### GET /health

Health check endpoint for monitoring the API status.

**Response:**
- Status: `200 OK`
- Content-Type: `application/json`

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:45.123Z"
}
```

**Example Request:**
```bash
curl http://localhost:3000/health
```

### Error Responses

The API returns consistent error responses:

#### 404 Not Found
```json
{
  "error": "Not Found"
}
```

#### 500 Internal Server Error
```json
{
  "error": "Internal Server Error"
}
```

## Development

### Available Scripts

- `npm start` - Start the production server
- `npm run dev` - Start the development server with nodemon
- `npm test` - Run tests (placeholder)

### Project Structure

```
hello-world-api/
├── src/
│   ├── index.js        # Main application file
│   └── config/
│       └── express.js  # Express configuration
├── package.json        # Project dependencies
├── package-lock.json   # Locked dependencies
├── .gitignore         # Git ignore rules
└── README.md          # This file
```

### Request Logging

All requests are logged to the console with the following format:
```
2024-01-15T10:30:45.123Z - GET /
```

## Error Handling

The API implements comprehensive error handling:

1. **Server Errors (500)**: Caught and logged with stack traces
2. **Not Found (404)**: Returned for undefined routes
3. **Startup Errors**: Port conflicts and permission issues handled gracefully

## Troubleshooting

### Common Issues

#### Port Already in Use
**Error:** `Port 3000 is already in use`

**Solution:**
1. Find the process using the port:
   ```bash
   lsof -i :3000  # On macOS/Linux
   netstat -ano | findstr :3000  # On Windows
   ```
2. Kill the process or use a different port:
   ```bash
   PORT=3001 npm start
   ```

#### Module Not Found
**Error:** `Cannot find module 'express'`

**Solution:**
Reinstall dependencies:
```bash
npm install
```

#### Permission Denied (EACCES)
**Error:** `Port 80 requires elevated privileges`

**Solution:**
Use a port above 1024 or run with appropriate permissions (not recommended).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues, questions, or contributions, please open an issue in the GitHub repository.
```

## Implementation Steps

1. **Create the README.md file** in the project root
2. **Copy the template** and adjust as needed
3. **Verify accuracy** of all examples and commands
4. **Test all documented commands** to ensure they work
5. **Review for completeness** and clarity

## Validation Checklist

- [ ] All sections are present and complete
- [ ] Code examples use proper syntax highlighting
- [ ] API examples show actual responses
- [ ] Installation steps are accurate
- [ ] Troubleshooting covers common issues
- [ ] Markdown formatting is correct
- [ ] Table of contents links work
- [ ] No placeholder text remains

## Important Notes

- Ensure all commands and examples match the actual implementation
- Use consistent formatting throughout
- Keep explanations clear and concise
- Include both curl and browser usage examples where applicable
- Make sure the README is helpful for both new users and developers

## Tools Required
- File system access to create README.md
- Text editing capability
- Markdown formatting knowledge

Proceed with creating the README.md file using the provided template, ensuring all information is accurate and the documentation provides value to users and developers working with the Hello World API.