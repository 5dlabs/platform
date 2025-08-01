# Toolman Guide for Task 6: Frontend Authentication and User Profile

## Overview

This guide provides detailed instructions for using the selected tools to implement Task 6, which focuses on building a comprehensive authentication system in the React frontend, including user registration, login, profile management, and JWT token handling with automatic refresh.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create the frontend authentication directory structure

**When to Use**: 
- At the beginning to organize authentication components
- When creating contexts, hooks, and utility folders
- For organizing authentication-related pages

**How to Use**:
```
# Create authentication structure
create_directory /chat-application/frontend/src/contexts
create_directory /chat-application/frontend/src/hooks
create_directory /chat-application/frontend/src/components/auth
create_directory /chat-application/frontend/src/pages/auth
create_directory /chat-application/frontend/src/services
create_directory /chat-application/frontend/src/utils
create_directory /chat-application/frontend/src/components/profile
```

**Parameters**:
- `path`: Directory path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create authentication components, contexts, hooks, and services

**When to Use**: 
- To create AuthContext and provider
- To implement login and registration forms
- To create protected route components
- To implement token management utilities

**How to Use**:
```
# Create AuthContext
write_file /chat-application/frontend/src/contexts/AuthContext.tsx <context-content>

# Create login component
write_file /chat-application/frontend/src/pages/auth/Login.tsx <login-content>

# Create registration component
write_file /chat-application/frontend/src/pages/auth/Register.tsx <register-content>

# Create protected route component
write_file /chat-application/frontend/src/components/auth/ProtectedRoute.tsx <protected-route>

# Create auth service
write_file /chat-application/frontend/src/services/authService.ts <service-content>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review existing React setup and configurations

**When to Use**: 
- To check existing React app structure
- To review package.json for dependencies
- To understand current routing setup

**How to Use**:
```
# Read App component
read_file /chat-application/frontend/src/App.tsx

# Check package.json
read_file /chat-application/frontend/package.json

# Review existing components
read_file /chat-application/frontend/src/index.tsx
```

**Parameters**:
- `path`: File to read
- `head`/`tail`: Optional line limits

### 4. **edit_file** (Local - filesystem)
**Purpose**: Update existing files to integrate authentication

**When to Use**: 
- To add authentication dependencies
- To wrap app with AuthProvider
- To update routing with protected routes
- To modify environment variables

**How to Use**:
```
# Add authentication dependencies
edit_file /chat-application/frontend/package.json
# Add: axios, react-router-dom, react-hook-form, @types/react-router-dom

# Update App component
edit_file /chat-application/frontend/src/App.tsx
# Wrap with AuthProvider and add routes

# Add environment variables
edit_file /chat-application/frontend/.env.example
# Add API_URL and other config
```

**Parameters**:
- `old_string`: Exact text to replace
- `new_string`: New text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify authentication structure creation

**When to Use**: 
- After creating directory structure
- To confirm component organization
- Before testing implementation

**How to Use**:
```
# Verify contexts
list_directory /chat-application/frontend/src/contexts

# Check auth components
list_directory /chat-application/frontend/src/components/auth
```

**Parameters**:
- `path`: Directory to list

## Implementation Flow

1. **Directory Structure Phase**
   - Use `create_directory` to build authentication structure
   - Organize by contexts, hooks, components, services

2. **Authentication Context Phase**
   - Use `write_file` to create AuthContext.tsx
   - Implement JWT token storage (localStorage)
   - Add automatic token refresh logic
   - Manage user state globally

3. **Authentication Service Phase**
   - Use `write_file` to create authService.ts
   - Implement API calls for:
     - User registration
     - User login
     - Token refresh
     - User profile fetch
     - Password reset

4. **Component Implementation Phase**
   - Create Login.tsx with form validation
   - Create Register.tsx with password requirements
   - Create ProtectedRoute.tsx for route protection
   - Create UserProfile.tsx with edit functionality
   - Create PasswordReset.tsx flow

5. **Hook Creation Phase**
   - Use `write_file` to create custom hooks:
     - useAuth() - access auth context
     - useAuthGuard() - redirect logic
     - useTokenRefresh() - automatic refresh

6. **Integration Phase**
   - Use `edit_file` to update App.tsx
   - Wrap app with AuthProvider
   - Configure protected routes
   - Add authentication interceptors

## Best Practices

1. **Token Storage**: Use httpOnly cookies for production, localStorage for development
2. **Form Validation**: Implement client-side validation with react-hook-form
3. **Error Handling**: Display user-friendly error messages
4. **Loading States**: Show loading indicators during auth operations
5. **Automatic Logout**: Clear tokens on 401 responses
6. **Password Security**: Enforce strong password requirements

## Task-Specific Implementation Details

### AuthContext Pattern
```typescript
// AuthContext.tsx
import React, { createContext, useState, useEffect } from 'react';
import { authService } from '../services/authService';

interface AuthContextType {
  user: User | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  register: (data: RegisterData) => Promise<void>;
  isLoading: boolean;
}

export const AuthContext = createContext<AuthContextType | null>(null);

export const AuthProvider: React.FC = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Check for stored token on mount
    const token = localStorage.getItem('accessToken');
    if (token) {
      authService.getProfile()
        .then(setUser)
        .catch(() => localStorage.removeItem('accessToken'))
        .finally(() => setIsLoading(false));
    } else {
      setIsLoading(false);
    }
  }, []);

  // Implement auth methods...
};
```

### Protected Route Pattern
```typescript
// ProtectedRoute.tsx
import { Navigate } from 'react-router-dom';
import { useAuth } from '../../hooks/useAuth';

export const ProtectedRoute: React.FC<{ children: ReactNode }> = ({ children }) => {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return <LoadingSpinner />;
  }

  return user ? children : <Navigate to="/login" />;
};
```

### Form Validation Pattern
```typescript
// Login.tsx form example
import { useForm } from 'react-hook-form';

const { register, handleSubmit, formState: { errors } } = useForm();

<input
  {...register('email', {
    required: 'Email is required',
    pattern: {
      value: /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i,
      message: 'Invalid email address'
    }
  })}
/>
```

### Token Interceptor Pattern
```typescript
// axios interceptor setup
axios.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

axios.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // Try to refresh token
      try {
        await authService.refreshToken();
        return axios.request(error.config);
      } catch {
        // Redirect to login
        window.location.href = '/login';
      }
    }
    return Promise.reject(error);
  }
);
```

## Troubleshooting

- **CORS Issues**: Ensure backend allows frontend origin
- **Token Expiration**: Implement refresh before expiry
- **State Persistence**: Handle page refresh properly
- **Route Protection**: Test all protected routes
- **Form Errors**: Display validation errors clearly

## Testing Approach

1. **Component Tests**:
   - Test form validation
   - Test error states
   - Test loading states

2. **Integration Tests**:
   - Test complete auth flow
   - Test token refresh
   - Test protected route access

3. **E2E Tests**:
   - Test registration process
   - Test login/logout flow
   - Test profile updates