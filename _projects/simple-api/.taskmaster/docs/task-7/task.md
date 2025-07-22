# Task 7: Configure package.json Scripts and Documentation

## Overview
Set up npm scripts for starting and developing the server, and provide comprehensive API documentation in the README.

## Task Details
- **Priority**: Medium
- **Dependencies**: Task 1 (Initialize Project and Environment Configuration)
- **Status**: Pending

## Implementation Guide

### 1. Update package.json Scripts
```json
{
  "name": "simple-express-api",
  "version": "1.0.0",
  "description": "A lightweight REST API built with Express.js",
  "main": "src/index.js",
  "type": "module",
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js",
    "test": "echo \"Error: no test specified\" && exit 1"
  },
  "keywords": ["express", "api", "rest", "nodejs"],
  "author": "",
  "license": "MIT",
  "engines": {
    "node": ">=18.0.0"
  }
}
```

### 2. Create Comprehensive README.md
```markdown
# Simple Express API

A lightweight REST API built with Express.js providing user management capabilities with a focus on simplicity and maintainability.

## Prerequisites

- Node.js 18+ LTS
- npm or yarn package manager

## Installation

1. Clone the repository:
   \`\`\`bash
   git clone <repository-url>
   cd simple-express-api
   \`\`\`

2. Install dependencies:
   \`\`\`bash
   npm install
   \`\`\`

3. Set up environment variables:
   \`\`\`bash
   cp .env.example .env
   \`\`\`

4. Configure your `.env` file:
   \`\`\`env
   PORT=3000
   NODE_ENV=development
   \`\`\`

## Available Scripts

- `npm start` - Start the production server
- `npm run dev` - Start the development server with hot reload

## API Endpoints

### General Endpoints

#### GET /
Returns API information and version.

**Response:**
\`\`\`json
{
  "message": "Welcome to Simple Express API",
  "version": "1.0.0",
  "timestamp": "2025-01-22T10:00:00.000Z"
}
\`\`\`

#### GET /health
Health check endpoint for monitoring.

**Response:**
\`\`\`json
{
  "status": "ok",
  "uptime": 123.456,
  "timestamp": "2025-01-22T10:00:00.000Z"
}
\`\`\`

### User Management

#### GET /api/users
Retrieve all users.

**Response:** `200 OK`
\`\`\`json
[
  {
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "createdAt": "2025-01-01T00:00:00.000Z"
  }
]
\`\`\`

#### POST /api/users
Create a new user.

**Request Body:**
\`\`\`json
{
  "name": "Jane Smith",
  "email": "jane@example.com"
}
\`\`\`

**Response:** `201 Created`
\`\`\`json
{
  "id": 3,
  "name": "Jane Smith",
  "email": "jane@example.com",
  "createdAt": "2025-01-22T10:00:00.000Z"
}
\`\`\`

**Validation Rules:**
- `name`: Required, 2-100 characters
- `email`: Required, valid email format

## Error Responses

All errors follow a consistent format:

\`\`\`json
{
  "error": "ErrorType",
  "message": "Descriptive error message"
}
\`\`\`

### Common Error Codes
- `400` - Bad Request (validation errors)
- `404` - Not Found
- `500` - Internal Server Error

## Project Structure

\`\`\`
simple-api/
├── src/
│   ├── index.js          # Main server file
│   ├── routes/           # API route definitions
│   ├── controllers/      # Business logic
│   ├── middleware/       # Express middleware
│   ├── utils/            # Utility functions
│   └── data/             # Mock data store
├── .env                  # Environment variables
├── .env.example          # Environment template
├── .gitignore           # Git ignore rules
├── package.json         # Dependencies and scripts
└── README.md            # This file
\`\`\`

## Development

1. Run in development mode:
   \`\`\`bash
   npm run dev
   \`\`\`

2. The server will restart automatically on file changes.

3. Default port is 3000, configure in `.env` file.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| PORT | Server port | 3000 |
| NODE_ENV | Environment mode | development |

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT
```

### 3. Create .env.example
```env
# Server Configuration
PORT=3000
NODE_ENV=development

# Add other configuration as needed
```

## Acceptance Criteria
- [ ] NPM scripts configured correctly
- [ ] README includes all endpoints
- [ ] Installation instructions clear
- [ ] API documentation complete
- [ ] Error responses documented
- [ ] Environment variables listed
- [ ] Project structure documented

## Test Strategy
1. Verify all npm scripts work
2. Follow README instructions on fresh clone
3. Check documentation accuracy
4. Ensure examples are correct
5. Test that .env.example has all needed vars