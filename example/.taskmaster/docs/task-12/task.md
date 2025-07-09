# Task 12: Create User Type Definition

## Overview
Define the User interface in TypeScript with id, name, email, and createdAt fields to establish a consistent data structure for user-related operations.

## Description
This task involves creating a TypeScript interface for the User entity, which will serve as the foundation for type-safe user operations throughout the application. The User type will include essential fields for user identification, contact information, and audit tracking.

## Priority
High

## Dependencies
- Task 11: Initialize Express TypeScript Project (must be completed first)

## Implementation Steps

### 1. Create user.ts file
- Create `src/types/user.ts` file with proper TypeScript structure
- Set up proper module exports for the User interface
- Include necessary TypeScript imports if required

### 2. Define User interface
- Define the User interface with the following properties:
  - `id`: Unique identifier (string or number)
  - `name`: User's full name (string)
  - `email`: User's email address (string)
  - `createdAt`: Timestamp of user creation (Date)
- Apply appropriate TypeScript types and constraints
- Add optional fields if needed for extensibility

## Implementation Details

### User Interface Structure
```typescript
export interface User {
  id: string;
  name: string;
  email: string;
  createdAt: Date;
}
```

### Extended User Interface (Optional)
```typescript
export interface CreateUserRequest {
  name: string;
  email: string;
}

export interface UpdateUserRequest {
  name?: string;
  email?: string;
}

export interface UserResponse {
  id: string;
  name: string;
  email: string;
  createdAt: string; // ISO string format for API responses
}
```

### Type Guards and Utilities
```typescript
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function isUser(obj: any): obj is User {
  return obj && 
    typeof obj.id === 'string' &&
    typeof obj.name === 'string' &&
    typeof obj.email === 'string' &&
    obj.createdAt instanceof Date;
}
```

## File Structure
```
src/
└── types/
    └── user.ts
```

## Test Strategy
- TypeScript compilation should pass without type errors
- Import User interface in other modules to verify export
- Test type checking with valid and invalid user objects
- Verify intellisense and autocompletion work correctly

## Expected Outcomes
- Well-defined User interface available for import
- Type safety for user-related operations
- Consistent data structure across the application
- Foundation for user CRUD operations

## Common Issues
- **Import/Export**: Ensure proper module exports are used
- **Type naming**: Use consistent naming conventions
- **Date handling**: Consider timezone and serialization issues
- **Validation**: Separate type definition from runtime validation

## Next Steps
After completion, this User type will be used in:
- User route handlers (GET /users, POST /users)
- Database operations (if implemented)
- Request/response validation
- Other user-related functionality

## Integration Points
- Will be imported in `src/routes/users.ts`
- May be used in middleware for validation
- Could be extended for authentication features
- Foundation for database schema if persistence is added