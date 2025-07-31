# Toolman Guide for Task 6: Frontend Authentication and User Profile

## Overview

This guide explains how to effectively use Toolman to implement the frontend authentication system for the React application. Toolman provides a comprehensive set of tools for React development, testing, and deployment.

## Getting Started

### 1. Initial Setup

Before starting the implementation, use Toolman to set up your development environment:

```bash
# Install required dependencies
toolman shell "npm install react-router-dom react-hook-form @hookform/resolvers yup axios @mui/material @emotion/react @emotion/styled"

# Install development dependencies
toolman shell "npm install -D @types/react @types/react-dom @testing-library/react @testing-library/jest-dom @testing-library/user-event"

# Verify TypeScript configuration
toolman read tsconfig.json
```

### 2. Project Structure Analysis

Use Toolman to understand the current project structure:

```bash
# Examine existing directory structure
toolman search "src/**/*.tsx" --type file

# Find existing authentication code
toolman search "auth|login|user" --type code

# Check current routing setup
toolman read src/App.tsx
toolman search "Router|Route" --type code
```

## Implementation Workflow

### Phase 1: Core Authentication Setup

#### Creating the Authentication Context

```bash
# Create the contexts directory
toolman shell "mkdir -p src/contexts"

# Create AuthContext.tsx
toolman write src/contexts/AuthContext.tsx --template react-context

# Add TypeScript types
toolman write src/types/user.ts --content "User interface types"
toolman write src/types/auth.ts --content "Auth response types"
```

#### Setting up API Service

```bash
# Create services directory
toolman shell "mkdir -p src/services"

# Create API configuration
toolman write src/services/api.ts --template axios-instance

# Add auth interceptors
toolman edit src/services/api.ts --add-interceptors
```

### Phase 2: Component Development

#### Creating Authentication Components

```bash
# Create component directories
toolman shell "mkdir -p src/components/auth"
toolman shell "mkdir -p src/components/profile"

# Generate form components
toolman generate component LoginForm --path src/components/auth --with-form
toolman generate component RegisterForm --path src/components/auth --with-form
toolman generate component ProtectedRoute --path src/components/auth --type hoc
```

#### Form Validation Setup

```bash
# Create validation schemas
toolman write src/schemas/auth.ts --content "Yup validation schemas"

# Test validation logic
toolman test src/schemas/auth.ts --watch
```

### Phase 3: Integration

#### Updating App Component

```bash
# Wrap app with AuthProvider
toolman edit src/App.tsx --wrap-provider AuthProvider

# Add routing configuration
toolman edit src/App.tsx --add-routes auth
```

#### Protected Route Implementation

```bash
# Find all routes that need protection
toolman search "<Route" --type code

# Wrap protected routes
toolman edit src/App.tsx --protect-routes "/dashboard,/profile,/settings"
```

### Phase 4: Testing

#### Unit Testing Components

```bash
# Run tests for auth components
toolman test src/components/auth --coverage

# Test specific component
toolman test LoginForm.test.tsx --watch

# Generate test file if missing
toolman generate test LoginForm --type component
```

#### Integration Testing

```bash
# Test complete auth flow
toolman test:integration "auth flow" --e2e

# Test API integration
toolman test src/services/api.test.ts --env test
```

### Phase 5: Type Checking and Linting

```bash
# Run TypeScript compiler
toolman check:types --strict

# Fix type errors
toolman fix:types --auto

# Run linter
toolman lint src --fix

# Check for React best practices
toolman analyze:react src/components
```

## Advanced Toolman Features

### 1. Component Generation

Toolman can generate boilerplate for common React patterns:

```bash
# Generate a form component with validation
toolman generate form PasswordResetForm --schema password-reset

# Generate a context provider
toolman generate context Theme --with-hook

# Generate a custom hook
toolman generate hook useAuthentication --return-type AuthState
```

### 2. Code Analysis

