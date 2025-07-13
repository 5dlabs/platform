# Autonomous Prompt for Task 2: Define gRPC Service Contracts

## Context

You are implementing the gRPC service contracts for the Task Board API. The project uses Protocol Buffers (proto3) to define service interfaces and message types. These contracts form the foundation of the API and must align with the architecture specification for three main services: UserService, BoardService, and TaskService.

The API supports:
- User authentication and management
- Project board CRUD operations with member management
- Task CRUD operations with assignment
- Real-time task updates via bidirectional streaming

## Task Requirements

### Primary Objective
Create well-designed Protocol Buffer definitions that define all gRPC services, methods, and message types for the Task Board API.

### Service Definitions Required

1. **UserService**
   - RegisterUser - Create new user account
   - LoginUser - Authenticate and receive JWT tokens
   - ValidateToken - Verify JWT validity
   - GetUserProfile - Retrieve user information
   - UpdateUserProfile - Modify user details
   - ListUsers - Admin endpoint for user listing

2. **BoardService**
   - CreateBoard - Create new project board
   - GetBoard - Retrieve board with members
   - UpdateBoard - Modify board details
   - DeleteBoard - Remove board
   - ListBoards - Get user's accessible boards
   - AddBoardMember - Add user to board
   - RemoveBoardMember - Remove user from board
   - ListBoardMembers - Get board members

3. **TaskService**
   - CreateTask - Create new task in board
   - GetTask - Retrieve task details
   - UpdateTask - Modify task
   - DeleteTask - Remove task
   - ListTasks - Get filtered task list
   - AssignTask - Assign/unassign user
   - StreamTaskUpdates - Bidirectional streaming for real-time updates

### Message Design Requirements
- Use proto3 syntax with optional fields where appropriate
- Include common types (UserRole, TaskStatus, TaskPriority)
- Implement pagination for list operations
- Design streaming messages for subscribe/unsubscribe and updates
- Use google.protobuf.Timestamp for time fields
- Include proper field numbering and naming conventions

## Implementation Instructions

### Step 1: Create Proto Directory Structure
```bash
mkdir -p proto
cd proto
```

### Step 2: Create Common Types (common.proto)
Define shared enums and messages:
- UserRole enum (UNSPECIFIED, MEMBER, ADMIN)
- TaskStatus enum (UNSPECIFIED, TODO, IN_PROGRESS, DONE, ARCHIVED)
- TaskPriority enum (UNSPECIFIED, LOW, MEDIUM, HIGH, CRITICAL)
- PaginationRequest/Response messages
- ErrorDetails message

### Step 3: Create User Service (user.proto)
Define UserService with all methods and related messages:
- User message with id, email, full_name, role, timestamps
- Request/Response pairs for each RPC method
- Include JWT tokens in auth responses

### Step 4: Create Board Service (board.proto)
Define BoardService with CRUD and member management:
- Board message with id, title, description, owner_id, timestamps
- BoardMember message for membership details
- Request/Response pairs for all operations

### Step 5: Create Task Service (task.proto)
Define TaskService with CRUD and streaming:
- Task message with all fields per PRD
- Filter options for list operations
- Streaming messages using oneof for different update types

### Step 6: Configure Build System
Update build.rs:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "proto/common.proto",
        "proto/user.proto", 
        "proto/board.proto",
        "proto/task.proto",
    ];
    
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(proto_files, &["proto"])?;
    
    Ok(())
}
```

### Step 7: Create Proto Module
Create src/proto.rs to expose generated code:
```rust
pub mod common {
    tonic::include_proto!("taskboard.v1");
}
// ... repeat for other modules

// Re-export common types
pub use common::{UserRole, TaskStatus, TaskPriority};
```

### Step 8: Validate Proto Files
Create and run validation script to ensure proto files are valid and follow best practices.

## Success Criteria

### Proto File Validation
- [ ] All proto files compile without errors
- [ ] Field numbers are sequential starting from 1
- [ ] No deprecated proto2 features used
- [ ] Consistent naming (TitleCase for messages/services)
- [ ] All enums start with UNSPECIFIED = 0

### Service Coverage
- [ ] UserService has all 6 required methods
- [ ] BoardService has all 8 required methods  
- [ ] TaskService has all 7 required methods including streaming
- [ ] All methods have proper request/response messages

### Message Design
- [ ] Common types defined and imported correctly
- [ ] Pagination implemented for list operations
- [ ] Optional fields used appropriately in proto3
- [ ] Timestamps use google.protobuf.Timestamp
- [ ] Streaming messages use oneof for polymorphism

### Build Integration
- [ ] build.rs compiles all proto files
- [ ] Generated code accessible via src/proto.rs
- [ ] No compilation warnings or errors
- [ ] Types can be imported and used in tests

### Best Practices
- [ ] Clear, descriptive field and message names
- [ ] Proper comments on services and complex messages
- [ ] Forward-compatible design (room for additions)
- [ ] Consistent patterns across all services
- [ ] Proper error handling considerations

### Testing
- [ ] Proto syntax validation passes
- [ ] Generated code imports work
- [ ] Basic type usage tests pass
- [ ] All expected types are accessible

## Important Notes
- Use `syntax = "proto3"` for all files
- Package should be `taskboard.v1` for versioning
- Import paths must be relative to proto directory
- Use `--experimental_allow_proto3_optional` for optional fields
- Ensure streaming RPCs use proper stream keywords
- Consider future extensibility in message design