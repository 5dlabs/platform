# Task 5: Implement Project Board CRUD Service

## Overview

This task implements the Project Board CRUD (Create, Read, Update, Delete) service for the Task Board API. The service manages project boards that serve as containers for tasks, supporting team collaboration through board membership and access control. It integrates with the authentication system from Task 4 to ensure proper authorization.

## Architecture Context

Based on the system architecture:

### Board Management Features
- **Board Operations**: Create, read, update, and delete project boards
- **Membership Management**: Add/remove members with specific roles
- **Access Control**: Owner-based permissions with member access
- **Data Model**: Boards contain tasks, have owners and members

### Authorization Model
- **Board Owner**: Full control over board settings and membership
- **Board Member**: Can view board and create/modify tasks
- **Admin Role**: Can perform any operation on any board
- **Non-members**: No access to board data

### Service Integration
- **gRPC Interface**: Implements BoardService trait from proto definitions
- **Authentication**: Validates JWT tokens for all operations
- **Database Layer**: Uses Diesel models for board persistence
- **Relationship Management**: Handles board-member many-to-many relationships

## Implementation Details

### 1. gRPC Service Implementation

```rust
// src/services/board_service.rs
use tonic::{Request, Response, Status};
use crate::proto::board_service_server::BoardService;
use crate::proto::{
    CreateBoardRequest, BoardResponse,
    GetBoardRequest, UpdateBoardRequest,
    DeleteBoardRequest, EmptyResponse,
    ListBoardsRequest, ListBoardsResponse,
    AddMemberRequest, RemoveMemberRequest
};

pub struct BoardServiceImpl {
    db_pool: DbPool,
    auth_service: Arc<AuthService>,
}

#[tonic::async_trait]
impl BoardService for BoardServiceImpl {
    async fn create_board(
        &self,
        request: Request<CreateBoardRequest>,
    ) -> Result<Response<BoardResponse>, Status> {
        // Extract and validate JWT from metadata
        let user_id = self.auth_service.validate_request(&request)?;
        
        // Implementation
    }

    async fn get_board(
        &self,
        request: Request<GetBoardRequest>,
    ) -> Result<Response<BoardResponse>, Status> {
        // Implementation with access control
    }

    async fn update_board(
        &self,
        request: Request<UpdateBoardRequest>,
    ) -> Result<Response<BoardResponse>, Status> {
        // Only owner or admin can update
    }

    async fn delete_board(
        &self,
        request: Request<DeleteBoardRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Only owner or admin can delete
    }

    async fn list_boards(
        &self,
        request: Request<ListBoardsRequest>,
    ) -> Result<Response<ListBoardsResponse>, Status> {
        // List boards user is member/owner of
    }

    async fn add_board_member(
        &self,
        request: Request<AddMemberRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Only owner or admin can add members
    }

    async fn remove_board_member(
        &self,
        request: Request<RemoveMemberRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        // Only owner or admin can remove members
    }
}
```

### 2. Authorization Middleware

```rust
// src/auth/middleware.rs
use tonic::{Request, Status};
use crate::auth::jwt::Claims;

pub struct AuthService {
    jwt_manager: JwtManager,
}

impl AuthService {
    pub fn validate_request<T>(&self, request: &Request<T>) -> Result<Uuid, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        let claims = self.jwt_manager
            .validate_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        Uuid::parse_str(&claims.sub)
            .map_err(|_| Status::internal("Invalid user ID in token"))
    }

    pub fn get_user_role(&self, request: &Request<impl std::any::Any>) -> Result<String, Status> {
        // Extract role from JWT claims
    }
}
```

### 3. Board Access Control

```rust
// src/services/board_service.rs
impl BoardServiceImpl {
    async fn check_board_access(
        &self,
        user_id: Uuid,
        board_id: Uuid,
        required_permission: BoardPermission,
    ) -> Result<(), Status> {
        // Check if user is admin
        let user = self.get_user(user_id).await?;
        if user.role == "admin" {
            return Ok(());
        }

        // Check board membership
        let membership = self.get_board_membership(user_id, board_id).await?;
        
        match required_permission {
            BoardPermission::View => {
                if membership.is_some() {
                    Ok(())
                } else {
                    Err(Status::permission_denied("Not a board member"))
                }
            }
            BoardPermission::Modify => {
                match membership {
                    Some(m) if m.role == "owner" => Ok(()),
                    _ => Err(Status::permission_denied("Only board owner can modify"))
                }
            }
        }
    }
}

#[derive(Debug)]
enum BoardPermission {
    View,
    Modify,
}
```