Analyze your authentication implementation:

```bash
# Check for security issues
toolman analyze:security src/components/auth

# Find unused code
toolman analyze:unused src

# Check bundle size impact
toolman analyze:bundle --entry src/index.tsx
```

### 3. Documentation Generation

Generate documentation from your code:

```bash
# Generate component documentation
toolman docs:generate src/components --format markdown

# Create API documentation
toolman docs:api src/contexts/AuthContext.tsx

# Generate usage examples
toolman docs:examples LoginForm --output docs/examples
```

### 4. Performance Optimization

```bash
# Analyze component render performance
toolman profile:components LoginForm

# Find unnecessary re-renders
toolman analyze:renders src/components/auth

# Suggest memoization opportunities
toolman optimize:react src/components
```

## Best Practices with Toolman

### 1. Development Workflow

```bash
# Start development server with hot reload
toolman dev --port 3000

# Watch for TypeScript errors
toolman watch:types

# Auto-fix linting issues on save
toolman watch:lint --fix
```

### 2. Testing Workflow

```bash
# Run tests in watch mode during development
toolman test --watch --coverage

# Run specific test suites
toolman test auth --suite unit
toolman test auth --suite integration

# Debug failing tests
toolman test LoginForm --debug
```

### 3. Code Quality Checks

```bash
# Pre-commit checks
toolman check:all --staged

# Full project validation
toolman validate --strict

# Generate quality report
toolman report:quality --output reports/
```

## Troubleshooting Common Issues

### Authentication Context Not Working

```bash
# Check provider setup
toolman debug:context AuthContext

# Verify provider wrapping
toolman analyze:providers src/App.tsx
```

### Form Validation Issues

```bash
# Test validation schema
toolman test:schema loginSchema --input sample.json

# Debug form state
toolman debug:form LoginForm
```

### Protected Routes Not Redirecting

```bash
# Trace route rendering
toolman trace:routes /dashboard

# Check authentication state
toolman debug:auth --verbose
```

### Token Refresh Problems

```bash
# Monitor API calls
toolman monitor:api --filter auth

# Debug interceptors
toolman debug:axios-interceptors
```

## Security Considerations

### 1. Token Storage

```bash
# Audit token storage
toolman audit:storage --tokens

# Check for exposed secrets
toolman scan:secrets src
```

### 2. XSS Prevention

```bash
# Scan for XSS vulnerabilities
toolman scan:xss src/components

# Validate input sanitization
toolman check:sanitization
```

### 3. Dependency Security

```bash
# Check for vulnerable dependencies
toolman audit:deps

# Update dependencies safely
toolman update:deps --safe
```

## Performance Optimization

### 1. Bundle Size

```bash
# Analyze bundle composition
toolman analyze:bundle --visualize

# Find large dependencies
toolman analyze:deps --size
```

### 2. Code Splitting

```bash
# Suggest code splitting points
toolman analyze:splits src

# Implement lazy loading
toolman refactor:lazy src/routes
```

### 3. Render Optimization

```bash
# Find expensive renders
toolman profile:renders

# Suggest memo usage
toolman suggest:memo src/components
```

## Deployment Preparation

```bash
# Build production bundle
toolman build --production

# Run production checks
toolman check:production

# Generate deployment report
toolman report:deployment
```

## Continuous Integration

```bash
# CI pipeline command
toolman ci:check

# Generate test reports
toolman test --ci --coverage --junit

# Type checking for CI
toolman check:types --no-emit
```

## Additional Resources

- Use `toolman help <command>` for detailed command documentation
- Run `toolman docs` to open the full Toolman documentation
- Check `toolman examples auth` for authentication-specific examples
- Join `toolman community` for support and best practices

Remember to commit your changes frequently and use Toolman's git integration:

```bash
# Stage and commit with conventional commits
toolman commit "feat: implement authentication context"

# Check commit message format
toolman check:commits
```