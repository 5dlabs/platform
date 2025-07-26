# Autonomous Agent Prompt for Task 13: Create README Documentation

## Task Context
You need to create comprehensive README documentation for the Hello World API project. This documentation will help users understand how to install, configure, and use the API.

## Your Mission
Create a professional README.md file that documents the project overview, installation instructions, usage guidelines, and complete API endpoint documentation.

## Step-by-Step Instructions

### 1. Navigate to Project Root
```bash
cd hello-world-api
```

### 2. Create README.md
Create a comprehensive README.md file with the following content:

```markdown
# Hello World API

A simple Express.js API that demonstrates basic REST endpoint implementation with health checking and error handling.

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Project Structure](#project-structure)
- [Error Handling](#error-handling)
- [Contributing](#contributing)
- [License](#license)

## Overview

This is a minimal Express.js API that serves as a starting point for building RESTful web services. It implements two basic endpoints and includes proper error handling, request logging, and follows Express.js best practices.

## Features

- ✅ Simple REST API with Express.js
- ✅ Health check endpoint for monitoring
- ✅ Request logging middleware
- ✅ Error handling for 404 and 500 errors
- ✅ Environment-based configuration
- ✅ Clean project structure
- ✅ Ready for extension

## Prerequisites

Before you begin, ensure you have the following installed:
- [Node.js](https://nodejs.org/) (v14.0.0 or higher)
- [npm](https://www.npmjs.com/) (v6.0.0 or higher)

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

3. Create environment variables (optional):
   ```bash
   # Create a .env file for environment-specific settings
   echo "PORT=3000" > .env
   ```

## Usage

### Starting the Server

1. For production:
   ```bash
   npm start
   ```

2. For development (with auto-reload):
   ```bash
   npm run dev
   ```

The server will start on port 3000 by default. You can change this by setting the `PORT` environment variable:

```bash
PORT=8080 npm start
```

### Making Requests

Once the server is running, you can access it at `http://localhost:3000`.

## API Documentation

### Base URL
```
http://localhost:3000
```

### Endpoints

#### 1. Root Endpoint

Returns a welcome message.

- **URL:** `/`
- **Method:** `GET`
- **Success Response:**
  - **Code:** 200
  - **Content:**
    ```json
    {
      "message": "Hello, World!"
    }
    ```

**Example Request:**
```bash
curl http://localhost:3000/
```

**Example Response:**
```json
{
  "message": "Hello, World!"
}
```

#### 2. Health Check Endpoint

Returns the health status of the API.

- **URL:** `/health`
- **Method:** `GET`
- **Success Response:**
  - **Code:** 200
  - **Content:**
    ```json
    {
      "status": "healthy",
      "timestamp": "2024-01-15T10:30:00.000Z"
    }
    ```

**Example Request:**
```bash
curl http://localhost:3000/health
```

**Example Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00.000Z"
}
```

### Error Responses

#### 404 Not Found
When accessing an undefined endpoint:

```json
{
  "error": "Not Found"
}
```

#### 500 Internal Server Error
When a server error occurs:

```json
{
  "error": "Internal Server Error"
}
```

## Development

### Available Scripts

- `npm start` - Start the production server
- `npm run dev` - Start the development server with hot reload
- `npm test` - Run tests (when implemented)

### Project Structure

```
hello-world-api/
├── src/
│   └── index.js        # Main application file
├── package.json        # Project metadata and dependencies
├── package-lock.json   # Locked dependency versions
├── .gitignore         # Git ignore file
└── README.md          # Project documentation
```

### Environment Variables

The application supports the following environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Port number for the server | `3000` |
| `NODE_ENV` | Environment mode | `development` |

## Error Handling

The API includes comprehensive error handling:

- **404 Handler:** Catches requests to undefined routes
- **500 Handler:** Catches and logs server errors
- **Request Logging:** All requests are logged with timestamp, method, and URL

## Testing

### Manual Testing

Test the API endpoints using curl:

```bash
# Test root endpoint
curl http://localhost:3000/

# Test health endpoint
curl http://localhost:3000/health

# Test 404 handling
curl http://localhost:3000/undefined

# Test with verbose output
curl -v http://localhost:3000/
```

### Using Postman

1. Import the endpoints into Postman
2. Create a new collection called "Hello World API"
3. Add requests for each endpoint
4. Test different scenarios

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```
   Error: EADDRINUSE: address already in use :::3000
   ```
   **Solution:** Change the port using the PORT environment variable or stop the process using port 3000.

2. **Module Not Found**
   ```
   Error: Cannot find module 'express'
   ```
   **Solution:** Run `npm install` to install dependencies.

3. **Permission Denied**
   ```
   Error: EACCES: permission denied
   ```
   **Solution:** Check file permissions or use a different port (ports below 1024 require admin privileges).

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Express.js](https://expressjs.com/)
- Inspired by REST API best practices
- Created as a learning project

---

**Note:** This is a demonstration project intended for learning purposes. For production use, consider adding authentication, rate limiting, and more comprehensive error handling.
```

### 3. Verify README Creation
```bash
# Check that file exists
ls -la README.md

# Preview the file
head -n 50 README.md
```

## Validation Steps

### 1. Check Markdown Formatting
```bash
# Install markdown linter if available
npm install -g markdownlint-cli
markdownlint README.md
```

### 2. Verify All Sections Present
```bash
# Check for main sections
grep -E "^##" README.md
```

### 3. Test Code Examples
Copy and run each code example from the README to ensure they work correctly.

### 4. Check Links
Ensure all links in the README are valid and point to correct resources.

## Expected Result
- Professional README.md file in project root
- All sections properly formatted
- Working code examples
- Complete API documentation
- Clear installation and usage instructions

## Important Notes
- Use proper Markdown syntax
- Include actual working examples
- Keep descriptions concise but complete
- Test all commands before documenting
- Consider the target audience (developers)
- Include troubleshooting for common issues