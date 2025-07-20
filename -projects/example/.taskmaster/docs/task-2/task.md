# Task 2: Define gRPC Service Contracts

## Overview

This task involves designing and implementing Protocol Buffers definitions for all gRPC services in the Task Board API. These contracts define the communication interface between clients and the server, establishing the foundation for all API interactions.

## Task Context

### Description
Design and implement Protocol Buffers definitions for all gRPC services.

### Priority
High - Service contracts must be defined before implementing any service logic.

### Dependencies
- Task 1: Setup Project Repository and Toolchain (protoc compiler required)

### Subtasks
1. Design User Service Proto
2. Design Board and Task Service Protos
3. Validate Proto Files for Style and Best Practices
4. Generate and Test Code from Proto Files

## Architecture Context

According to architecture.md, the gRPC service layer consists of three main services:

1. **User Service**: Authentication and user management
   - RegisterUser, LoginUser, ValidateToken, GetUserProfile

2. **Board Service**: Project board and membership management
   - CRUD operations for boards
   - Member management (add, remove, list)

3. **Task Service**: Task CRUD and real-time streaming
   - CRUD operations for tasks
   - Task assignment
   - Bidirectional streaming for real-time updates

Each service requires carefully designed Protocol Buffers definitions that support the specified RPC methods and message types.

## Implementation Details

### 1. Proto File Structure

Create the following proto files in the `proto/` directory:

#### proto/common.proto
```protobuf
syntax = "proto3";

package taskboard.v1;

import "google/protobuf/timestamp.proto";

// Common enums and messages used across services

enum UserRole {
  USER_ROLE_UNSPECIFIED = 0;
  USER_ROLE_MEMBER = 1;
  USER_ROLE_ADMIN = 2;
}

enum TaskStatus {
  TASK_STATUS_UNSPECIFIED = 0;
  TASK_STATUS_TODO = 1;
  TASK_STATUS_IN_PROGRESS = 2;
  TASK_STATUS_DONE = 3;
  TASK_STATUS_ARCHIVED = 4;
}

enum TaskPriority {
  TASK_PRIORITY_UNSPECIFIED = 0;
  TASK_PRIORITY_LOW = 1;
  TASK_PRIORITY_MEDIUM = 2;
  TASK_PRIORITY_HIGH = 3;
  TASK_PRIORITY_CRITICAL = 4;
}

// Pagination request/response messages
message PaginationRequest {
  int32 page_size = 1;
  string page_token = 2;
}

message PaginationResponse {
  string next_page_token = 1;
  int32 total_count = 2;
}

// Common error details
message ErrorDetails {
  string field = 1;
  string message = 2;
}
```

#### proto/user.proto
```protobuf
syntax = "proto3";

package taskboard.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

service UserService {
  // Register a new user
  rpc RegisterUser(RegisterUserRequest) returns (RegisterUserResponse);
  
  // Login with credentials
  rpc LoginUser(LoginUserRequest) returns (LoginUserResponse);
  
  // Validate JWT token
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  
  // Get user profile
  rpc GetUserProfile(GetUserProfileRequest) returns (GetUserProfileResponse);
  
  // Update user profile
  rpc UpdateUserProfile(UpdateUserProfileRequest) returns (UpdateUserProfileResponse);
  
  // List users (admin only)
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
}

// User messages
message User {
  string id = 1;
  string email = 2;
  string full_name = 3;
  UserRole role = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
}

// Register messages
message RegisterUserRequest {
  string email = 1;
  string password = 2;
  string full_name = 3;
}

message RegisterUserResponse {
  User user = 1;
  string access_token = 2;
  string refresh_token = 3;
}

// Login messages
message LoginUserRequest {
  string email = 1;
  string password = 2;
}

message LoginUserResponse {
  User user = 1;
  string access_token = 2;
  string refresh_token = 3;
}

// Token validation
message ValidateTokenRequest {
  string token = 1;
}

message ValidateTokenResponse {
  bool valid = 1;
  string user_id = 2;
  UserRole role = 3;
}

// Profile messages
message GetUserProfileRequest {
  string user_id = 1;
}

message GetUserProfileResponse {
  User user = 1;
}

message UpdateUserProfileRequest {
  string user_id = 1;
  optional string full_name = 2;
  optional string email = 3;
}

message UpdateUserProfileResponse {
  User user = 1;
}

// List users
message ListUsersRequest {
  PaginationRequest pagination = 1;
  optional string search_query = 2;
}

message ListUsersResponse {
  repeated User users = 1;
  PaginationResponse pagination = 2;
}
```

