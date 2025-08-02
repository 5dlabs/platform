# Task 6: Frontend Authentication and User Profile

## Overview
Implement a complete authentication system in the React frontend, including user registration, login, profile management, and secure token handling. This task creates the client-side foundation for user authentication and authorization throughout the application.

## Technical Architecture

### Authentication Stack
- **State Management**: React Context API for auth state
- **Form Handling**: React Hook Form with validation
- **HTTP Client**: Axios with interceptors
- **Token Storage**: Secure localStorage with HttpOnly cookies fallback
- **Routing**: React Router with protected routes
- **UI Components**: Material-UI or custom components

### Security Features
- JWT token management with auto-refresh
- Secure token storage strategy
- XSS protection
- CSRF protection
- Input validation
- Error handling without information leakage

## Implementation Details

### 1. Authentication Context

```typescript
// frontend/src/contexts/AuthContext.tsx
import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { User, LoginCredentials, RegisterData } from '../types/auth';
import { authService } from '../services/authService';
import { api } from '../services/api';

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (credentials: LoginCredentials) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => void;
  updateProfile: (data: Partial<User>) => Promise<void>;
  resetPassword: (email: string) => Promise<void>;
  confirmReset: (token: string, password: string) => Promise<void>;
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

  // Initialize auth state on mount
  useEffect(() => {
    const initializeAuth = async () => {
      try {
        const accessToken = authService.getAccessToken();
        if (!accessToken) {
          setIsLoading(false);
          return;
        }

        // Validate token and get user profile
        const user = await authService.getCurrentUser();
        setUser(user);
        
        // Set up axios interceptor for token
        api.defaults.headers.common['Authorization'] = `Bearer ${accessToken}`;
      } catch (error) {
        console.error('Auth initialization error:', error);
        // Try to refresh token
        try {
          const newToken = await authService.refreshAccessToken();
          if (newToken) {
            const user = await authService.getCurrentUser();
            setUser(user);
          }
        } catch (refreshError) {
          // Clear invalid tokens
          authService.clearTokens();
        }
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();
  }, []);

  const login = useCallback(async (credentials: LoginCredentials) => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await authService.login(credentials);
      setUser(response.user);
      
      // Set authorization header for future requests
      api.defaults.headers.common['Authorization'] = `Bearer ${response.accessToken}`;
      
      // Redirect handled by component
    } catch (error: any) {
      setError(error.response?.data?.message || 'Login failed');
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const register = useCallback(async (data: RegisterData) => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await authService.register(data);
      setUser(response.user);
      
      api.defaults.headers.common['Authorization'] = `Bearer ${response.accessToken}`;
    } catch (error: any) {
      setError(error.response?.data?.message || 'Registration failed');
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const logout = useCallback(() => {
    authService.logout();
    setUser(null);
    delete api.defaults.headers.common['Authorization'];
    // Redirect to login handled by router
  }, []);

  const updateProfile = useCallback(async (data: Partial<User>) => {
    setIsLoading(true);
    setError(null);
    try {
      const updatedUser = await authService.updateProfile(data);
      setUser(updatedUser);
    } catch (error: any) {
      setError(error.response?.data?.message || 'Profile update failed');
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const resetPassword = useCallback(async (email: string) => {
    setError(null);
    try {
      await authService.requestPasswordReset(email);
    } catch (error: any) {
      setError(error.response?.data?.message || 'Password reset failed');
      throw error;
    }
  }, []);

  const confirmReset = useCallback(async (token: string, password: string) => {
    setError(null);
    try {
      await authService.confirmPasswordReset(token, password);
    } catch (error: any) {
      setError(error.response?.data?.message || 'Password reset failed');
      throw error;
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const value = {
    user,
    isAuthenticated: !!user,
    isLoading,
    error,
    login,
    register,
    logout,
    updateProfile,
    resetPassword,
    confirmReset,
    clearError,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
```

### 2. Authentication Service

