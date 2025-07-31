# Task 6: Frontend Authentication and User Profile

## Overview

This task implements a comprehensive authentication system for the React frontend, including user registration, login, profile management, and authentication state management. The implementation uses React Context API for global state management, JWT tokens for authentication, and includes secure handling of user sessions.

## Technical Architecture

### Core Components

1. **Authentication Context Provider**
   - Global state management for user authentication
   - JWT token handling and automatic refresh
   - Persistent session management
   - Loading and error state management

2. **Authentication Forms**
   - Login form with email/password validation
   - Registration form with username, email, and password
   - Password reset request and confirmation forms
   - Real-time validation feedback

3. **Protected Routes**
   - HOC (Higher Order Component) for route protection
   - Automatic redirection for unauthorized access
   - Loading states during authentication checks

4. **User Profile Management**
   - Profile viewing and editing interface
   - Avatar upload functionality
   - Account settings management
   - Password change functionality

## Implementation Guide

### 1. Authentication Context Setup

```typescript
// src/contexts/AuthContext.tsx
import React, { createContext, useState, useEffect, useContext, useCallback } from 'react';
import { api, setAuthToken } from '../services/api';
import { User } from '../types/user';

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, username: string, password: string) => Promise<void>;
  logout: () => void;
  updateProfile: (data: Partial<User>) => Promise<void>;
  resetPassword: (email: string) => Promise<void>;
  confirmPasswordReset: (token: string, newPassword: string) => Promise<void>;
  clearError: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Token refresh logic
  const refreshAccessToken = useCallback(async () => {
    const refreshToken = localStorage.getItem('refreshToken');
    if (!refreshToken) throw new Error('No refresh token available');

    const response = await api.post('/api/auth/refresh', { refreshToken });
    const { accessToken } = response.data;
    
    localStorage.setItem('accessToken', accessToken);
    setAuthToken(accessToken);
    
    return accessToken;
  }, []);

  // Initialize authentication on mount
  useEffect(() => {
    const initAuth = async () => {
      const token = localStorage.getItem('accessToken');
      if (token) {
        setAuthToken(token);
        try {
          const response = await api.get('/api/auth/profile');
          setUser(response.data);
        } catch (error) {
          if (error.response?.status === 401) {
            try {
              await refreshAccessToken();
              const response = await api.get('/api/auth/profile');
              setUser(response.data);
            } catch (refreshError) {
              logout();
            }
          }
        }
      }
      setIsLoading(false);
    };

    initAuth();
  }, [refreshAccessToken]);

  // Set up axios interceptor for token refresh
  useEffect(() => {
    const interceptor = api.interceptors.response.use(
      (response) => response,
      async (error) => {
        const originalRequest = error.config;
        
        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;
          
          try {
            await refreshAccessToken();
            return api(originalRequest);
          } catch (refreshError) {
            logout();
            return Promise.reject(refreshError);
          }
        }
        
        return Promise.reject(error);
      }
    );

    return () => {
      api.interceptors.response.eject(interceptor);
    };
  }, [refreshAccessToken]);

  const login = async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await api.post('/api/auth/login', { email, password });
      const { accessToken, refreshToken, user } = response.data;
      
      localStorage.setItem('accessToken', accessToken);
      localStorage.setItem('refreshToken', refreshToken);
      setAuthToken(accessToken);
      setUser(user);
    } catch (error: any) {
      setError(error.response?.data?.message || 'Login failed');
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const register = async (email: string, username: string, password: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await api.post('/api/auth/register', { 
        email, 
        username, 
        password 
      });
      const { accessToken, refreshToken, user } = response.data;
      
      localStorage.setItem('accessToken', accessToken);
      localStorage.setItem('refreshToken', refreshToken);
      setAuthToken(accessToken);
      setUser(user);
    } catch (error: any) {
      setError(error.response?.data?.message || 'Registration failed');
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const logout = () => {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('refreshToken');
    setAuthToken(null);
    setUser(null);
    setError(null);
  };

  const updateProfile = async (data: Partial<User>) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await api.patch('/api/auth/profile', data);
      setUser(response.data);
    } catch (error: any) {
      setError(error.response?.data?.message || 'Profile update failed');
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const resetPassword = async (email: string) => {
    setError(null);
    
    try {
      await api.post('/api/auth/reset-password', { email });
    } catch (error: any) {
      setError(error.response?.data?.message || 'Password reset failed');
      throw error;
    }
  };

  const confirmPasswordReset = async (token: string, newPassword: string) => {
    setError(null);
    
    try {
      await api.post('/api/auth/reset-password/confirm', { token, newPassword });
    } catch (error: any) {
      setError(error.response?.data?.message || 'Password reset confirmation failed');
      throw error;
    }
  };

  const clearError = () => setError(null);

  return (
    <AuthContext.Provider value={{
      user,
      isAuthenticated: !!user,
      isLoading,
      error,
      login,
      register,
      logout,
      updateProfile,
      resetPassword,
      confirmPasswordReset,
      clearError
    }}>
      {children}
    </AuthContext.Provider>
  );
};
```

