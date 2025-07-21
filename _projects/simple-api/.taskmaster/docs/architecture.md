# Simple Todo API - Architecture Document

## System Overview

The Simple Todo API is a lightweight REST API built with Node.js and Express, designed for managing todo items with persistent SQLite storage. The architecture follows a clean separation of concerns with controllers, models, routes, and middleware.

## Technology Stack

- **Runtime**: Node.js 18+
- **Framework**: Express.js 4.x
- **Database**: SQLite with better-sqlite3 driver
- **Testing**: Jest with supertest
- **Validation**: express-validator
- **Documentation**: OpenAPI/Swagger
- **Development**: nodemon, prettier

## Architecture Layers

### 1. Application Layer (`src/app.js`)
- Express application setup
- Middleware configuration
- Error handling middleware
- Database initialization

### 2. Routes Layer (`src/routes/`)
- API route definitions
- Request routing to controllers
- Input validation middleware
- Route-level error handling

### 3. Controller Layer (`src/controllers/`)
- Business logic implementation
- Request/response handling
- Data validation and sanitization
- Error response formatting

### 4. Model Layer (`src/models/`)
- Database interaction
- Data access objects (DAO)
- SQL query execution
- Data validation

### 5. Middleware (`src/middleware/`)
- Request validation
- Error handling
- Logging
- CORS configuration

## Project Structure

```
simple-api/
├── src/
│   ├── controllers/
│   │   └── todoController.js
│   ├── models/
│   │   └── todoModel.js
│   ├── routes/
│   │   ├── index.js
│   │   └── todos.js
│   ├── middleware/
│   │   ├── validation.js
│   │   └── errorHandler.js
│   └── app.js
├── tests/
│   ├── unit/
│   ├── integration/
│   └── fixtures/
├── data/
│   └── todos.db
├── docs/
│   ├── api.md
│   └── deployment.md
├── package.json
├── server.js
├── .env.example
└── README.md
```

## Data Model

### Todo Entity
```javascript
{
  id: integer (primary key, auto-increment)
  title: string (required, max 200 chars)
  description: string (optional, max 1000 chars)
  completed: boolean (default false)
  createdAt: datetime (ISO 8601)
  updatedAt: datetime (ISO 8601)
}
```

### Database Schema
```sql
CREATE TABLE todos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL CHECK(length(title) <= 200),
  description TEXT CHECK(length(description) <= 1000),
  completed BOOLEAN NOT NULL DEFAULT 0,
  createdAt DATETIME DEFAULT CURRENT_TIMESTAMP,
  updatedAt DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_todos_timestamp
  AFTER UPDATE ON todos
  BEGIN
    UPDATE todos SET updatedAt = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;
```

## API Endpoints

### Core CRUD Operations
- `GET /api/todos` - List todos with optional filtering
- `POST /api/todos` - Create new todo
- `GET /api/todos/:id` - Get specific todo
- `PUT /api/todos/:id` - Update todo
- `DELETE /api/todos/:id` - Delete todo

### System Endpoints
- `GET /api/health` - Health check
- `GET /api/docs` - API documentation

## Error Handling Strategy

### Error Response Format
```javascript
{
  error: {
    message: "Human readable error message",
    code: "ERROR_CODE",
    details: {...} // Optional additional details
  }
}
```

### HTTP Status Codes
- `200` - Successful GET/PUT
- `201` - Successful POST
- `204` - Successful DELETE
- `400` - Bad Request (validation errors)
- `404` - Resource not found
- `500` - Internal server error

## Validation Rules

### Input Validation
- Title: Required, 1-200 characters
- Description: Optional, max 1000 characters
- Completed: Boolean (true/false)
- ID parameters: Positive integers only

### Data Sanitization
- Trim whitespace from strings
- HTML encode special characters
- Normalize boolean inputs

## Testing Strategy

### Unit Tests
- Controller logic testing
- Model data access testing
- Validation middleware testing
- Error handling testing

### Integration Tests
- Full API endpoint testing
- Database transaction testing
- Error scenario testing
- Response format validation

### Test Coverage Goals
- Minimum 90% code coverage
- All error paths tested
- All validation rules tested
- Database constraints tested

## Development Workflow

### Environment Setup
1. Install Node.js 18+
2. Clone repository
3. Install dependencies: `npm install`
4. Copy `.env.example` to `.env`
5. Initialize database: `npm run db:init`

### Development Commands
- `npm run dev` - Start development server with auto-reload
- `npm test` - Run test suite
- `npm run test:watch` - Run tests in watch mode
- `npm run test:coverage` - Generate coverage report
- `npm run lint` - Run code linting
- `npm run format` - Format code with prettier

## Deployment Considerations

### Production Configuration
- Environment variables for configuration
- Database file location configuration
- Logging configuration
- Security headers middleware

### Performance Optimizations
- SQLite WAL mode for better concurrency
- Request compression middleware
- Response caching for read operations
- Connection pooling (if needed for scale)

## Security Measures

### Input Security
- SQL injection prevention via parameterized queries
- XSS prevention via input sanitization
- Request rate limiting
- Input size limits

### Data Security
- Database file permissions
- Secure error messages (no sensitive data exposure)
- CORS configuration
- Security headers (helmet.js)

## Monitoring and Logging

### Application Logging
- Request/response logging
- Error logging with stack traces
- Performance timing logs
- Database operation logs

### Health Monitoring
- Database connectivity checks
- Application uptime tracking
- Response time monitoring
- Error rate tracking

## Scalability Considerations

### Current Limitations
- SQLite is suitable for low-to-medium traffic
- Single-threaded JavaScript execution
- File-based database storage

### Future Enhancements
- PostgreSQL migration for higher scale
- Redis caching layer
- Horizontal scaling with load balancer
- Microservice decomposition if needed

## Integration Points

### External Dependencies
- Node.js runtime
- SQLite database engine
- File system access
- HTTP request/response handling

### API Consumers
- Frontend web applications
- Mobile applications
- Other microservices
- Testing frameworks