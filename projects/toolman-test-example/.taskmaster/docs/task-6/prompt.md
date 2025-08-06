# Task 6: Frontend Authentication and User Profile - AI Agent Prompt

You are a senior frontend developer tasked with implementing a complete authentication system in React. Your implementation must provide secure user authentication, profile management, and seamless user experience with proper error handling and state management.

## Primary Objectives

1. **Authentication Context**: Create a global auth state management system using React Context API with full authentication lifecycle support.

2. **Authentication Forms**: Build login, registration, and password reset forms with comprehensive validation and error handling.

3. **Protected Routes**: Implement route protection to ensure only authenticated users can access certain pages.

4. **Token Management**: Handle JWT tokens securely with automatic refresh and proper storage strategies.

5. **User Profile**: Create profile viewing and editing functionality with avatar upload support.

## Required Actions

### Phase 1: Setup and Dependencies (10 minutes)
1. Install required packages:
   ```bash
   npm install react-hook-form @hookform/resolvers yup
   npm install axios react-router-dom
   npm install @mui/material @emotion/react @emotion/styled
   npm install -D @types/react @types/react-router-dom
   ```

2. Create folder structure:
   ```
   src/
   ├── contexts/
   ├── services/
   ├── components/
   │   ├── auth/
   │   └── profile/
   ├── hooks/
   └── types/
   ```

3. Set up TypeScript types:
   - User interface
   - Auth response types
   - Form data types

### Phase 2: Auth Context Implementation (20 minutes)
1. Create AuthContext with:
   - User state management
   - Authentication status
   - Loading states
   - Error handling

2. Implement auth methods:
   - login()
   - register()
   - logout()
   - updateProfile()
   - resetPassword()

3. Add token initialization:
   - Check for existing tokens
   - Validate on mount
   - Auto-refresh setup

### Phase 3: Authentication Service (15 minutes)
1. Create authService class:
   - API communication methods
   - Token management
   - Secure storage handling
   - Token refresh logic

2. Implement Axios interceptors:
   - Request interceptor for auth headers
   - Response interceptor for token refresh
   - Error handling for 401 responses

3. Token utilities:
   - Token validation
   - Expiry checking
   - Secure storage methods

### Phase 4: Form Components (20 minutes)
1. Login Form:
   - Email/password fields
   - Validation with Yup
   - Error display
   - Loading states
   - Remember me option

2. Registration Form:
   - All required fields
   - Password strength indicator
   - Confirm password
   - Terms acceptance
   - Success redirect

3. Password Reset:
   - Request reset form
   - Reset confirmation form
   - Token validation
   - Success feedback

### Phase 5: Protected Routes (10 minutes)
1. ProtectedRoute component:
   - Check authentication
   - Handle loading states
   - Redirect logic
   - Role-based access

2. Route configuration:
   - Public routes
   - Protected routes
   - Auth redirect logic
   - Deep linking support

### Phase 6: User Profile (15 minutes)
1. Profile view component:
   - Display user info
   - Avatar display
   - Edit mode toggle

2. Profile edit form:
   - Update user details
   - Avatar upload
   - Validation
   - Success feedback

### Phase 7: Testing & Polish (10 minutes)
1. Test all flows:
   - Login/logout
   - Registration
   - Token refresh
   - Profile updates

2. Polish UX:
   - Loading indicators
   - Error messages
   - Success feedback
   - Smooth transitions

## Implementation Requirements

### Authentication Flow
```typescript
// 1. User enters credentials
// 2. Send to backend API
// 3. Receive tokens and user data
// 4. Store tokens securely
// 5. Update auth context
// 6. Redirect to dashboard
```

### Token Storage Strategy
```typescript
// Development: localStorage
localStorage.setItem('accessToken', token);

// Production: Consider httpOnly cookies
// or in-memory storage with refresh token in httpOnly cookie
```

### Form Validation Rules
```typescript
const validationSchema = {
  email: 'valid email required',
  password: 'min 8 chars, upper, lower, number',
  username: '3-30 chars, alphanumeric + underscore',
};
```

### Protected Route Pattern
```typescript
<Route 
  path="/dashboard" 
  element={
    <ProtectedRoute>
      <Dashboard />
    </ProtectedRoute>
  } 
/>
```

## Security Requirements

### Token Security
- [ ] Never store sensitive data in localStorage in production
- [ ] Implement token rotation
- [ ] Clear tokens on logout
- [ ] Handle token expiry gracefully
- [ ] Use HTTPS in production

### XSS Prevention
- [ ] Sanitize all user inputs
- [ ] Use React's built-in protections
- [ ] Validate API responses
- [ ] Escape special characters

### Authentication Best Practices
- [ ] Rate limit login attempts
- [ ] Show generic error messages
- [ ] Implement CAPTCHA for repeated failures
- [ ] Log security events

## Error Handling

### User-Friendly Messages
```typescript
const errorMessages = {
  'INVALID_CREDENTIALS': 'Email or password is incorrect',
  'EMAIL_EXISTS': 'This email is already registered',
  'NETWORK_ERROR': 'Connection failed. Please try again',
  'TOKEN_EXPIRED': 'Session expired. Please login again',
};
```

### Network Error Handling
- Show offline indicator
- Queue failed requests
- Retry with exponential backoff
- Provide manual retry option

## State Management

### Auth Context State
```typescript
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}
```

### Loading States
- Initial auth check
- Form submissions
- Profile updates
- Token refresh

## UI/UX Requirements

### Form Design
- Clear labels and placeholders
- Inline validation feedback
- Password visibility toggle
- Loading button states
- Success animations

### Error Display
- Non-blocking alerts
- Field-specific errors
- Clear error messages
- Dismissible notifications

### Responsive Design
- Mobile-first approach
- Touch-friendly inputs
- Appropriate keyboard types
- Smooth transitions

## Testing Checklist

### Unit Tests
```typescript
describe('AuthContext', () => {
  test('login updates user state');
  test('logout clears user data');
  test('token refresh works');
  test('handles auth errors');
});

describe('LoginForm', () => {
  test('validates email format');
  test('requires password');
  test('shows loading state');
  test('displays errors');
});
```

### Integration Tests
- Complete login flow
- Registration process
- Password reset flow
- Token expiry handling
- Protected route access

## Performance Optimization

### Code Splitting
```typescript
const Login = lazy(() => import('./components/auth/Login'));
const Profile = lazy(() => import('./components/profile/Profile'));
```

### Memoization
- Memoize auth context value
- Use React.memo for forms
- Optimize re-renders
- Cache user data

## Accessibility

### Form Accessibility
- Proper labels
- ARIA attributes
- Keyboard navigation
- Screen reader support
- Focus management

### Error Announcement
- Use ARIA live regions
- Clear error associations
- Descriptive messages
- Status updates

## Documentation

### Component Documentation
```typescript
/**
 * ProtectedRoute - Wraps components that require authentication
 * @param {ReactNode} children - Components to protect
 * @param {string} requiredRole - Optional role requirement
 * @returns Protected component or redirect to login
 */
```

## Final Deliverables

Before marking complete:
- [ ] Auth context fully functional
- [ ] All forms working with validation
- [ ] Protected routes implemented
- [ ] Token refresh automatic
- [ ] Profile management complete
- [ ] Error handling comprehensive
- [ ] Loading states smooth
- [ ] Security best practices followed
- [ ] Responsive on all devices
- [ ] Tests passing

Execute this task systematically, ensuring each component is secure, user-friendly, and well-integrated with the backend authentication system.