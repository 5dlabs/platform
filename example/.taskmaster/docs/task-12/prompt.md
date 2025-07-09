# Autonomous Prompt: Create User Type Definition

## Task Context
You are an AI assistant tasked with creating TypeScript type definitions for a User entity. This is a foundational task that establishes the data structure for user-related operations in an Express TypeScript application.

## Objective
Create a comprehensive User interface in TypeScript with proper type definitions, validation utilities, and related types for a complete user management system.

## Required Actions

### 1. Create the User Type File
Create `src/types/user.ts` with the following structure:

```typescript
// src/types/user.ts

/**
 * Core User interface representing a user in the system
 */
export interface User {
  id: string;
  name: string;
  email: string;
  createdAt: Date;
}

/**
 * Request payload for creating a new user
 */
export interface CreateUserRequest {
  name: string;
  email: string;
}

/**
 * Request payload for updating an existing user
 */
export interface UpdateUserRequest {
  name?: string;
  email?: string;
}

/**
 * Response format for user data in API responses
 */
export interface UserResponse {
  id: string;
  name: string;
  email: string;
  createdAt: string; // ISO string format
}
```

### 2. Add Validation Utilities
Extend the file with utility functions:

```typescript
/**
 * Validates if a string is a valid email address
 */
export function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

/**
 * Type guard to check if an object is a valid User
 */
export function isUser(obj: any): obj is User {
  return obj && 
    typeof obj.id === 'string' &&
    typeof obj.name === 'string' &&
    typeof obj.email === 'string' &&
    obj.createdAt instanceof Date;
}

/**
 * Validates if a CreateUserRequest is valid
 */
export function isValidCreateUserRequest(obj: any): obj is CreateUserRequest {
  return obj &&
    typeof obj.name === 'string' &&
    typeof obj.email === 'string' &&
    isValidEmail(obj.email) &&
    obj.name.trim().length > 0;
}

/**
 * Converts a User object to UserResponse format
 */
export function userToResponse(user: User): UserResponse {
  return {
    id: user.id,
    name: user.name,
    email: user.email,
    createdAt: user.createdAt.toISOString()
  };
}
```

### 3. Add Type Constants and Enums
Include any necessary constants:

```typescript
/**
 * User validation constraints
 */
export const USER_CONSTRAINTS = {
  NAME_MIN_LENGTH: 1,
  NAME_MAX_LENGTH: 100,
  EMAIL_MAX_LENGTH: 254,
  ID_LENGTH: 36 // UUID v4 length
} as const;

/**
 * User-related error messages
 */
export const USER_ERRORS = {
  INVALID_EMAIL: 'Invalid email address format',
  INVALID_NAME: 'Name must be between 1 and 100 characters',
  INVALID_ID: 'Invalid user ID format',
  USER_NOT_FOUND: 'User not found',
  DUPLICATE_EMAIL: 'Email address already exists'
} as const;
```

### 4. Create Index Export (Optional)
If creating a types index file, create `src/types/index.ts`:

```typescript
// src/types/index.ts
export * from './user';
```

## Validation Steps
1. **TypeScript Compilation**: Run `npx tsc --noEmit` to check for type errors
2. **Import Test**: Create a test file to verify imports work correctly
3. **Type Checking**: Test the type guards with sample data
4. **Validation Functions**: Test email validation with various inputs
5. **IDE Integration**: Verify intellisense and autocompletion work

## Testing the Implementation
Create a temporary test file `test-user-types.ts`:

```typescript
import { 
  User, 
  CreateUserRequest, 
  isValidEmail, 
  isUser, 
  isValidCreateUserRequest,
  userToResponse 
} from './src/types/user';

// Test User interface
const testUser: User = {
  id: '123e4567-e89b-12d3-a456-426614174000',
  name: 'John Doe',
  email: 'john@example.com',
  createdAt: new Date()
};

// Test validation functions
console.log('Valid email:', isValidEmail('test@example.com')); // true
console.log('Invalid email:', isValidEmail('invalid-email')); // false

// Test type guard
console.log('Is User:', isUser(testUser)); // true
console.log('Is User (invalid):', isUser({ id: 123 })); // false

// Test create request validation
const createRequest: CreateUserRequest = {
  name: 'Jane Doe',
  email: 'jane@example.com'
};
console.log('Valid create request:', isValidCreateUserRequest(createRequest)); // true

// Test response conversion
const response = userToResponse(testUser);
console.log('User response:', response);
```

## Success Criteria
- [ ] User interface is properly defined with all required fields
- [ ] All exports are accessible from other modules
- [ ] TypeScript compilation passes without errors
- [ ] Validation functions work correctly
- [ ] Type guards provide proper type narrowing
- [ ] Response conversion functions work as expected
- [ ] Code follows TypeScript best practices
- [ ] JSDoc comments are included for documentation

## Error Handling
- If TypeScript compilation fails, check interface syntax
- If imports don't work, verify export statements
- If validation fails, review function logic
- If type guards don't work, check return type annotations

## Tools Available
- TypeScript compiler for type checking
- File system operations for creating files
- Node.js runtime for testing validation functions

## Final Deliverables
- `src/types/user.ts` with complete User interface and utilities
- All validation functions implemented and tested
- Type guards for runtime type checking
- Response transformation utilities
- Constants for validation constraints
- Comprehensive JSDoc documentation

## Integration Notes
- This file will be imported by route handlers
- Validation functions will be used in middleware
- Type guards will help with runtime safety
- Response types will ensure API consistency
- Constants will be shared across the application