```typescript
// frontend/src/services/authService.ts
import { api } from './api';
import { LoginCredentials, RegisterData, AuthResponse, User } from '../types/auth';

const TOKEN_KEY = 'accessToken';
const REFRESH_TOKEN_KEY = 'refreshToken';

class AuthService {
  private refreshTokenPromise: Promise<string> | null = null;

  async login(credentials: LoginCredentials): Promise<AuthResponse> {
    const response = await api.post<AuthResponse>('/api/auth/login', credentials);
    const { accessToken, refreshToken, user } = response.data;
    
    this.setTokens(accessToken, refreshToken);
    return response.data;
  }

  async register(data: RegisterData): Promise<AuthResponse> {
    const response = await api.post<AuthResponse>('/api/auth/register', data);
    const { accessToken, refreshToken, user } = response.data;
    
    this.setTokens(accessToken, refreshToken);
    return response.data;
  }

  async logout(): Promise<void> {
    try {
      await api.post('/api/auth/logout');
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      this.clearTokens();
    }
  }

  async getCurrentUser(): Promise<User> {
    const response = await api.get<User>('/api/auth/profile');
    return response.data;
  }

  async updateProfile(data: Partial<User>): Promise<User> {
    const response = await api.put<User>('/api/auth/profile', data);
    return response.data;
  }

  async requestPasswordReset(email: string): Promise<void> {
    await api.post('/api/auth/forgot-password', { email });
  }

  async confirmPasswordReset(token: string, password: string): Promise<void> {
    await api.post('/api/auth/reset-password', { token, password });
  }

  async refreshAccessToken(): Promise<string | null> {
    // Prevent multiple simultaneous refresh requests
    if (this.refreshTokenPromise) {
      return this.refreshTokenPromise;
    }

    const refreshToken = this.getRefreshToken();
    if (!refreshToken) {
      return null;
    }

    this.refreshTokenPromise = api
      .post<{ accessToken: string; refreshToken: string }>('/api/auth/refresh', {
        refreshToken,
      })
      .then((response) => {
        const { accessToken, refreshToken: newRefreshToken } = response.data;
        this.setTokens(accessToken, newRefreshToken);
        return accessToken;
      })
      .catch((error) => {
        this.clearTokens();
        throw error;
      })
      .finally(() => {
        this.refreshTokenPromise = null;
      });

    return this.refreshTokenPromise;
  }

  getAccessToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
  }

  getRefreshToken(): string | null {
    return localStorage.getItem(REFRESH_TOKEN_KEY);
  }

  setTokens(accessToken: string, refreshToken: string): void {
    localStorage.setItem(TOKEN_KEY, accessToken);
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  }

  clearTokens(): void {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
  }

  isTokenExpired(token: string): boolean {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return payload.exp * 1000 < Date.now();
    } catch {
      return true;
    }
  }
}

export const authService = new AuthService();
```

### 3. Axios Interceptors

```typescript
// frontend/src/services/api.ts
import axios, { AxiosError, AxiosRequestConfig } from 'axios';
import { authService } from './authService';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:3000';

export const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add auth token
api.interceptors.request.use(
  (config) => {
    const token = authService.getAccessToken();
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor to handle token refresh
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as AxiosRequestConfig & { _retry?: boolean };

    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      try {
        const newToken = await authService.refreshAccessToken();
        if (newToken) {
          originalRequest.headers = originalRequest.headers || {};
          originalRequest.headers.Authorization = `Bearer ${newToken}`;
          return api(originalRequest);
        }
      } catch (refreshError) {
        // Refresh failed, redirect to login
        authService.clearTokens();
        window.location.href = '/login';
        return Promise.reject(refreshError);
      }
    }

    return Promise.reject(error);
  }
);
```

### 4. Login Component