### 2. Login Component Implementation

```typescript
// src/components/auth/LoginForm.tsx
import React, { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useAuth } from '../../contexts/AuthContext';
import { Alert, Button, TextField, Box, Typography, CircularProgress } from '@mui/material';

const loginSchema = yup.object({
  email: yup.string().email('Invalid email').required('Email is required'),
  password: yup.string().required('Password is required').min(6, 'Password must be at least 6 characters'),
});

type LoginFormData = yup.InferType<typeof loginSchema>;

export const LoginForm: React.FC = () => {
  const navigate = useNavigate();
  const { login, error, clearError } = useAuth();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const { register, handleSubmit, formState: { errors } } = useForm<LoginFormData>({
    resolver: yupResolver(loginSchema),
  });

  const onSubmit = async (data: LoginFormData) => {
    setIsSubmitting(true);
    clearError();
    
    try {
      await login(data.email, data.password);
      navigate('/dashboard');
    } catch (error) {
      // Error is handled in context
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Box sx={{ maxWidth: 400, mx: 'auto', mt: 4 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        Login
      </Typography>
      
      {error && (
        <Alert severity="error" onClose={clearError} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <form onSubmit={handleSubmit(onSubmit)}>
        <TextField
          {...register('email')}
          label="Email"
          type="email"
          fullWidth
          margin="normal"
          error={!!errors.email}
          helperText={errors.email?.message}
          disabled={isSubmitting}
        />

        <TextField
          {...register('password')}
          label="Password"
          type="password"
          fullWidth
          margin="normal"
          error={!!errors.password}
          helperText={errors.password?.message}
          disabled={isSubmitting}
        />

        <Button
          type="submit"
          fullWidth
          variant="contained"
          sx={{ mt: 3, mb: 2 }}
          disabled={isSubmitting}
        >
          {isSubmitting ? <CircularProgress size={24} /> : 'Login'}
        </Button>

        <Box sx={{ textAlign: 'center' }}>
          <Link to="/auth/forgot-password">
            Forgot password?
          </Link>
        </Box>

        <Box sx={{ textAlign: 'center', mt: 2 }}>
          Don't have an account? <Link to="/auth/register">Register</Link>
        </Box>
      </form>
    </Box>
  );
};
```

### 3. Registration Component

