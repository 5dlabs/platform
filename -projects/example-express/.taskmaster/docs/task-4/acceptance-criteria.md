# Task 4: Create Task Management API - Acceptance Criteria

## Overview
This document defines the acceptance criteria for the Task Management API endpoints. Each criterion includes verification steps to ensure the API provides secure, functional CRUD operations for tasks.

## API Structure Criteria

### ✓ Routes File Created
- **Requirement**: Task routes module exists
- **Verification**:
  ```bash
  test -f src/routes/tasks.js && echo "Tasks router exists"
  ```
- **Expected**: File exists with all route handlers

### ✓ Routes Integrated with Express
- **Requirement**: Task routes mounted at /api/tasks
- **Verification**:
  ```bash
  grep -n "app.use('/api/tasks'" src/app.js
  ```
- **Expected**: Route is registered after auth routes

### ✓ Authentication Applied
- **Requirement**: All task routes require authentication
- **Verification**: Try accessing without token
  ```bash
  curl http://localhost:3000/api/tasks
  ```
- **Expected**: 401 "Access token required"

## GET /api/tasks Criteria

### ✓ List User's Tasks
- **Requirement**: Returns only authenticated user's tasks
- **Test**:
  ```bash
  curl http://localhost:3000/api/tasks \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected Response Structure**:
  ```json
  {
    "tasks": [
      {
        "id": 1,
        "title": "Task title",
        "description": "Description",
        "completed": false,
        "createdAt": "2025-07-20T12:00:00.000Z",
        "updatedAt": "2025-07-20T12:00:00.000Z"
      }
    ],
    "pagination": {
      "total": 10,
      "limit": 20,
      "offset": 0,
      "hasNext": false,
      "hasPrev": false
    }
  }
  ```
- **Status Code**: 200

### ✓ Pagination Support
- **Requirement**: Supports limit and offset parameters
- **Test Cases**:
  ```bash
  # Default pagination
  curl "http://localhost:3000/api/tasks" -H "Authorization: Bearer $TOKEN"
  # Expected: limit=20, offset=0
  
  # Custom limit
  curl "http://localhost:3000/api/tasks?limit=5" -H "Authorization: Bearer $TOKEN"
  # Expected: Returns max 5 tasks
  
  # Offset for page 2
  curl "http://localhost:3000/api/tasks?limit=5&offset=5" -H "Authorization: Bearer $TOKEN"
  # Expected: Skips first 5 tasks
  ```

### ✓ Pagination Limits
- **Requirement**: Enforces reasonable limits
- **Test**:
  ```bash
  curl "http://localhost:3000/api/tasks?limit=1000" -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: limit capped at 100

### ✓ Completed Filter
- **Requirement**: Can filter by completion status
- **Test Cases**:
  ```bash
  # Completed tasks only
  curl "http://localhost:3000/api/tasks?completed=true" -H "Authorization: Bearer $TOKEN"
  
  # Incomplete tasks only
  curl "http://localhost:3000/api/tasks?completed=false" -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: Only matching tasks returned

### ✓ Pagination Metadata
- **Requirement**: Includes accurate pagination info
- **Verification**:
  - total: Total count of user's tasks
  - limit: Current page size
  - offset: Current offset
  - hasNext: true if more results exist
  - hasPrev: true if offset > 0

## POST /api/tasks Criteria

### ✓ Create New Task
- **Requirement**: Creates task for authenticated user
- **Test**:
  ```bash
  curl -X POST http://localhost:3000/api/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"title":"New Task","description":"Task description"}'
  ```
- **Expected Response**:
  ```json
  {
    "id": 1,
    "title": "New Task",
    "description": "Task description",
    "completed": false,
    "createdAt": "2025-07-20T12:00:00.000Z",
    "updatedAt": "2025-07-20T12:00:00.000Z"
  }
  ```
- **Status Code**: 201

### ✓ Title Required
- **Requirement**: Title is mandatory
- **Test**:
  ```bash
  curl -X POST http://localhost:3000/api/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"description":"No title"}'
  ```
- **Expected**: 400 "Title is required"

### ✓ Title Length Validation
- **Requirement**: Title max 255 characters
- **Test**: Send title with 256+ characters
- **Expected**: 400 "Title must be 255 characters or less"

### ✓ Description Optional
- **Requirement**: Description can be null
- **Test**:
  ```bash
  curl -X POST http://localhost:3000/api/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"title":"Task without description"}'
  ```
- **Expected**: Success with description: null

### ✓ Description Length Validation
- **Requirement**: Description max 1000 characters
- **Test**: Send description with 1001+ characters
- **Expected**: 400 "Description must be 1000 characters or less"

### ✓ Default Completed Status
- **Requirement**: New tasks are incomplete by default
- **Verification**: Check created task
- **Expected**: completed: false

### ✓ Input Trimming
- **Requirement**: Title and description are trimmed
- **Test**: Send "  Spaced Title  "
- **Expected**: Saved as "Spaced Title"

## GET /api/tasks/:id Criteria

### ✓ Get Own Task
- **Requirement**: Can retrieve own tasks
- **Test**:
  ```bash
  curl http://localhost:3000/api/tasks/1 \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: 200 with task details