```typescript
// frontend/src/components/auth/LoginForm.tsx
import React, { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import {
  TextField,
  Button,
  Box,
  Typography,
  Alert,
  CircularProgress,
  InputAdornment,
  IconButton,
} from '@mui/material';
import { Visibility, VisibilityOff } from '@mui/icons-material';
import { useAuth } from '../../contexts/AuthContext';

const schema = yup.object({
  email: yup.string().email('Invalid email').required('Email is required'),
  password: yup.string().required('Password is required'),
});

type FormData = {
  email: string;
  password: string;
};

export const LoginForm: React.FC = () => {
  const navigate = useNavigate();
  const { login, isLoading, error, clearError } = useAuth();
  const [showPassword, setShowPassword] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<FormData>({
    resolver: yupResolver(schema),
  });

  const onSubmit = async (data: FormData) => {
    try {
      await login(data);
      navigate('/chat');
    } catch (error) {
      // Error handled by context
    }
  };

  return (
    <Box
      component="form"
      onSubmit={handleSubmit(onSubmit)}
      sx={{
        maxWidth: 400,
        mx: 'auto',
        mt: 8,
        p: 3,
        borderRadius: 2,
        boxShadow: 3,
      }}
    >
      <Typography variant="h4" component="h1" gutterBottom align="center">
        Sign In
      </Typography>

      {error && (
        <Alert severity="error" onClose={clearError} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <TextField
        {...register('email')}
        label="Email"
        type="email"
        fullWidth
        margin="normal"
        error={!!errors.email}
        helperText={errors.email?.message}
        autoComplete="email"
        autoFocus
      />

      <TextField
        {...register('password')}
        label="Password"
        type={showPassword ? 'text' : 'password'}
        fullWidth
        margin="normal"
        error={!!errors.password}
        helperText={errors.password?.message}
        autoComplete="current-password"
        InputProps={{
          endAdornment: (
            <InputAdornment position="end">
              <IconButton
                onClick={() => setShowPassword(!showPassword)}
                edge="end"
              >
                {showPassword ? <VisibilityOff /> : <Visibility />}
              </IconButton>
            </InputAdornment>
          ),
        }}
      />

      <Button
        type="submit"
        fullWidth
        variant="contained"
        sx={{ mt: 3, mb: 2 }}
        disabled={isLoading}
      >
        {isLoading ? <CircularProgress size={24} /> : 'Sign In'}
      </Button>

      <Box sx={{ textAlign: 'center' }}>
        <Link to="/forgot-password" style={{ textDecoration: 'none' }}>
          <Typography variant="body2" color="primary">
            Forgot password?
          </Typography>
        </Link>
        <Typography variant="body2" sx={{ mt: 1 }}>
          Don't have an account?{' '}
          <Link to="/register" style={{ textDecoration: 'none' }}>
            Sign Up
          </Link>
        </Typography>
      </Box>
    </Box>
  );
};
```

### 5. Protected Route Component

```typescript
// frontend/src/components/auth/ProtectedRoute.tsx
import React from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { CircularProgress, Box } from '@mui/material';
import { useAuth } from '../../contexts/AuthContext';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requiredRole?: string;
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requiredRole,
}) => {
  const { isAuthenticated, isLoading, user } = useAuth();
  const location = useLocation();

  if (isLoading) {
    return (
      <Box
        display="flex"
        justifyContent="center"
        alignItems="center"
        minHeight="100vh"
      >
        <CircularProgress />
      </Box>
    );
  }

  if (!isAuthenticated) {
    // Redirect to login page but save the attempted location
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  if (requiredRole && user?.role !== requiredRole) {
    // User doesn't have required role
    return <Navigate to="/unauthorized" replace />;
  }

  return <>{children}</>;
};
```

### 6. User Profile Component

