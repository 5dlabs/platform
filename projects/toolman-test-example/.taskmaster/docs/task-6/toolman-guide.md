# Task 6: Frontend Authentication and User Profile - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement a complete authentication system in React. The tools focus on file creation for React components and researching best practices for secure frontend authentication.

## Core Tools

### 1. brave_web_search
**Purpose**: Research React authentication patterns and security best practices
**When to use**: 
- Before implementing auth architecture
- When choosing token storage strategies
- For form validation patterns
- To find JWT handling best practices

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "React authentication context JWT best practices 2024",
  "freshness": "year"
}
```

**Key research topics**:
- "React Context API authentication pattern TypeScript"
- "JWT token storage React security best practices"
- "React Hook Form validation with Yup examples"
- "React Router v6 protected routes authentication"
- "Axios interceptors token refresh React"

### 2. create_directory
**Purpose**: Organize React authentication code structure
**When to use**:
- Setting up component directories
- Creating service folders
- Organizing context and hooks

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/frontend/src/contexts"
}
```

**Directory structure**:
```
/frontend/src/
├── contexts/
│   └── AuthContext.tsx
├── services/
│   ├── authService.ts
│   └── api.ts
├── components/
│   ├── auth/
│   │   ├── LoginForm.tsx
│   │   ├── RegisterForm.tsx
│   │   ├── ProtectedRoute.tsx
│   │   └── PasswordReset.tsx
│   └── profile/
│       └── UserProfile.tsx
├── hooks/
│   └── useAuth.ts
└── types/
    └── auth.ts
```

### 3. write_file
**Purpose**: Create React components and authentication logic
**When to use**:
- Writing component files
- Creating context providers
- Implementing services
- Setting up types

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/frontend/src/contexts/AuthContext.tsx",
  "content": "// Authentication context implementation"
}
```

### 4. edit_file
**Purpose**: Update existing React files with auth integration
**When to use**:
- Adding auth provider to App.tsx
- Updating route configuration
- Modifying API client setup
- Adding types to existing files

**How to use**:
```json
{
  "tool": "edit_file",
  "path": "/chat-application/frontend/src/App.tsx",
  "old_string": "function App() {",
  "new_string": "function App() {\n  return (\n    <AuthProvider>"
}
```

### 5. read_file
**Purpose**: Review existing code before modifications
**When to use**:
- Before updating App component
- To understand current routing setup
- To check existing API configuration
- Before modifying package.json

## Implementation Flow

### Phase 1: Research Best Practices (15 minutes)
1. **Auth patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "React authentication patterns JWT vs session 2024"
   }
   ```

2. **Security practices**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "React JWT token storage security localStorage vs memory"
   }
   ```

3. **Form validation**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "React Hook Form Yup validation authentication forms"
   }
   ```

### Phase 2: Create Auth Structure (20 minutes)
1. **Create directories**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/frontend/src/components/auth"
   }
   ```

2. **Write auth types**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/types/auth.ts",
     "content": "// User and auth response types"
   }
   ```

3. **Create auth context**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/contexts/AuthContext.tsx",
     "content": "// Complete auth context with state management"
   }
   ```

### Phase 3: Implement Auth Service (15 minutes)
1. **Auth service class**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/services/authService.ts",
     "content": "// API calls and token management"
   }
   ```

2. **Axios configuration**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/services/api.ts",
     "content": "// Axios instance with interceptors"
   }
   ```

### Phase 4: Create Components (25 minutes)
1. **Login form**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/components/auth/LoginForm.tsx",
     "content": "// Login form with validation"
   }
   ```

2. **Registration form**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/components/auth/RegisterForm.tsx",
     "content": "// Registration with password requirements"
   }
   ```

3. **Protected route**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/components/auth/ProtectedRoute.tsx",
     "content": "// Route protection component"
   }
   ```

### Phase 5: Integration (15 minutes)
1. **Update App.tsx**:
   ```json
   {
     "tool": "read_file",
     "path": "/chat-application/frontend/src/App.tsx"
   }
   ```
   Then:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/frontend/src/App.tsx",
     "old_string": "<Router>",
     "new_string": "<Router>\n    <AuthProvider>"
   }
   ```

2. **Update routing**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/frontend/src/App.tsx",
     "old_string": "<Route path=\"/dashboard\"",
     "new_string": "<Route path=\"/dashboard\" element={<ProtectedRoute><Dashboard /></ProtectedRoute>}"
   }
   ```

