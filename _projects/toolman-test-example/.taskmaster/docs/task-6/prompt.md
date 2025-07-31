# Task 6: Frontend Authentication and User Profile - AI Agent Prompt

## Objective

You are tasked with implementing a comprehensive authentication system for a React frontend application. This includes user registration, login, profile management, and secure authentication state management using modern React patterns and TypeScript.

## Context

You are working on a React application that requires a complete authentication system. The backend API is already implemented with JWT-based authentication endpoints. Your task is to create the frontend components and logic to integrate with these endpoints and provide a seamless user experience.

## Requirements

### 1. Authentication Context and Provider

Create a global authentication context using React Context API that:
- Manages user state across the application
- Handles JWT token storage and automatic refresh
- Provides authentication methods (login, register, logout)
- Manages loading and error states
- Implements automatic token refresh using axios interceptors
- Persists authentication state across page refreshes

### 2. Authentication Forms

Implement the following forms with proper validation:

**Login Form:**
- Email and password fields
- Client-side validation using react-hook-form and yup
- Error handling with user-friendly messages
- Loading states during submission
- Link to registration and password reset

**Registration Form:**
- Username, email, password, and confirm password fields
- Strong password requirements (8+ chars, uppercase, lowercase, number)
- Username format validation (alphanumeric and underscore only)
- Real-time validation feedback
- Automatic login after successful registration

**Password Reset Forms:**
- Request form with email field
- Confirmation form with token and new password fields
- Success messages and error handling

### 3. Protected Routes

Create a ProtectedRoute component that:
- Checks authentication status before rendering protected content
- Shows loading state during authentication verification
- Redirects to login page for unauthenticated users
- Preserves the attempted URL for post-login redirect
- Handles edge cases like token expiration during navigation

### 4. User Profile Management

Implement a profile page with:
- Display of current user information
- Editable fields for username, email, and bio
- Avatar upload functionality with preview
- Form validation for profile updates
- Success/error feedback for all operations
- Toggle between view and edit modes

### 5. Security Implementation

Ensure security best practices:
- Secure token storage in localStorage
- Automatic token refresh before expiration
- Proper cleanup on logout
- Protection against XSS attacks
- Rate limiting awareness
- Secure password requirements

## Technical Stack

Use the following technologies:
- React 18+ with TypeScript
- React Router v6 for navigation
- React Hook Form for form management
- Yup for schema validation
- Axios for API communication
- Material-UI (MUI) for UI components
- React Context API for state management

## API Endpoints

The backend provides these endpoints:
- POST /api/auth/register - User registration
- POST /api/auth/login - User login
- POST /api/auth/refresh - Token refresh
- GET /api/auth/profile - Get user profile
- PATCH /api/auth/profile - Update user profile
- POST /api/auth/avatar - Upload avatar
- POST /api/auth/reset-password - Request password reset
- POST /api/auth/reset-password/confirm - Confirm password reset

## Implementation Guidelines

1. **Code Organization:**
   - Place auth components in `src/components/auth/`
   - Place profile components in `src/components/profile/`
   - Create context in `src/contexts/AuthContext.tsx`
   - Define types in `src/types/`
   - Configure API in `src/services/api.ts`

2. **Error Handling:**
   - Display user-friendly error messages
   - Clear errors when appropriate
   - Handle network errors gracefully
   - Implement retry logic for failed requests

3. **User Experience:**
   - Show loading indicators during async operations
   - Disable form inputs during submission
   - Provide clear feedback for all actions
   - Implement smooth transitions between states

4. **Testing Requirements:**
   - Write unit tests for context methods
   - Test form validation logic
   - Test protected route behavior
   - Mock API calls in tests

## Success Criteria

The implementation is complete when:
1. Users can register with validated information
2. Users can login and their session persists
3. Protected routes redirect unauthenticated users
4. Users can view and edit their profile
5. Tokens automatically refresh without user intervention
6. Password reset flow works end-to-end
7. All forms have proper validation and error handling
8. The UI is responsive and provides good feedback

## Additional Considerations

- Implement accessibility features (ARIA labels, keyboard navigation)
- Consider implementing social authentication in the future
- Plan for two-factor authentication support
- Design with mobile responsiveness in mind
- Follow React best practices and hooks guidelines
- Ensure TypeScript types are properly defined
- Document complex authentication flows

## Example Code Structure

```
src/
├── components/
│   ├── auth/
│   │   ├── LoginForm.tsx
│   │   ├── RegisterForm.tsx
│   │   ├── ForgotPassword.tsx
│   │   ├── ResetPassword.tsx
│   │   └── ProtectedRoute.tsx
│   └── profile/
│       ├── UserProfile.tsx
│       └── AvatarUpload.tsx
├── contexts/
│   └── AuthContext.tsx
├── services/
│   └── api.ts
├── types/
│   ├── user.ts
│   └── auth.ts
└── utils/
    └── auth.ts
```

Remember to handle edge cases, provide excellent user experience, and follow security best practices throughout the implementation.