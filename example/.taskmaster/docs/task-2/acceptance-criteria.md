# Acceptance Criteria for Task 2: Define gRPC Service Contracts

## Functional Requirements

### FR-1: Service Definition Coverage
- **FR-1.1**: UserService must define 6 RPC methods for authentication and user management
- **FR-1.2**: BoardService must define 8 RPC methods for board CRUD and member management
- **FR-1.3**: TaskService must define 7 RPC methods including bidirectional streaming
- **FR-1.4**: All services must use consistent naming patterns for CRUD operations

### FR-2: Message Type Requirements
- **FR-2.1**: Common types (enums and shared messages) must be defined in common.proto
- **FR-2.2**: Each RPC method must have dedicated Request and Response message types
- **FR-2.3**: All timestamp fields must use google.protobuf.Timestamp
- **FR-2.4**: List operations must support pagination

### FR-3: Streaming Requirements
- **FR-3.1**: TaskService must support bidirectional streaming for real-time updates
- **FR-3.2**: Streaming must support subscribe/unsubscribe operations
- **FR-3.3**: Update messages must indicate the type of change (created, updated, deleted, assigned)

## Technical Requirements

### TR-1: Proto3 Syntax and Standards
- **TR-1.1**: All proto files must use `syntax = "proto3"`
- **TR-1.2**: Package must be `taskboard.v1` for API versioning
- **TR-1.3**: Field numbers must start at 1 and be sequential
- **TR-1.4**: Enums must start with UNSPECIFIED = 0

### TR-2: Field and Message Design
- **TR-2.1**: Optional fields must use proto3 optional syntax
- **TR-2.2**: Message names must use TitleCase
- **TR-2.3**: Field names must use snake_case
- **TR-2.4**: Service and RPC names must use TitleCase

### TR-3: Build Integration
- **TR-3.1**: build.rs must compile all proto files with tonic-build
- **TR-3.2**: Generated code must be accessible through src/proto.rs module
- **TR-3.3**: Common types must be re-exported for easy access
- **TR-3.4**: Build must use --experimental_allow_proto3_optional flag

### TR-4: Data Model Alignment
- **TR-4.1**: User message must include: id, email, full_name, role, created_at, updated_at
- **TR-4.2**: Board message must include: id, title, description, owner_id, timestamps
- **TR-4.3**: Task message must include all fields specified in PRD
- **TR-4.4**: Enums must match architecture specification exactly

## Test Cases

### TC-1: Proto Compilation Tests
```bash
# Test Case 1.1: Verify proto files exist
ls proto/common.proto proto/user.proto proto/board.proto proto/task.proto
# Expected: All files exist

# Test Case 1.2: Validate proto syntax
protoc --proto_path=proto --descriptor_set_out=/dev/null proto/*.proto
# Expected: No errors

# Test Case 1.3: Build project with proto compilation
cargo build
# Expected: Build succeeds, generates code in target/
```

### TC-2: Service Definition Tests
```bash
# Test Case 2.1: Verify UserService methods
grep -E "rpc (RegisterUser|LoginUser|ValidateToken|GetUserProfile|UpdateUserProfile|ListUsers)" proto/user.proto | wc -l
# Expected: 6

# Test Case 2.2: Verify BoardService methods
grep -E "rpc (CreateBoard|GetBoard|UpdateBoard|DeleteBoard|ListBoards|AddBoardMember|RemoveBoardMember|ListBoardMembers)" proto/board.proto | wc -l
# Expected: 8

# Test Case 2.3: Verify TaskService methods
grep -E "rpc (CreateTask|GetTask|UpdateTask|DeleteTask|ListTasks|AssignTask|StreamTaskUpdates)" proto/task.proto | wc -l
# Expected: 7

# Test Case 2.4: Verify streaming RPC
grep "rpc StreamTaskUpdates(stream StreamTaskUpdatesRequest) returns (stream StreamTaskUpdatesResponse)" proto/task.proto
# Expected: Match found
```