## Best Practices

### Token Storage Strategy
```typescript
// Development - Simple localStorage
const storeToken = (token: string) => {
  localStorage.setItem('accessToken', token);
};

// Production - More secure options
// Option 1: In-memory with refresh in httpOnly cookie
// Option 2: Encrypted localStorage
// Option 3: Service worker storage
```

### Form Validation Patterns
```typescript
// Consistent validation schema
const schema = yup.object({
  email: yup.string()
    .email('Invalid email format')
    .required('Email is required'),
  password: yup.string()
    .min(8, 'Password must be at least 8 characters')
    .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/, 
      'Must contain uppercase, lowercase and number')
    .required('Password is required')
});
```

### Error Handling Pattern
```typescript
// Centralized error handling
const handleAuthError = (error: any): string => {
  if (error.response?.status === 401) {
    return 'Invalid credentials';
  }
  if (error.response?.status === 409) {
    return 'Email already exists';
  }
  return 'An error occurred. Please try again.';
};
```

## Common Patterns

### Research → Design → Implement
```javascript
// 1. Research token strategies
const strategies = await brave_web_search("JWT storage strategies React 2024");

// 2. Design based on findings
const authDesign = createAuthArchitecture(strategies);

// 3. Implement
await write_file("contexts/AuthContext.tsx", authDesign);
```

### Component Creation Flow
```javascript
// 1. Create component file
await write_file("components/auth/LoginForm.tsx", componentCode);

// 2. Add to routing
await edit_file("App.tsx", 
  "<Routes>",
  "<Routes>\n  <Route path=\"/login\" element={<LoginForm />} />"
);

// 3. Test integration
await write_file("components/auth/LoginForm.test.tsx", testCode);
```

## Security Patterns

### XSS Prevention
```typescript
// Always sanitize user input
import DOMPurify from 'dompurify';

const sanitizedInput = DOMPurify.sanitize(userInput);

// React automatically escapes values
<div>{userGeneratedContent}</div> // Safe

// Avoid dangerouslySetInnerHTML
```

### CSRF Protection
```typescript
// Include CSRF token in requests
const csrfToken = document.querySelector('meta[name="csrf-token"]')?.content;

api.defaults.headers.common['X-CSRF-Token'] = csrfToken;
```

## React-Specific Patterns

### Custom Hook Pattern
```typescript
// Create reusable auth hook
export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
};

// Usage in components
const { user, login, logout } = useAuth();
```

### Protected Route Pattern
```typescript
// Wrapper component for protection
<ProtectedRoute requiredRole="admin">
  <AdminDashboard />
</ProtectedRoute>

// Conditional rendering
{isAuthenticated ? <AuthenticatedApp /> : <PublicApp />}
```

## Troubleshooting

### Issue: Token not persisting on refresh
**Solution**: Check token storage implementation, verify localStorage/cookie settings

### Issue: Infinite redirect loops
**Solution**: Check ProtectedRoute logic, ensure auth state updates correctly

### Issue: Form validation not showing
**Solution**: Verify React Hook Form setup, check error object destructuring

### Issue: API calls missing auth header
**Solution**: Verify Axios interceptor setup, check token retrieval

## Performance Optimization

### Code Splitting
```typescript
// Lazy load auth components
const LoginForm = lazy(() => import('./components/auth/LoginForm'));
const UserProfile = lazy(() => import('./components/profile/UserProfile'));

// Wrap with Suspense
<Suspense fallback={<Loading />}>
  <LoginForm />
</Suspense>
```

### Context Optimization
```typescript
// Memoize context value
const value = useMemo(() => ({
  user,
  login,
  logout,
  // ... other values
}), [user]); // Only recreate when user changes

// Split contexts if needed
<AuthContext.Provider>
  <UserContext.Provider>
    <App />
  </UserContext.Provider>
</AuthContext.Provider>
```

## Task Completion Checklist
- [ ] Auth context created and working
- [ ] Login form with validation
- [ ] Registration form complete
- [ ] Protected routes functional
- [ ] Token management implemented
- [ ] Auto-refresh working
- [ ] Profile page created
- [ ] Error handling comprehensive
- [ ] Loading states smooth
- [ ] Security best practices followed

This systematic approach ensures a secure, user-friendly authentication system that integrates seamlessly with the React application.