#### proto/board.proto
```protobuf
syntax = "proto3";

package taskboard.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

service BoardService {
  // Create a new board
  rpc CreateBoard(CreateBoardRequest) returns (CreateBoardResponse);
  
  // Get board details
  rpc GetBoard(GetBoardRequest) returns (GetBoardResponse);
  
  // Update board
  rpc UpdateBoard(UpdateBoardRequest) returns (UpdateBoardResponse);
  
  // Delete board
  rpc DeleteBoard(DeleteBoardRequest) returns (DeleteBoardResponse);
  
  // List boards for user
  rpc ListBoards(ListBoardsRequest) returns (ListBoardsResponse);
  
  // Add member to board
  rpc AddBoardMember(AddBoardMemberRequest) returns (AddBoardMemberResponse);
  
  // Remove member from board
  rpc RemoveBoardMember(RemoveBoardMemberRequest) returns (RemoveBoardMemberResponse);
  
  // List board members
  rpc ListBoardMembers(ListBoardMembersRequest) returns (ListBoardMembersResponse);
}

// Board messages
message Board {
  string id = 1;
  string title = 2;
  string description = 3;
  string owner_id = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
}

message BoardMember {
  string id = 1;
  string board_id = 2;
  string user_id = 3;
  string user_email = 4;
  string user_full_name = 5;
  UserRole role = 6;
  google.protobuf.Timestamp joined_at = 7;
}

// Create board
message CreateBoardRequest {
  string title = 1;
  optional string description = 2;
}

message CreateBoardResponse {
  Board board = 1;
}

// Get board
message GetBoardRequest {
  string board_id = 1;
}

message GetBoardResponse {
  Board board = 1;
  repeated BoardMember members = 2;
}

// Update board
message UpdateBoardRequest {
  string board_id = 1;
  optional string title = 2;
  optional string description = 3;
}

message UpdateBoardResponse {
  Board board = 1;
}

// Delete board
message DeleteBoardRequest {
  string board_id = 1;
}

message DeleteBoardResponse {
  bool success = 1;
}

// List boards
message ListBoardsRequest {
  PaginationRequest pagination = 1;
  optional string search_query = 2;
}

message ListBoardsResponse {
  repeated Board boards = 1;
  PaginationResponse pagination = 2;
}

// Board members
message AddBoardMemberRequest {
  string board_id = 1;
  string user_email = 2;
  UserRole role = 3;
}

message AddBoardMemberResponse {
  BoardMember member = 1;
}

message RemoveBoardMemberRequest {
  string board_id = 1;
  string user_id = 2;
}

message RemoveBoardMemberResponse {
  bool success = 1;
}

message ListBoardMembersRequest {
  string board_id = 1;
}

message ListBoardMembersResponse {
  repeated BoardMember members = 1;
}
```

#### proto/task.proto
```protobuf
syntax = "proto3";

package taskboard.v1;

import "google/protobuf/timestamp.proto";
import "common.proto";

service TaskService {
  // Create a new task
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  
  // Get task details
  rpc GetTask(GetTaskRequest) returns (GetTaskResponse);
  
  // Update task
  rpc UpdateTask(UpdateTaskRequest) returns (UpdateTaskResponse);
  
  // Delete task
  rpc DeleteTask(DeleteTaskRequest) returns (DeleteTaskResponse);
  
  // List tasks in a board
  rpc ListTasks(ListTasksRequest) returns (ListTasksResponse);
  
  // Assign task to user
  rpc AssignTask(AssignTaskRequest) returns (AssignTaskResponse);
  
  // Stream task updates (bidirectional)
  rpc StreamTaskUpdates(stream StreamTaskUpdatesRequest) returns (stream StreamTaskUpdatesResponse);
}

// Task messages
message Task {
  string id = 1;
  string board_id = 2;
  string title = 3;
  string description = 4;
  optional string assignee_id = 5;
  optional string assignee_email = 6;
  optional string assignee_full_name = 7;
  TaskStatus status = 8;
  TaskPriority priority = 9;
  optional google.protobuf.Timestamp due_date = 10;
  google.protobuf.Timestamp created_at = 11;
  google.protobuf.Timestamp updated_at = 12;
}

// Create task
message CreateTaskRequest {
  string board_id = 1;
  string title = 2;
  optional string description = 3;
  optional string assignee_id = 4;
  TaskPriority priority = 5;
  optional google.protobuf.Timestamp due_date = 6;
}

message CreateTaskResponse {
  Task task = 1;
}

// Get task
message GetTaskRequest {
  string task_id = 1;
}

message GetTaskResponse {
  Task task = 1;
}

// Update task
message UpdateTaskRequest {
  string task_id = 1;
  optional string title = 2;
  optional string description = 3;
  optional TaskStatus status = 4;
  optional TaskPriority priority = 5;
  optional google.protobuf.Timestamp due_date = 6;
}

message UpdateTaskResponse {
  Task task = 1;
}

// Delete task
message DeleteTaskRequest {
  string task_id = 1;
}

message DeleteTaskResponse {
  bool success = 1;
}

// List tasks
message ListTasksRequest {
  string board_id = 1;
  PaginationRequest pagination = 2;
  optional TaskStatus status_filter = 3;
  optional string assignee_filter = 4;
  optional TaskPriority priority_filter = 5;
}

message ListTasksResponse {
  repeated Task tasks = 1;
  PaginationResponse pagination = 2;
}

// Assign task
message AssignTaskRequest {
  string task_id = 1;
  optional string assignee_id = 2; // null to unassign
}

message AssignTaskResponse {
  Task task = 1;
}

// Streaming
message StreamTaskUpdatesRequest {
  oneof request {
    SubscribeRequest subscribe = 1;
    UnsubscribeRequest unsubscribe = 2;
  }
  
  message SubscribeRequest {
    repeated string board_ids = 1;
  }
  
  message UnsubscribeRequest {
    repeated string board_ids = 1;
  }
}

message StreamTaskUpdatesResponse {
  oneof update {
    TaskCreated task_created = 1;
    TaskUpdated task_updated = 2;
    TaskDeleted task_deleted = 3;
    TaskAssigned task_assigned = 4;
  }
  
  message TaskCreated {
    Task task = 1;
  }
  
  message TaskUpdated {
    Task task = 1;
    repeated string changed_fields = 2;
  }
  
  message TaskDeleted {
    string task_id = 1;
    string board_id = 2;
  }
  
  message TaskAssigned {
    Task task = 1;
    optional string previous_assignee_id = 2;
  }
}
```

