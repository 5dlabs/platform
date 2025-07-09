# Autonomous Prompt: Create README Documentation

## Task Context
You are an AI assistant tasked with creating comprehensive README documentation for an Express.js TypeScript API. This documentation should serve as the primary guide for developers to understand, set up, and use the API.

## Objective
Create a professional, comprehensive README.md file that covers all aspects of the application including setup, API documentation, usage examples, and development guidelines.

## Required Actions

### 1. Create README.md File
Create a comprehensive `README.md` file in the project root with the following structure:

```markdown
# Express TypeScript API

![Node.js](https://img.shields.io/badge/Node.js-v18+-green.svg)
![TypeScript](https://img.shields.io/badge/TypeScript-v5+-blue.svg)
![Express](https://img.shields.io/badge/Express-v4+-lightgrey.svg)
![License](https://img.shields.io/badge/License-MIT-yellow.svg)

A modern, type-safe REST API built with Express.js and TypeScript, featuring user management, health checks, and comprehensive error handling.

## 🚀 Features

- **Type-Safe Development**: Full TypeScript support with strict type checking
- **RESTful API Design**: Clean, consistent REST endpoints
- **User Management**: Complete CRUD operations for users
- **Health Monitoring**: Built-in health check endpoints
- **Error Handling**: Comprehensive error handling with consistent response format
- **Request Validation**: Input validation and sanitization
- **Security**: Rate limiting, CORS, and security headers
- **Development Tools**: Hot reload, TypeScript compilation, and debugging support

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js**: v18.0.0 or higher
- **npm**: v8.0.0 or higher (or yarn equivalent)
- **TypeScript**: v5.0.0 or higher (installed globally or via npm)

## 🛠️ Installation

### 1. Clone the repository
```bash
git clone https://github.com/your-username/express-typescript-api.git
cd express-typescript-api
```

### 2. Install dependencies
```bash
npm install
```

### 3. Environment setup
Create a `.env` file in the root directory:
```env
NODE_ENV=development
PORT=3000
SERVICE_NAME=express-typescript-api
SERVICE_VERSION=1.0.0
```

### 4. Start the development server
```bash
npm run dev
```

### 5. Build for production
```bash
npm run build
npm start
```

The API will be available at `http://localhost:3000`

## 📚 API Documentation

### Base URL
```
http://localhost:3000/api
```

### Authentication
Currently, no authentication is required for any endpoints.

### Response Format
All responses follow a consistent format:

**Success Response:**
```json
{
  "data": "response data here",
  "timestamp": "2023-07-09T15:30:00.000Z"
}
```

**Error Response:**
```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "timestamp": "2023-07-09T15:30:00.000Z",
  "path": "/api/endpoint",
  "method": "GET",
  "requestId": "req_abc123"
}
```

## 🏥 Health Check Endpoints

### GET /api/health
Check the health status of the API.

**Response (200 OK):**
```json
{
  "status": "ok",
  "timestamp": "2023-07-09T15:30:00.000Z",
  "uptime": 123.456,
  "service": "express-typescript-api",
  "version": "1.0.0",
  "environment": "development"
}
```

**curl Example:**
```bash
curl -X GET http://localhost:3000/api/health
```

### GET /api/ping
Simple connectivity test.

**Response (200 OK):**
```json
{
  "message": "pong"
}
```

**curl Example:**
```bash
curl -X GET http://localhost:3000/api/ping
```

## 👥 User Management Endpoints

### GET /api/users
Retrieve all users.

**Response (200 OK):**
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "John Doe",
    "email": "john@example.com",
    "createdAt": "2023-07-09T15:30:00.000Z"
  },
  {
    "id": "987fcdeb-51a2-43d1-9b12-345678901234",
    "name": "Jane Smith",
    "email": "jane@example.com",
    "createdAt": "2023-07-09T15:31:00.000Z"
  }
]
```

**curl Example:**
```bash
curl -X GET http://localhost:3000/api/users
```

### GET /api/users/:id
Retrieve a specific user by ID.

**Parameters:**
- `id` (path): User ID (UUID)

**Response (200 OK):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "John Doe",
  "email": "john@example.com",
  "createdAt": "2023-07-09T15:30:00.000Z"
}
```

