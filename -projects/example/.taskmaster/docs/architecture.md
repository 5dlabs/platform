# Task Board API - System Architecture

## Overview

The Task Board API is a collaborative task management service implemented as a Rust microservice using gRPC for communication. This document outlines the system architecture, component interactions, and design decisions for the implementation.

## System Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   gRPC Client   │    │   gRPC Client   │    │   gRPC Client   │
│   (Web/Mobile)  │    │   (CLI Tools)   │    │  (Other APIs)   │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │                         │
                    │    Task Board API       │
                    │   (Rust gRPC Server)    │
                    │                         │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │                         │
                    │    PostgreSQL DB        │
                    │                         │
                    └─────────────────────────┘
```

### Technology Stack

- **Language**: Rust (latest stable)
- **Communication**: gRPC with Tonic framework
- **Database**: PostgreSQL with Diesel ORM
- **Async Runtime**: Tokio
- **Authentication**: JWT with jsonwebtoken crate
- **Password Hashing**: Argon2 or bcrypt
- **Logging**: Structured logging with tracing crate
- **Testing**: tokio::test for async testing
- **Containerization**: Docker + Kubernetes

## Component Architecture

### 1. gRPC Service Layer

The application is structured around three main gRPC services:

#### User Service
- **Purpose**: Handle user registration, authentication, and management
- **Key Methods**:
  - `RegisterUser(RegisterRequest) -> RegisterResponse`
  - `LoginUser(LoginRequest) -> LoginResponse`
  - `ValidateToken(TokenRequest) -> TokenResponse`
  - `GetUserProfile(UserRequest) -> UserResponse`

#### Board Service
- **Purpose**: Manage project boards and board membership
- **Key Methods**:
  - `CreateBoard(CreateBoardRequest) -> BoardResponse`
  - `GetBoard(GetBoardRequest) -> BoardResponse`
  - `UpdateBoard(UpdateBoardRequest) -> BoardResponse`
  - `DeleteBoard(DeleteBoardRequest) -> EmptyResponse`
  - `ListBoards(ListBoardsRequest) -> ListBoardsResponse`
  - `AddBoardMember(AddMemberRequest) -> EmptyResponse`
  - `RemoveBoardMember(RemoveMemberRequest) -> EmptyResponse`

#### Task Service
- **Purpose**: Handle task CRUD operations and real-time updates
- **Key Methods**:
  - `CreateTask(CreateTaskRequest) -> TaskResponse`
  - `GetTask(GetTaskRequest) -> TaskResponse`
  - `UpdateTask(UpdateTaskRequest) -> TaskResponse`
  - `DeleteTask(DeleteTaskRequest) -> EmptyResponse`
  - `ListTasks(ListTasksRequest) -> ListTasksResponse`
  - `StreamTaskUpdates(StreamRequest) -> Stream<TaskUpdate>` (bidirectional)

### 2. Database Schema

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Boards table
CREATE TABLE boards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Board members table (many-to-many relationship)
CREATE TABLE board_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(board_id, user_id)
);

-- Tasks table
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    assignee_id UUID REFERENCES users(id),
    status VARCHAR(50) NOT NULL DEFAULT 'todo',
    priority VARCHAR(50) NOT NULL DEFAULT 'medium',
    due_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_board_members_board_id ON board_members(board_id);
CREATE INDEX idx_board_members_user_id ON board_members(user_id);
CREATE INDEX idx_tasks_board_id ON tasks(board_id);
CREATE INDEX idx_tasks_assignee_id ON tasks(assignee_id);
CREATE INDEX idx_tasks_status ON tasks(status);
```

### 3. Authentication & Authorization

#### JWT Token Structure
```json
{
  "sub": "user_id",
  "email": "user@example.com",
  "role": "admin|member",
  "exp": 1234567890,
  "iat": 1234567890
}
```

#### Authorization Model
- **Admin**: Can create/modify/delete any board and task
- **Board Owner**: Can modify board settings and manage members
- **Board Member**: Can view board and create/modify assigned tasks
- **Non-member**: No access to board data

### 4. Real-time Streaming Architecture