### 4. Database Operations

```rust
// src/db/board_queries.rs
use diesel::prelude::*;
use crate::models::{Board, NewBoard, BoardMember, NewBoardMember};

pub async fn create_board_with_owner(
    pool: &DbPool,
    title: String,
    description: Option<String>,
    owner_id: Uuid,
) -> Result<Board, diesel::result::Error> {
    run_async_query(pool, move |conn| {
        conn.transaction::<Board, diesel::result::Error, _>(|conn| {
            // Create board
            let new_board = NewBoard {
                title: &title,
                description: description.as_deref(),
                owner_id,
            };

            let board = diesel::insert_into(boards::table)
                .values(&new_board)
                .get_result::<Board>(conn)?;

            // Add owner as member
            let new_member = NewBoardMember {
                board_id: board.id,
                user_id: owner_id,
                role: "owner",
            };

            diesel::insert_into(board_members::table)
                .values(&new_member)
                .execute(conn)?;

            Ok(board)
        })
    })
    .await
}

pub async fn get_user_boards(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<Board>, diesel::result::Error> {
    run_async_query(pool, move |conn| {
        boards::table
            .inner_join(board_members::table)
            .filter(board_members::user_id.eq(user_id))
            .select(boards::all_columns)
            .load::<Board>(conn)
    })
    .await
}
```

### 5. Service Method Implementations

```rust
// Create Board
async fn create_board(
    &self,
    request: Request<CreateBoardRequest>,
) -> Result<Response<BoardResponse>, Status> {
    let user_id = self.auth_service.validate_request(&request)?;
    let req = request.into_inner();

    // Validate input
    if req.title.trim().is_empty() {
        return Err(Status::invalid_argument("Board title cannot be empty"));
    }

    if req.title.len() > 255 {
        return Err(Status::invalid_argument("Board title too long"));
    }

    // Create board with owner
    let board = create_board_with_owner(
        &self.db_pool,
        req.title,
        req.description,
        user_id,
    )
    .await
    .map_err(|e| Status::internal(format!("Failed to create board: {}", e)))?;

    // Get member count
    let member_count = self.get_board_member_count(board.id).await?;

    Ok(Response::new(BoardResponse {
        id: board.id.to_string(),
        title: board.title,
        description: board.description,
        owner_id: board.owner_id.to_string(),
        member_count: member_count as i32,
        created_at: board.created_at.to_rfc3339(),
        updated_at: board.updated_at.to_rfc3339(),
    }))
}

// List Boards
async fn list_boards(
    &self,
    request: Request<ListBoardsRequest>,
) -> Result<Response<ListBoardsResponse>, Status> {
    let user_id = self.auth_service.validate_request(&request)?;
    let req = request.into_inner();

    // Apply pagination
    let limit = req.limit.unwrap_or(20).min(100) as i64;
    let offset = req.offset.unwrap_or(0) as i64;

    // Get user's boards
    let boards = get_user_boards_paginated(
        &self.db_pool,
        user_id,
        limit,
        offset,
    )
    .await
    .map_err(|e| Status::internal(format!("Failed to list boards: {}", e)))?;

    // Convert to proto format
    let board_responses = futures::future::try_join_all(
        boards.into_iter().map(|board| async {
            let member_count = self.get_board_member_count(board.id).await?;
            Ok::<BoardResponse, Status>(BoardResponse {
                id: board.id.to_string(),
                title: board.title,
                description: board.description,
                owner_id: board.owner_id.to_string(),
                member_count: member_count as i32,
                created_at: board.created_at.to_rfc3339(),
                updated_at: board.updated_at.to_rfc3339(),
            })
        })
    ).await?;

    Ok(Response::new(ListBoardsResponse {
        boards: board_responses,
        total_count: self.get_user_board_count(user_id).await? as i32,
    }))
}
```

### 6. Board Membership Management