### TC-3: Message Structure Tests
```rust
// Test Case 3.1: Verify generated types compile
// Create test file: src/proto_validation_test.rs
#[cfg(test)]
mod tests {
    use crate::proto::*;
    
    #[test]
    fn test_enum_values() {
        // Verify enums have expected values
        assert_eq!(UserRole::UserRoleUnspecified as i32, 0);
        assert_eq!(TaskStatus::TaskStatusUnspecified as i32, 0);
        assert_eq!(TaskPriority::TaskPriorityUnspecified as i32, 0);
    }
    
    #[test]
    fn test_message_creation() {
        use crate::proto::user::User;
        
        // Verify User message can be created
        let user = User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            full_name: "Test User".to_string(),
            role: UserRole::UserRoleMember as i32,
            created_at: None,
            updated_at: None,
        };
        
        assert_eq!(user.email, "test@example.com");
    }
}
```

### TC-4: Import and Dependency Tests
```bash
# Test Case 4.1: Verify common.proto imports
grep 'import "common.proto"' proto/user.proto proto/board.proto proto/task.proto | wc -l
# Expected: 3

# Test Case 4.2: Verify timestamp imports
grep 'import "google/protobuf/timestamp.proto"' proto/*.proto | wc -l
# Expected: 4 (all proto files)

# Test Case 4.3: Verify package declaration
grep 'package taskboard.v1;' proto/*.proto | wc -l
# Expected: 4
```

## Verification Steps

### Step 1: Proto File Structure Verification
1. Navigate to proto directory
2. Verify all 4 proto files exist (common, user, board, task)
3. Check each file has proper syntax declaration
4. Verify package naming is consistent

### Step 2: Service Coverage Verification
1. Open each service proto file
2. Count RPC methods match requirements
3. Verify each method has Request/Response types
4. Check streaming syntax for TaskService

### Step 3: Message Type Verification
1. Check common.proto defines all shared enums
2. Verify enum values start with UNSPECIFIED = 0
3. Confirm pagination messages in common.proto
4. Validate timestamp usage in all messages

### Step 4: Build Integration Verification
1. Run `cargo build` to compile protos
2. Check for generated files in target directory
3. Verify src/proto.rs module exists and compiles
4. Test type imports in a sample file

### Step 5: Best Practices Verification
1. Check field numbering is sequential
2. Verify naming conventions (TitleCase, snake_case)
3. Confirm no deprecated proto2 features used
4. Validate optional field syntax

## Success Metrics

### Quantitative Metrics
- Proto files: 4 files created
- Service methods: 21 total RPCs defined
- Enum types: 3 with UNSPECIFIED values
- Build errors: 0
- Compilation warnings: 0

### Qualitative Metrics
- Consistent naming conventions throughout
- Clear separation of concerns between files
- Forward-compatible message design
- Proper use of proto3 features
- Well-organized imports and dependencies

## Edge Cases and Error Handling

### EC-1: Proto Compilation Errors
- Missing imports should fail fast with clear errors
- Invalid syntax should be caught by protoc
- Circular dependencies must be avoided

### EC-2: Optional Field Handling
- Proto3 optional fields require experimental flag
- Ensure build.rs includes necessary protoc arguments
- Test that optional fields work as expected

### EC-3: Streaming Considerations
- Bidirectional streaming requires proper syntax
- Connection lifecycle must be considered
- Error propagation in streams needs design

## Dependencies and Blockers

### Dependencies
- Task 1: Requires protoc compiler installed
- Requires tonic-build in build dependencies
- Needs proto directory structure created

### Enables
- Task 4: User Management Service (requires UserService contract)
- Task 5: Board CRUD Service (requires BoardService contract)
- Task 6: Task CRUD Service (requires TaskService contract)
- Task 7: Real-time Updates (requires streaming contract)

### Potential Blockers
- Incorrect protoc version (<3.21)
- Missing proto3 optional support
- Build system configuration issues
- Import path resolution problems