```typescript
// src/components/auth/RegisterForm.tsx
import React, { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useAuth } from '../../contexts/AuthContext';
import { Alert, Button, TextField, Box, Typography, CircularProgress } from '@mui/material';

const registerSchema = yup.object({
  username: yup.string()
    .required('Username is required')
    .min(3, 'Username must be at least 3 characters')
    .matches(/^[a-zA-Z0-9_]+$/, 'Username can only contain letters, numbers, and underscores'),
  email: yup.string().email('Invalid email').required('Email is required'),
  password: yup.string()
    .required('Password is required')
    .min(8, 'Password must be at least 8 characters')
    .matches(
      /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/,
      'Password must contain at least one uppercase letter, one lowercase letter, and one number'
    ),
  confirmPassword: yup.string()
    .required('Please confirm your password')
    .oneOf([yup.ref('password')], 'Passwords must match'),
});

type RegisterFormData = yup.InferType<typeof registerSchema>;

export const RegisterForm: React.FC = () => {
  const navigate = useNavigate();
  const { register: registerUser, error, clearError } = useAuth();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const { register, handleSubmit, formState: { errors } } = useForm<RegisterFormData>({
    resolver: yupResolver(registerSchema),
  });

  const onSubmit = async (data: RegisterFormData) => {
    setIsSubmitting(true);
    clearError();
    
    try {
      await registerUser(data.email, data.username, data.password);
      navigate('/dashboard');
    } catch (error) {
      // Error is handled in context
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Box sx={{ maxWidth: 400, mx: 'auto', mt: 4 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        Register
      </Typography>
      
      {error && (
        <Alert severity="error" onClose={clearError} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <form onSubmit={handleSubmit(onSubmit)}>
        <TextField
          {...register('username')}
          label="Username"
          fullWidth
          margin="normal"
          error={!!errors.username}
          helperText={errors.username?.message}
          disabled={isSubmitting}
        />

        <TextField
          {...register('email')}
          label="Email"
          type="email"
          fullWidth
          margin="normal"
          error={!!errors.email}
          helperText={errors.email?.message}
          disabled={isSubmitting}
        />

        <TextField
          {...register('password')}
          label="Password"
          type="password"
          fullWidth
          margin="normal"
          error={!!errors.password}
          helperText={errors.password?.message}
          disabled={isSubmitting}
        />

        <TextField
          {...register('confirmPassword')}
          label="Confirm Password"
          type="password"
          fullWidth
          margin="normal"
          error={!!errors.confirmPassword}
          helperText={errors.confirmPassword?.message}
          disabled={isSubmitting}
        />

        <Button
          type="submit"
          fullWidth
          variant="contained"
          sx={{ mt: 3, mb: 2 }}
          disabled={isSubmitting}
        >
          {isSubmitting ? <CircularProgress size={24} /> : 'Register'}
        </Button>

        <Box sx={{ textAlign: 'center', mt: 2 }}>
          Already have an account? <Link to="/auth/login">Login</Link>
        </Box>
      </form>
    </Box>
  );
};
```

### 4. Protected Route Component

```typescript
// src/components/auth/ProtectedRoute.tsx
import React from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '../../contexts/AuthContext';
import { CircularProgress, Box } from '@mui/material';

interface ProtectedRouteProps {
  children: React.ReactNode;
  redirectTo?: string;
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ 
  children, 
  redirectTo = '/auth/login' 
}) => {
  const { isAuthenticated, isLoading } = useAuth();
  const location = useLocation();

  if (isLoading) {
    return (
      <Box sx={{ 
        display: 'flex', 
        justifyContent: 'center', 
        alignItems: 'center', 
        height: '100vh' 
      }}>
        <CircularProgress />
      </Box>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to={redirectTo} state={{ from: location }} replace />;
  }

  return <>{children}</>;
};
```

### 5. User Profile Component