**Response (404 Not Found):**
```json
{
  "error": "User not found",
  "code": "NOT_FOUND",
  "timestamp": "2023-07-09T15:30:00.000Z",
  "path": "/api/users/nonexistent-id",
  "method": "GET",
  "requestId": "req_abc123"
}
```

**curl Example:**
```bash
curl -X GET http://localhost:3000/api/users/123e4567-e89b-12d3-a456-426614174000
```

### POST /api/users
Create a new user.

**Request Body:**
```json
{
  "name": "Alice Johnson",
  "email": "alice@example.com"
}
```

**Response (201 Created):**
```json
{
  "id": "456e7890-a12b-34c5-d678-901234567890",
  "name": "Alice Johnson",
  "email": "alice@example.com",
  "createdAt": "2023-07-09T15:32:00.000Z"
}
```

**Response (400 Bad Request):**
```json
{
  "error": "Validation failed",
  "code": "VALIDATION_ERROR",
  "timestamp": "2023-07-09T15:30:00.000Z",
  "path": "/api/users",
  "method": "POST",
  "requestId": "req_abc123",
  "details": {
    "name": "Name is required and must be non-empty",
    "email": "Email is required and must be valid"
  }
}
```

**Response (409 Conflict):**
```json
{
  "error": "Email already exists",
  "code": "CONFLICT",
  "timestamp": "2023-07-09T15:30:00.000Z",
  "path": "/api/users",
  "method": "POST",
  "requestId": "req_abc123"
}
```

**curl Example:**
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Johnson","email":"alice@example.com"}'
```

### PUT /api/users/:id
Update an existing user.

**Parameters:**
- `id` (path): User ID (UUID)

**Request Body:**
```json
{
  "name": "Updated Name",
  "email": "updated@example.com"
}
```

**Response (200 OK):**
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Updated Name",
  "email": "updated@example.com",
  "createdAt": "2023-07-09T15:30:00.000Z"
}
```

**curl Example:**
```bash
curl -X PUT http://localhost:3000/api/users/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Name"}'
```

### DELETE /api/users/:id
Delete a user.

**Parameters:**
- `id` (path): User ID (UUID)

**Response (204 No Content):**
No response body.

**curl Example:**
```bash
curl -X DELETE http://localhost:3000/api/users/123e4567-e89b-12d3-a456-426614174000
```

## 🔧 Development

### Available Scripts

- `npm run dev` - Start development server with hot reload
- `npm run build` - Build the application for production
- `npm start` - Start production server
- `npm run type-check` - Run TypeScript type checking
- `npm test` - Run tests (when implemented)

### Project Structure
```
src/
├── index.ts              # Application entry point
├── routes/               # API route handlers
│   ├── health.ts         # Health check routes
│   └── users.ts          # User management routes
├── middleware/           # Custom middleware
│   └── error.ts          # Error handling middleware
├── types/                # TypeScript type definitions
│   ├── user.ts           # User-related types
│   └── error.ts          # Error types
└── utils/                # Utility functions
    └── errors.ts         # Error utilities
```

### TypeScript Configuration
The project uses strict TypeScript configuration for maximum type safety:

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
    "forceConsistentCasingInFileNames": true
  }
}
```

### Adding New Endpoints

1. **Create route handler** in `src/routes/`
2. **Define types** in `src/types/`
3. **Add validation** using type guards
4. **Implement error handling** with custom error classes
5. **Update documentation** in this README

### Error Handling

The API uses a comprehensive error handling system:

**Error Types:**
- `ValidationError` (400) - Invalid input data
- `UnauthorizedError` (401) - Authentication required
- `ForbiddenError` (403) - Access denied
- `NotFoundError` (404) - Resource not found
- `ConflictError` (409) - Resource conflict
- `InternalServerError` (500) - Server error

**Custom Error Classes:**
```typescript
import { ValidationError, NotFoundError, ConflictError } from './types/error';

// Throw validation error
throw new ValidationError('Invalid data', { field: 'email' });

