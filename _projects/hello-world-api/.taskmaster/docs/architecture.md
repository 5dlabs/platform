# Hello World API Architecture

## Overview
A minimal Express.js API that responds with "Hello, World!" and basic health checks.

## Technology Stack
- Node.js 20+
- Express.js 4.x
- No database (in-memory only)
- No authentication

## API Endpoints
- `GET /` - Returns "Hello, World!"
- `GET /health` - Returns service health status

## Project Structure
```
hello-world-api/
├── src/
│   └── index.js      # Main server file
├── package.json      # Dependencies
└── README.md         # Documentation
```

## Deployment
- Single Node.js process
- Port 3000
- Environment: Development only