```typescript
// src/components/profile/UserProfile.tsx
import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useAuth } from '../../contexts/AuthContext';
import {
  Alert,
  Avatar,
  Box,
  Button,
  Card,
  CardContent,
  CircularProgress,
  TextField,
  Typography,
  IconButton,
  Input,
} from '@mui/material';
import { PhotoCamera } from '@mui/icons-material';

const profileSchema = yup.object({
  username: yup.string()
    .required('Username is required')
    .min(3, 'Username must be at least 3 characters'),
  email: yup.string().email('Invalid email').required('Email is required'),
  bio: yup.string().max(500, 'Bio must be less than 500 characters'),
});

type ProfileFormData = yup.InferType<typeof profileSchema>;

export const UserProfile: React.FC = () => {
  const { user, updateProfile, error, clearError } = useAuth();
  const [isEditing, setIsEditing] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [uploadingAvatar, setUploadingAvatar] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const { register, handleSubmit, formState: { errors }, reset } = useForm<ProfileFormData>({
    resolver: yupResolver(profileSchema),
    defaultValues: {
      username: user?.username || '',
      email: user?.email || '',
      bio: user?.bio || '',
    },
  });

  const onSubmit = async (data: ProfileFormData) => {
    setIsSubmitting(true);
    clearError();
    setSuccessMessage(null);
    
    try {
      await updateProfile(data);
      setSuccessMessage('Profile updated successfully');
      setIsEditing(false);
    } catch (error) {
      // Error is handled in context
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleAvatarUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setUploadingAvatar(true);
    clearError();
    
    try {
      const formData = new FormData();
      formData.append('avatar', file);
      
      const response = await fetch('/api/auth/avatar', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('accessToken')}`,
        },
        body: formData,
      });

      if (!response.ok) throw new Error('Avatar upload failed');
      
      const data = await response.json();
      await updateProfile({ avatarUrl: data.avatarUrl });
      setSuccessMessage('Avatar updated successfully');
    } catch (error) {
      // Handle error
    } finally {
      setUploadingAvatar(false);
    }
  };

  const handleCancel = () => {
    reset();
    setIsEditing(false);
    clearError();
  };

  if (!user) return null;

  return (
    <Box sx={{ maxWidth: 600, mx: 'auto', mt: 4 }}>
      <Card>
        <CardContent>
          <Box sx={{ display: 'flex', alignItems: 'center', mb: 3 }}>
            <Box sx={{ position: 'relative' }}>
              <Avatar
                src={user.avatarUrl}
                sx={{ width: 100, height: 100 }}
              >
                {user.username.charAt(0).toUpperCase()}
              </Avatar>
              {isEditing && (
                <IconButton
                  color="primary"
                  aria-label="upload picture"
                  component="label"
                  sx={{
                    position: 'absolute',
                    bottom: 0,
                    right: 0,
                    backgroundColor: 'background.paper',
                  }}
                  disabled={uploadingAvatar}
                >
                  <Input
                    type="file"
                    sx={{ display: 'none' }}
                    onChange={handleAvatarUpload}
                    inputProps={{ accept: 'image/*' }}
                  />
                  {uploadingAvatar ? <CircularProgress size={20} /> : <PhotoCamera />}
                </IconButton>
              )}
            </Box>
            <Box sx={{ ml: 3 }}>
              <Typography variant="h5">{user.username}</Typography>
              <Typography variant="body2" color="text.secondary">
                Member since {new Date(user.createdAt).toLocaleDateString()}
              </Typography>
            </Box>
          </Box>

          {error && (
            <Alert severity="error" onClose={clearError} sx={{ mb: 2 }}>
              {error}
            </Alert>
          )}

          {successMessage && (
            <Alert severity="success" onClose={() => setSuccessMessage(null)} sx={{ mb: 2 }}>
              {successMessage}
            </Alert>
          )}

          <form onSubmit={handleSubmit(onSubmit)}>
            <TextField
              {...register('username')}
              label="Username"
              fullWidth
              margin="normal"
              error={!!errors.username}
              helperText={errors.username?.message}
              disabled={!isEditing || isSubmitting}
            />

            <TextField
              {...register('email')}
              label="Email"
              type="email"
              fullWidth
              margin="normal"
              error={!!errors.email}
              helperText={errors.email?.message}
              disabled={!isEditing || isSubmitting}
            />

            <TextField
              {...register('bio')}
              label="Bio"
              multiline
              rows={4}
              fullWidth
              margin="normal"
              error={!!errors.bio}
              helperText={errors.bio?.message}
              disabled={!isEditing || isSubmitting}
            />

            <Box sx={{ mt: 3, display: 'flex', gap: 2 }}>
              {isEditing ? (
                <>
                  <Button
                    type="submit"
                    variant="contained"
                    disabled={isSubmitting}
                  >
                    {isSubmitting ? <CircularProgress size={24} /> : 'Save Changes'}
                  </Button>
                  <Button
                    variant="outlined"
                    onClick={handleCancel}
                    disabled={isSubmitting}
                  >
                    Cancel
                  </Button>
                </>
              ) : (
                <Button
                  variant="contained"
                  onClick={() => setIsEditing(true)}
                >
                  Edit Profile
                </Button>
              )}
            </Box>
          </form>
        </CardContent>
      </Card>
    </Box>
  );
};
```

### 6. Password Reset Flow

```typescript
// src/components/auth/ForgotPassword.tsx
import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useAuth } from '../../contexts/AuthContext';
import { Alert, Button, TextField, Box, Typography, CircularProgress } from '@mui/material';

const schema = yup.object({
  email: yup.string().email('Invalid email').required('Email is required'),
});

