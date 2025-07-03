# Product Requirements Document: Todo API Service

## Overview
Build a RESTful API service for managing todo items with persistent storage, comprehensive testing, and production-ready features.

## Objectives
- Create a simple but complete example of a REST API
- Demonstrate best practices for Node.js development
- Provide a foundation for learning API development

## Technical Requirements

### Technology Stack
- Runtime: Node.js 20+
- Framework: Express.js
- Database: SQLite (via better-sqlite3)
- Testing: Jest
- Documentation: OpenAPI/Swagger

### Core Features

#### 1. Todo Management
- Create new todo items
- List all todos with filtering options
- Get individual todo details
- Update todo items (partial updates supported)
- Delete todo items
- Batch operations for marking multiple as complete

#### 2. Data Model
```
Todo {
  id: integer (auto-increment)
  title: string (required, max 200 chars)
  description: string (optional, max 1000 chars)
  completed: boolean (default: false)
  priority: enum ['low', 'medium', 'high'] (default: 'medium')
  due_date: datetime (optional)
  tags: array of strings (optional)
  created_at: datetime
  updated_at: datetime
}
```

#### 3. API Endpoints
- `GET /api/todos` - List todos with pagination, filtering, and sorting
- `GET /api/todos/:id` - Get single todo
- `POST /api/todos` - Create todo
- `PUT /api/todos/:id` - Update todo (full update)
- `PATCH /api/todos/:id` - Update todo (partial update)
- `DELETE /api/todos/:id` - Delete todo
- `POST /api/todos/batch/complete` - Mark multiple as complete
- `GET /api/todos/stats` - Get statistics (count by status, priority)

#### 4. Features
- Input validation with clear error messages
- Pagination with limit/offset
- Filtering by status, priority, tags
- Sorting by created_at, due_date, priority
- Search by title/description
- Rate limiting (100 requests per minute)
- Request logging
- CORS support
- Health check endpoint
- API versioning

#### 5. Non-Functional Requirements
- Response time < 200ms for single item operations
- Support 100+ concurrent connections
- 95%+ test coverage
- Comprehensive error handling
- Environment-based configuration
- Docker support

## Implementation Phases

### Phase 1: Project Setup and Basic API
1. Initialize Node.js project with proper structure
2. Set up Express server with middleware
3. Implement basic CRUD endpoints
4. Add input validation

### Phase 2: Database and Persistence
1. Set up SQLite database
2. Create schema and migrations
3. Implement repository pattern
4. Add database error handling

### Phase 3: Advanced Features
1. Add pagination and filtering
2. Implement sorting and search
3. Add batch operations
4. Create statistics endpoint

### Phase 4: Production Readiness
1. Add comprehensive test suite
2. Implement rate limiting
3. Add logging and monitoring
4. Create Docker configuration
5. Write API documentation

## Success Criteria
- All endpoints return correct status codes
- Input validation prevents invalid data
- Tests achieve >90% coverage
- API handles errors gracefully
- Performance meets requirements
- Documentation is complete and accurate

## Security Considerations
- SQL injection prevention via parameterized queries
- Input sanitization
- Rate limiting to prevent abuse
- No sensitive data in logs
- Proper error messages (no stack traces in production)

## Future Enhancements (Out of Scope)
- User authentication
- Todo sharing/collaboration
- File attachments
- Email notifications
- Webhooks
- GraphQL endpoint