```rust
// Add Member
async fn add_board_member(
    &self,
    request: Request<AddMemberRequest>,
) -> Result<Response<EmptyResponse>, Status> {
    let requesting_user_id = self.auth_service.validate_request(&request)?;
    let req = request.into_inner();

    let board_id = Uuid::parse_str(&req.board_id)
        .map_err(|_| Status::invalid_argument("Invalid board ID"))?;
    
    let user_id = Uuid::parse_str(&req.user_id)
        .map_err(|_| Status::invalid_argument("Invalid user ID"))?;

    // Check permission
    self.check_board_access(requesting_user_id, board_id, BoardPermission::Modify).await?;

    // Check if user exists
    self.get_user(user_id).await
        .map_err(|_| Status::not_found("User not found"))?;

    // Check if already member
    if self.is_board_member(board_id, user_id).await? {
        return Err(Status::already_exists("User is already a board member"));
    }

    // Add member
    let new_member = NewBoardMember {
        board_id,
        user_id,
        role: req.role.as_deref().unwrap_or("member"),
    };

    run_async_query(&self.db_pool, move |conn| {
        diesel::insert_into(board_members::table)
            .values(&new_member)
            .execute(conn)
    })
    .await
    .map_err(|e| Status::internal(format!("Failed to add member: {}", e)))?;

    Ok(Response::new(EmptyResponse {}))
}
```

### 7. Error Handling

```rust
// src/errors/board.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BoardError {
    #[error("Board not found")]
    NotFound,
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Invalid board data: {0}")]
    ValidationError(String),
    
    #[error("Member already exists")]
    MemberExists,
    
    #[error("Cannot remove board owner")]
    CannotRemoveOwner,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
}

impl From<BoardError> for Status {
    fn from(err: BoardError) -> Self {
        match err {
            BoardError::NotFound => Status::not_found(err.to_string()),
            BoardError::AccessDenied(_) => Status::permission_denied(err.to_string()),
            BoardError::ValidationError(_) => Status::invalid_argument(err.to_string()),
            BoardError::MemberExists => Status::already_exists(err.to_string()),
            BoardError::CannotRemoveOwner => Status::failed_precondition(err.to_string()),
            BoardError::DatabaseError(_) => Status::internal(err.to_string()),
        }
    }
}
```

## Dependencies

### Task Dependencies
- **Task 2**: gRPC Service Contracts (BoardService proto definition)
- **Task 3**: Database Schema and ORM (Board and BoardMember models)
- **Task 4**: User Management Service (Authentication and authorization)

### Cargo Dependencies
```toml
[dependencies]
# Already included from previous tasks
tonic = "0.9"
diesel = { version = "2.1", features = ["postgres", "uuid", "chrono"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# Error handling
thiserror = "1.0"

# Tracing
tracing = "0.1"
```

## Testing Strategy

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_validation() {
        // Test empty title rejection
        // Test title length limits
        // Test description optional
    }

    #[test]
    fn test_permission_checks() {
        // Test owner has modify permission
        // Test member has view permission
        // Test non-member has no permission
        // Test admin bypasses checks
    }
}
```

### 2. Integration Tests

```rust
#[tokio::test]
async fn test_board_lifecycle() {
    // Create board
    // Verify owner is member
    // Update board
    // Add members
    // Remove members
    // Delete board
    // Verify cascade deletes
}

#[tokio::test]
async fn test_authorization_enforcement() {
    // Non-member cannot view board
    // Member cannot update board
    // Owner can perform all operations
    // Admin can perform all operations
}

#[tokio::test]
async fn test_membership_management() {
    // Add member to board
    // Verify member can view
    // Remove member
    // Verify member cannot view
    // Cannot remove owner
}
```

### 3. Concurrency Tests

```rust
#[tokio::test]
async fn test_concurrent_member_additions() {
    // Multiple users adding members simultaneously
    // Verify no duplicate memberships
    // Verify all valid additions succeed
}
```

## Subtask Breakdown

1. **Design CRUD Endpoints** - Define API structure and permissions
2. **Integrate Database Operations** - Connect endpoints to Diesel queries
3. **Implement Authorization Checks** - Ensure proper access control
4. **Add Error Handling** - Comprehensive error types and messages
5. **Test Endpoint Functionality** - Unit and integration tests
6. **Document API Endpoints** - Clear usage documentation

## Success Metrics

- All CRUD operations functional with proper authorization
- Board membership management working correctly
- Cascade deletes remove related data
- Response times under 100ms for all operations
- 100% test coverage for authorization logic
- No unauthorized access possible
- Clear error messages for all failure cases