type FormData = yup.InferType<typeof schema>;

export const ForgotPassword: React.FC = () => {
  const { resetPassword, error, clearError } = useAuth();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);

  const { register, handleSubmit, formState: { errors } } = useForm<FormData>({
    resolver: yupResolver(schema),
  });

  const onSubmit = async (data: FormData) => {
    setIsSubmitting(true);
    clearError();
    setSuccess(false);
    
    try {
      await resetPassword(data.email);
      setSuccess(true);
    } catch (error) {
      // Error is handled in context
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Box sx={{ maxWidth: 400, mx: 'auto', mt: 4 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        Reset Password
      </Typography>

      <Typography variant="body1" sx={{ mb: 3 }}>
        Enter your email address and we'll send you a link to reset your password.
      </Typography>
      
      {error && (
        <Alert severity="error" onClose={clearError} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert severity="success" sx={{ mb: 2 }}>
          Password reset link sent! Check your email.
        </Alert>
      )}

      <form onSubmit={handleSubmit(onSubmit)}>
        <TextField
          {...register('email')}
          label="Email"
          type="email"
          fullWidth
          margin="normal"
          error={!!errors.email}
          helperText={errors.email?.message}
          disabled={isSubmitting || success}
        />

        <Button
          type="submit"
          fullWidth
          variant="contained"
          sx={{ mt: 3, mb: 2 }}
          disabled={isSubmitting || success}
        >
          {isSubmitting ? <CircularProgress size={24} /> : 'Send Reset Link'}
        </Button>

        <Box sx={{ textAlign: 'center' }}>
          <Link to="/auth/login">
            Back to Login
          </Link>
        </Box>
      </form>
    </Box>
  );
};
```

## API Service Configuration

```typescript
// src/services/api.ts
import axios from 'axios';

export const api = axios.create({
  baseURL: process.env.REACT_APP_API_URL || 'http://localhost:3000',
  headers: {
    'Content-Type': 'application/json',
  },
});

export const setAuthToken = (token: string | null) => {
  if (token) {
    api.defaults.headers.common['Authorization'] = `Bearer ${token}`;
  } else {
    delete api.defaults.headers.common['Authorization'];
  }
};
```

## Type Definitions

```typescript
// src/types/user.ts
export interface User {
  id: string;
  username: string;
  email: string;
  avatarUrl?: string;
  bio?: string;
  createdAt: string;
  updatedAt: string;
}

// src/types/auth.ts
export interface LoginResponse {
  accessToken: string;
  refreshToken: string;
  user: User;
}

export interface RegisterResponse {
  accessToken: string;
  refreshToken: string;
  user: User;
}
```

## Security Considerations

1. **Token Storage**
   - Store tokens in localStorage with HttpOnly cookie fallback
   - Implement automatic token refresh before expiration
   - Clear tokens on logout and authentication errors

2. **Form Validation**
   - Client-side validation with yup schemas
   - Server-side validation for all inputs
   - Rate limiting on authentication endpoints

3. **Protected Routes**
   - Verify authentication before rendering protected components
   - Redirect to login with return URL preservation
   - Show loading states during authentication checks

4. **Error Handling**
   - Graceful error messages for users
   - Automatic retry with token refresh
   - Clear error states on component unmount

## Testing Requirements

1. **Unit Tests**
   - Test authentication context methods
   - Test form validation logic
   - Test protected route behavior
   - Test token refresh mechanism

2. **Integration Tests**
   - Test complete login flow
   - Test registration with validation
   - Test profile update functionality
   - Test password reset flow

3. **E2E Tests**
   - Test authentication persistence
   - Test protected route navigation
   - Test form submission and error handling

## Dependencies

```json
{
  "react": "^18.2.0",
  "react-router-dom": "^6.8.0",
  "react-hook-form": "^7.43.0",
  "@hookform/resolvers": "^2.9.0",
  "yup": "^1.0.0",
  "axios": "^1.3.0",
  "@mui/material": "^5.11.0",
  "@emotion/react": "^11.10.0",
  "@emotion/styled": "^11.10.0"
}
```

## Next Steps

1. Implement social authentication (OAuth)
2. Add two-factor authentication
3. Implement remember me functionality
4. Add session management UI
5. Implement account deletion flow