### 2. Build Configuration

Update `build.rs` to compile all proto files:

```rust
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "proto/common.proto",
        "proto/user.proto",
        "proto/board.proto",
        "proto/task.proto",
    ];
    
    // Tell Cargo to recompile if any proto file changes
    for proto in proto_files {
        println!("cargo:rerun-if-changed={}", proto);
    }
    
    let proto_path = PathBuf::from("proto");
    
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(proto_files, &[proto_path])?;
    
    Ok(())
}
```

### 3. Proto Validation Script

Create `scripts/validate-protos.sh`:

```bash
#!/bin/bash

# Validate proto files
echo "Validating proto files..."

# Check syntax
for proto in proto/*.proto; do
    echo "Checking $proto..."
    protoc --proto_path=proto --descriptor_set_out=/dev/null "$proto" || exit 1
done

# Run protolint if available
if command -v protolint &> /dev/null; then
    echo "Running protolint..."
    protolint lint proto/
else
    echo "protolint not found, skipping style checks"
fi

echo "Proto validation complete!"
```

### 4. Generated Code Module Structure

Create `src/proto.rs` to expose generated code:

```rust
// Re-export generated proto modules
pub mod common {
    tonic::include_proto!("taskboard.v1");
}

pub mod user {
    tonic::include_proto!("taskboard.v1");
}

pub mod board {
    tonic::include_proto!("taskboard.v1");
}

pub mod task {
    tonic::include_proto!("taskboard.v1");
}

// Re-export commonly used types
pub use common::{UserRole, TaskStatus, TaskPriority};
pub use user::user_service_server::UserServiceServer;
pub use board::board_service_server::BoardServiceServer;
pub use task::task_service_server::TaskServiceServer;
```

Update `src/main.rs` to include the proto module:

```rust
mod proto;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "task_board_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Task Board API server...");

    // TODO: Initialize services and start gRPC server

    Ok(())
}
```

## Dependencies

- Task 1: Requires protoc compiler and Rust toolchain
- Enables: Tasks 4-7 (all service implementations depend on these contracts)

## Testing Strategy

### Validation Tests
1. **Proto Compilation**
   ```bash
   cargo build
   # Should generate code without errors
   ```

2. **Proto Syntax Validation**
   ```bash
   chmod +x scripts/validate-protos.sh
   ./scripts/validate-protos.sh
   # All files should pass validation
   ```

3. **Generated Code Verification**
   ```bash
   # Check generated files exist
   ls target/debug/build/*/out/*.rs
   ```

4. **Import Resolution**
   ```rust
   // Create test file src/proto_test.rs
   #[cfg(test)]
   mod tests {
       use crate::proto::*;
       
       #[test]
       fn test_proto_imports() {
           // Verify types are accessible
           let _role = UserRole::UserRoleMember;
           let _status = TaskStatus::TaskStatusTodo;
           let _priority = TaskPriority::TaskPriorityHigh;
       }
   }
   ```

### Best Practices Validation
- Field numbering starts at 1 and is sequential
- No use of deprecated features (required fields, groups)
- Consistent naming conventions (TitleCase for messages/services)
- Proper use of optional fields in proto3
- Clear service method names following CRUD patterns
- Enums start with UNSPECIFIED = 0 for forward compatibility