// Throw not found error
throw new NotFoundError('User not found');

// Throw conflict error
throw new ConflictError('Email already exists');
```

## 🧪 Testing

### Manual Testing
Use the provided curl examples to test all endpoints.

### Automated Testing
```bash
# Run type checking
npm run type-check

# Build the project
npm run build

# Test the health endpoint
curl -X GET http://localhost:3000/api/health
```

### Load Testing
```bash
# Install hey for load testing
go install github.com/rakyll/hey@latest

# Test with 100 concurrent requests
hey -n 1000 -c 100 http://localhost:3000/api/health
```

## 🚀 Deployment

### Environment Variables
```env
NODE_ENV=production
PORT=3000
SERVICE_NAME=express-typescript-api
SERVICE_VERSION=1.0.0
```

### Docker Deployment
```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY dist ./dist

EXPOSE 3000

CMD ["node", "dist/index.js"]
```

### Production Checklist
- [ ] Set `NODE_ENV=production`
- [ ] Configure proper logging
- [ ] Set up monitoring and alerting
- [ ] Configure reverse proxy (nginx)
- [ ] Set up SSL/TLS certificates
- [ ] Configure database (if applicable)
- [ ] Set up backup and recovery

## 📊 Monitoring

### Health Checks
The API provides health check endpoints suitable for:
- Load balancers
- Container orchestration (Kubernetes, Docker Swarm)
- Monitoring systems (Prometheus, Grafana)
- Uptime monitoring services

### Metrics
Key metrics to monitor:
- Response time
- Error rate
- Request volume
- Memory usage
- CPU usage

## 🔒 Security

### Security Features
- **Rate Limiting**: 100 requests per 15 minutes per IP
- **CORS**: Configurable cross-origin resource sharing
- **Helmet**: Security headers middleware
- **Input Validation**: Comprehensive input validation
- **Error Sanitization**: No sensitive data in error responses

### Security Headers
The API includes security headers:
- `X-Content-Type-Options`
- `X-Frame-Options`
- `X-XSS-Protection`
- `Strict-Transport-Security`
- `Content-Security-Policy`

## 🤝 Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Install dependencies: `npm install`
4. Start development server: `npm run dev`
5. Make your changes
6. Run type checking: `npm run type-check`
7. Build the project: `npm run build`
8. Submit a pull request

### Code Style
- Use TypeScript for all new code
- Follow the existing code style
- Add type definitions for all functions
- Include error handling for all operations
- Update documentation for new features

### Pull Request Process
1. Ensure TypeScript compilation passes
2. Add tests for new features
3. Update documentation
4. Ensure all existing tests pass
5. Request code review

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Express.js](https://expressjs.com/) - Web framework
- [TypeScript](https://www.typescriptlang.org/) - Type safety
- [Node.js](https://nodejs.org/) - Runtime environment

## 📞 Support

For support and questions:
- Create an issue on GitHub
- Check existing documentation
- Review the API examples

---

**Happy coding! 🚀**
```

### 2. Validation Steps
1. **Follow README instructions** on a fresh clone
2. **Test all curl examples** to ensure they work
3. **Verify setup instructions** are complete and accurate
4. **Check formatting** and ensure proper markdown syntax
5. **Validate links** and references

### 3. Documentation Maintenance
- Update README when API changes
- Keep examples current with latest code
- Maintain consistent formatting
- Regular review and updates
- Version documentation with releases

## Success Criteria
- [ ] Complete project documentation
- [ ] All API endpoints are documented with examples
- [ ] Setup instructions are clear and complete
- [ ] Error responses are documented
- [ ] Development guidelines are included
- [ ] Professional presentation and formatting
- [ ] All curl examples work correctly
- [ ] README follows markdown best practices

## Final Deliverables
- [ ] Comprehensive README.md file
- [ ] API documentation with request/response examples
- [ ] Setup and installation instructions
- [ ] Development guidelines and project structure
- [ ] Error handling documentation
- [ ] Security considerations
- [ ] Contributing guidelines
- [ ] Professional formatting and presentation