#### Streaming Implementation
```rust
// Bidirectional streaming for real-time updates
impl TaskService for TaskServiceImpl {
    type StreamTaskUpdatesStream = ReceiverStream<Result<TaskUpdate, Status>>;
    
    async fn stream_task_updates(
        &self,
        request: Request<Streaming<StreamRequest>>,
    ) -> Result<Response<Self::StreamTaskUpdatesStream>, Status> {
        // Connection management and real-time updates
    }
}
```

#### Connection Management
- **Client Registration**: Clients register for board-specific updates
- **Event Broadcasting**: Task changes broadcast to relevant clients
- **Connection Lifecycle**: Handle disconnections and reconnections gracefully
- **Resource Cleanup**: Remove client subscriptions on disconnect

## Data Flow Patterns

### 1. User Authentication Flow
```
Client -> LoginRequest -> UserService -> Database -> Password Verification -> JWT Generation -> LoginResponse -> Client
```

### 2. Task Creation Flow
```
Client -> CreateTaskRequest -> Authorization Check -> TaskService -> Database Insert -> Event Broadcast -> Real-time Clients
```

### 3. Real-time Update Flow
```
Task Update -> Event Queue -> Connected Clients Filter -> gRPC Stream -> Client Notification
```

## Error Handling Strategy

### Custom Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum TaskBoardError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Authorization denied: {0}")]
    AuthorizationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Not found: {0}")]
    NotFoundError(String),
}
```

### Error Response Format
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Task title is required",
    "details": {
      "field": "title",
      "constraint": "required"
    }
  }
}
```

## Logging & Observability

### Structured Logging
```rust
use tracing::{info, error, instrument};

#[instrument(skip(self, request))]
async fn create_task(&self, request: CreateTaskRequest) -> Result<TaskResponse> {
    info!(
        user_id = %request.user_id,
        board_id = %request.board_id,
        "Creating new task"
    );
    // Implementation
}
```

### Key Metrics
- Request latency per endpoint
- Active streaming connections
- Database query performance
- Authentication success/failure rates
- Task creation/completion rates

## Testing Strategy

### Test Pyramid
1. **Unit Tests**: Individual service methods with mocked dependencies
2. **Integration Tests**: gRPC endpoints with test database
3. **End-to-End Tests**: Full workflow testing with real components

### Test Infrastructure
```rust
// Test setup with container
#[tokio::test]
async fn test_create_task_endpoint() {
    let test_db = setup_test_database().await;
    let service = TaskService::new(test_db);
    
    // Test implementation
}
```

## Deployment Architecture

### Container Strategy
```dockerfile
# Multi-stage build for smaller production image
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/task-board-api /usr/local/bin/
CMD ["task-board-api"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: task-board-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: task-board-api
  template:
    metadata:
      labels:
        app: task-board-api
    spec:
      containers:
      - name: api
        image: task-board-api:latest
        ports:
        - containerPort: 50051
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
```

## Performance Considerations

### Database Optimization
- Connection pooling with Diesel
- Indexed queries for common operations
- Efficient pagination for large datasets
- Read replicas for scaling reads

### gRPC Optimization
- Connection multiplexing
- Streaming for large responses
- Compression for reduced bandwidth
- Keep-alive for long-lived connections

### Memory Management
- Bounded channels for streaming
- Connection limits to prevent resource exhaustion
- Graceful degradation under load

## Security Considerations

### Authentication Security
- Strong password hashing (Argon2/bcrypt)
- JWT with short expiration times
- Secure token storage recommendations
- Rate limiting on auth endpoints

### Authorization Security
- Role-based access control
- Resource-level permissions
- Input validation and sanitization
- SQL injection prevention via ORM

### Network Security
- TLS encryption for gRPC
- Certificate-based authentication
- Network policies in Kubernetes
- Secrets management best practices

## Scalability Design

### Horizontal Scaling
- Stateless service design
- Load balancing across instances
- Database connection pooling
- Shared session state via JWT

### Vertical Scaling
- Async I/O with Tokio
- Efficient memory usage
- Connection reuse
- Resource monitoring and alerting

This architecture provides a robust foundation for the Task Board API, balancing simplicity with the ability to handle collaborative real-time features and scale as needed.