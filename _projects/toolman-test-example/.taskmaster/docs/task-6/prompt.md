# Autonomous Agent Prompt: Frontend Authentication and User Profile

You are tasked with implementing a complete authentication system in a React frontend application, including user registration, login, profile management, and secure token handling.

## Objective
Build a robust authentication system with React Context API for state management, JWT token handling with automatic refresh, protected routes, and user profile functionality.

## Detailed Requirements

### 1. Authentication Context
Create a global authentication context that:
- Stores current user information
- Manages authentication state (user, isAuthenticated, isLoading)
- Provides methods: login, register, logout, updateProfile
- Handles token initialization on app load
- Implements automatic token refresh

Context should expose:
```typescript
interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => void;
  updateProfile: (data: UpdateProfileData) => Promise<void>;
}
```

### 2. API Service Configuration
Set up Axios with:
- Base URL from environment variables
- Request interceptor to add Bearer token
- Response interceptor for 401 handling
- Automatic token refresh on 401
- Queue requests during token refresh
- Redirect to login on refresh failure

### 3. Token Management
Implement secure token handling:
- Store access token in memory/localStorage
- Store refresh token securely
- Refresh access token before expiry
- Clear tokens on logout
- Handle concurrent requests during refresh

### 4. Login Component
Create login form with:
- Email and password fields
- Client-side validation
- Error display
- Loading states
- Link to registration
- Link to password reset
- Remember me option (optional)

### 5. Registration Component
Build registration form with:
- Email, username, password fields
- Password confirmation
- Input validation:
  - Email format
  - Username (3-20 chars, alphanumeric)
  - Password strength (8+ chars, mixed case, number)
- Duplicate email/username handling
- Success redirect to chat

### 6. Protected Routes
Implement route protection:
- ProtectedRoute wrapper component
- Check authentication status
- Show loading during auth check
- Redirect to login if not authenticated
- Preserve intended destination
- Handle deep linking

### 7. User Profile
Create profile management:
- Display user information
- Edit mode for updates
- Username change validation
- Avatar URL update
- Success/error feedback
- Real-time update reflection

### 8. Password Reset Flow
Implement password reset:
- Forgot password form
- Email validation
- Success message display
- Reset password form with token
- New password validation
- Confirmation field
- Auto-login after reset

### 9. Form Validation
Consistent validation across forms:
- Real-time field validation
- Clear error messages
- Disable submit during processing
- Show field-level errors
- Handle server errors gracefully

### 10. UI/UX Considerations
- Responsive design
- Dark mode support
- Loading indicators
- Smooth transitions
- Accessible forms
- Clear CTAs

## Expected Deliverables

1. AuthContext provider and hook
2. API service with interceptors
3. Login component
4. Registration component
5. Protected route wrapper
6. User profile component
7. Password reset components
8. Validation utilities
9. Type definitions
10. App router setup

## Implementation Standards

### File Structure
```
frontend/src/
├── components/
│   ├── auth/
│   │   ├── LoginForm.tsx
│   │   ├── RegisterForm.tsx
│   │   ├── ProtectedRoute.tsx
│   │   ├── ForgotPassword.tsx
│   │   └── ResetPassword.tsx
│   └── profile/
│       └── UserProfile.tsx
├── contexts/
│   └── AuthContext.tsx
├── services/
│   └── api.ts
├── utils/
│   └── validators.ts
└── types/
    └── user.ts
```

### State Management
- Use React Context for auth state
- Local state for form data
- Loading states for async operations
- Error handling at component level

### Security Best Practices
- Never log sensitive data
- Clear tokens on logout
- Validate all inputs
- Use HTTPS in production
- Implement CSRF protection
- Set secure cookie flags

## Testing Requirements

Write tests for:
1. Authentication context behavior
2. Login form submission
3. Registration validation
4. Protected route access
5. Token refresh flow
6. Profile updates
7. Password reset process

## Styling Guidelines

Use Tailwind CSS classes:
- Consistent spacing and sizing
- Proper color schemes
- Focus states for accessibility
- Error state styling
- Loading state animations
- Responsive breakpoints

## Error Handling

Handle these scenarios:
- Network errors
- Invalid credentials
- Expired tokens
- Server errors
- Validation errors
- Rate limiting

## Verification Steps

1. Test registration with new account
2. Login with valid credentials
3. Verify token in localStorage
4. Access protected route
5. Refresh page and stay logged in
6. Wait for token expiry and verify refresh
7. Update profile information
8. Complete password reset flow
9. Logout and verify redirect

Begin by creating the authentication context and API service configuration, then build the authentication components and integrate them with the routing system.