### ✓ Task Not Found
- **Requirement**: Non-existent tasks return 404
- **Test**:
  ```bash
  curl http://localhost:3000/api/tasks/99999 \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: 404 "Task not found"

### ✓ Access Control
- **Requirement**: Cannot access other users' tasks
- **Test**:
  1. Create task as User A
  2. Try to access as User B
- **Expected**: 403 "Access denied"

### ✓ Invalid ID Format
- **Requirement**: Non-numeric IDs rejected
- **Test**:
  ```bash
  curl http://localhost:3000/api/tasks/abc \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: 400 "Invalid task ID"

## PUT /api/tasks/:id Criteria

### ✓ Update Own Task
- **Requirement**: Can update own tasks
- **Test**:
  ```bash
  curl -X PUT http://localhost:3000/api/tasks/1 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"title":"Updated Title"}'
  ```
- **Expected**: 200 with updated task

### ✓ Partial Updates
- **Requirement**: Can update individual fields
- **Test Cases**:
  ```bash
  # Update title only
  -d '{"title":"New Title"}'
  
  # Update description only
  -d '{"description":"New Description"}'
  
  # Update completed only
  -d '{"completed":true}'
  ```
- **Expected**: Only specified fields change

### ✓ Title Validation on Update
- **Requirement**: Title cannot be empty
- **Test**:
  ```bash
  curl -X PUT http://localhost:3000/api/tasks/1 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"title":""}'
  ```
- **Expected**: 400 "Title cannot be empty"

### ✓ Update Timestamp
- **Requirement**: updated_at changes on update
- **Test**:
  1. Note original updated_at
  2. Update task
  3. Check new updated_at
- **Expected**: Timestamp is newer

### ✓ No Updates Validation
- **Requirement**: Empty updates rejected
- **Test**:
  ```bash
  curl -X PUT http://localhost:3000/api/tasks/1 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{}'
  ```
- **Expected**: 400 "No valid fields to update"

### ✓ Ownership Check
- **Requirement**: Cannot update other users' tasks
- **Test**: Update task as different user
- **Expected**: 404 "Task not found or access denied"

## DELETE /api/tasks/:id Criteria

### ✓ Delete Own Task
- **Requirement**: Can delete own tasks
- **Test**:
  ```bash
  curl -X DELETE http://localhost:3000/api/tasks/1 \
    -H "Authorization: Bearer $TOKEN"
  ```
- **Expected**: 204 No Content

### ✓ Task Actually Deleted
- **Requirement**: Task is removed from database
- **Test**: Try to GET deleted task
- **Expected**: 404 Not Found

### ✓ Ownership Check
- **Requirement**: Cannot delete other users' tasks
- **Test**: Delete task as different user
- **Expected**: 404 "Task not found or access denied"

### ✓ Idempotent Delete
- **Requirement**: Deleting twice doesn't error
- **Test**: Delete same task twice
- **Expected**: Both return 404 (second is already gone)

## Cross-User Isolation Criteria

### ✓ Complete User Isolation
- **Requirement**: Users cannot interact with others' tasks
- **Test Scenario**:
  1. User A creates tasks A1, A2
  2. User B creates tasks B1, B2
  3. Verify:
     - User A GET /api/tasks shows only A1, A2
     - User B GET /api/tasks shows only B1, B2
     - User A cannot GET/PUT/DELETE B1
     - User B cannot GET/PUT/DELETE A1

## Error Handling Criteria

### ✓ Consistent Error Format
- **Requirement**: All errors follow same structure
- **Expected Format**:
  ```json
  {
    "error": {
      "message": "Human-readable message",
      "code": "ERROR_CODE",
      "field": "field_name" // optional
    }
  }
  ```

### ✓ Appropriate Status Codes
- **Requirement**: Correct HTTP status codes
- **Expected**:
  - 200: Successful GET/PUT
  - 201: Successful POST
  - 204: Successful DELETE
  - 400: Bad Request (validation)
  - 401: Unauthorized (no token)
  - 403: Forbidden (wrong user)
  - 404: Not Found
  - 500: Server Error

## Performance Criteria

### ✓ Efficient Queries
- **Requirement**: Database queries are optimized
- **Verification**: Check query logs
- **Expected**: Uses indexes, no N+1 queries

### ✓ Response Times
- **Requirement**: Reasonable response times
- **Expected**: < 100ms for most operations

## Test Summary Checklist

- [ ] Task routes file created and integrated
- [ ] All endpoints require authentication
- [ ] GET /api/tasks returns user's tasks only
- [ ] Pagination works with limit/offset
- [ ] Completed filter works correctly
- [ ] POST /api/tasks creates new tasks
- [ ] Title validation enforced
- [ ] Description optional but length-limited
- [ ] GET /api/tasks/:id returns specific task
- [ ] Access control prevents cross-user access
- [ ] PUT /api/tasks/:id updates tasks
- [ ] Partial updates supported
- [ ] updated_at timestamp changes
- [ ] DELETE /api/tasks/:id removes tasks
- [ ] Complete user isolation maintained
- [ ] Consistent error response format
- [ ] Appropriate HTTP status codes
- [ ] Input trimming works
- [ ] Invalid IDs handled gracefully

## Definition of Done

Task 4 is complete when:
1. All CRUD endpoints are implemented
2. Authentication is required on all routes
3. Authorization ensures user data isolation
4. Pagination and filtering work correctly
5. Input validation prevents bad data
6. Error responses are consistent
7. All acceptance criteria are met
8. Integration with auth system is seamless

## Notes

- Test with multiple users to verify isolation
- Check edge cases (empty strings, large numbers)
- Verify pagination metadata accuracy
- Ensure no SQL injection vulnerabilities
- Test with invalid/expired tokens