```typescript
// frontend/src/components/profile/UserProfile.tsx
import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import {
  Container,
  Paper,
  TextField,
  Button,
  Avatar,
  Box,
  Typography,
  Alert,
  Snackbar,
  IconButton,
} from '@mui/material';
import { PhotoCamera } from '@mui/icons-material';
import { useAuth } from '../../contexts/AuthContext';

const schema = yup.object({
  username: yup.string().min(3).max(30).required('Username is required'),
  email: yup.string().email('Invalid email').required('Email is required'),
  bio: yup.string().max(500, 'Bio must be less than 500 characters'),
});

type FormData = {
  username: string;
  email: string;
  bio?: string;
};

export const UserProfile: React.FC = () => {
  const { user, updateProfile, isLoading } = useAuth();
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors, isDirty },
  } = useForm<FormData>({
    resolver: yupResolver(schema),
    defaultValues: {
      username: user?.username || '',
      email: user?.email || '',
      bio: user?.bio || '',
    },
  });

  const onSubmit = async (data: FormData) => {
    try {
      await updateProfile(data);
      setSuccess(true);
    } catch (error: any) {
      setError(error.message || 'Failed to update profile');
    }
  };

  const handleAvatarUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    // TODO: Implement avatar upload
    const formData = new FormData();
    formData.append('avatar', file);
    // Upload logic here
  };

  return (
    <Container maxWidth="sm" sx={{ mt: 4 }}>
      <Paper elevation={3} sx={{ p: 4 }}>
        <Typography variant="h4" component="h1" gutterBottom align="center">
          My Profile
        </Typography>

        <Box display="flex" justifyContent="center" mb={3}>
          <Box position="relative">
            <Avatar
              src={user?.avatarUrl}
              sx={{ width: 100, height: 100 }}
            />
            <IconButton
              color="primary"
              aria-label="upload picture"
              component="label"
              sx={{
                position: 'absolute',
                bottom: 0,
                right: 0,
                bgcolor: 'background.paper',
              }}
            >
              <input
                hidden
                accept="image/*"
                type="file"
                onChange={handleAvatarUpload}
              />
              <PhotoCamera />
            </IconButton>
          </Box>
        </Box>

        <Box component="form" onSubmit={handleSubmit(onSubmit)}>
          <TextField
            {...register('username')}
            label="Username"
            fullWidth
            margin="normal"
            error={!!errors.username}
            helperText={errors.username?.message}
          />

          <TextField
            {...register('email')}
            label="Email"
            type="email"
            fullWidth
            margin="normal"
            error={!!errors.email}
            helperText={errors.email?.message}
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
          />

          <Button
            type="submit"
            fullWidth
            variant="contained"
            sx={{ mt: 3 }}
            disabled={isLoading || !isDirty}
          >
            Update Profile
          </Button>
        </Box>

        <Snackbar
          open={success}
          autoHideDuration={6000}
          onClose={() => setSuccess(false)}
        >
          <Alert onClose={() => setSuccess(false)} severity="success">
            Profile updated successfully!
          </Alert>
        </Snackbar>

        {error && (
          <Alert severity="error" sx={{ mt: 2 }} onClose={() => setError(null)}>
            {error}
          </Alert>
        )}
      </Paper>
    </Container>
  );
};
```

## Form Validation

### Validation Schema Examples
```typescript
// Registration validation
const registerSchema = yup.object({
  email: yup
    .string()
    .email('Invalid email format')
    .required('Email is required'),
  username: yup
    .string()
    .min(3, 'Username must be at least 3 characters')
    .max(30, 'Username must be less than 30 characters')
    .matches(/^[a-zA-Z0-9_]+$/, 'Username can only contain letters, numbers, and underscores')
    .required('Username is required'),
  password: yup
    .string()
    .min(8, 'Password must be at least 8 characters')
    .matches(
      /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/,
      'Password must contain uppercase, lowercase, and number'
    )
    .required('Password is required'),
  confirmPassword: yup
    .string()
    .oneOf([yup.ref('password')], 'Passwords must match')
    .required('Please confirm your password'),
});
```

## Security Best Practices

### Token Storage
- Store access tokens in memory when possible
- Use httpOnly cookies for refresh tokens in production
- Implement token rotation on refresh
- Clear tokens on logout

### XSS Prevention
- Sanitize all user inputs
- Use React's built-in XSS protection
- Avoid dangerouslySetInnerHTML
- Validate all data from API

### CSRF Protection
- Include CSRF tokens in forms
- Verify origin headers
- Use SameSite cookie attribute

## Error Handling

### User-Friendly Error Messages
```typescript
const getErrorMessage = (error: any): string => {
  if (error.response?.data?.errors) {
    // Validation errors from backend
    return error.response.data.errors
      .map((e: any) => e.message)
      .join(', ');
  }
  
  if (error.response?.data?.message) {
    return error.response.data.message;
  }
  
  if (error.code === 'NETWORK_ERROR') {
    return 'Unable to connect to server. Please check your connection.';
  }
  
  return 'An unexpected error occurred. Please try again.';
};
```

## Testing Strategy

### Unit Tests
- Test auth context methods
- Test form validation
- Test protected route logic
- Test token refresh mechanism

### Integration Tests
- Complete login flow
- Registration with validation
- Token expiry and refresh
- Profile update flow