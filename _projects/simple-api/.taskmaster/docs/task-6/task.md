# Task 6: Add Input Validation Middleware

## Overview
Implement middleware for validating user input on POST `/api/users` with meaningful error messages. This provides a reusable validation layer that runs before controller logic.

## Task Details
- **Priority**: Medium
- **Dependencies**: Task 4 (Develop User Management Endpoints)
- **Status**: Pending

## Implementation Guide

### 1. Create Validation Middleware
`/src/middleware/validateUser.js`:
```javascript
import validator from 'validator';
import { ValidationError } from '../utils/errors.js';

export const validateCreateUser = (req, res, next) => {
  try {
    const { name, email } = req.body;
    const errors = [];
    
    // Validate name
    if (!name) {
      errors.push('Name is required');
    } else if (typeof name !== 'string') {
      errors.push('Name must be a string');
    } else if (name.trim().length === 0) {
      errors.push('Name cannot be empty');
    } else if (name.trim().length < 2) {
      errors.push('Name must be at least 2 characters long');
    } else if (name.trim().length > 100) {
      errors.push('Name must not exceed 100 characters');
    }
    
    // Validate email
    if (!email) {
      errors.push('Email is required');
    } else if (typeof email !== 'string') {
      errors.push('Email must be a string');
    } else if (!validator.isEmail(email)) {
      errors.push('Email must be a valid email address');
    }
    
    // Check for errors
    if (errors.length > 0) {
      throw new ValidationError(errors.join(', '));
    }
    
    // Sanitize data
    req.body.name = name.trim();
    req.body.email = email.toLowerCase().trim();
    
    next();
  } catch (error) {
    next(error);
  }
};
```

### 2. Create Generic Validation Helpers
`/src/utils/validators.js`:
```javascript
import validator from 'validator';

export const validateString = (value, fieldName, options = {}) => {
  const { min = 1, max = 255, required = true } = options;
  const errors = [];
  
  if (required && !value) {
    errors.push(`${fieldName} is required`);
    return errors;
  }
  
  if (value && typeof value !== 'string') {
    errors.push(`${fieldName} must be a string`);
    return errors;
  }
  
  if (value) {
    const trimmed = value.trim();
    if (trimmed.length < min) {
      errors.push(`${fieldName} must be at least ${min} characters long`);
    }
    if (trimmed.length > max) {
      errors.push(`${fieldName} must not exceed ${max} characters`);
    }
  }
  
  return errors;
};

export const validateEmail = (value, fieldName = 'Email') => {
  const errors = [];
  
  if (!value) {
    errors.push(`${fieldName} is required`);
    return errors;
  }
  
  if (!validator.isEmail(value)) {
    errors.push(`${fieldName} must be a valid email address`);
  }
  
  return errors;
};
```

### 3. Update User Routes
`/src/routes/users.js`:
```javascript
import { Router } from 'express';
import { listUsers, addUser } from '../controllers/users.js';
import { validateCreateUser } from '../middleware/validateUser.js';

const router = Router();

router.get('/api/users', listUsers);
router.post('/api/users', validateCreateUser, addUser);

export default router;
```

### 4. Simplify Controller
Update user controller to remove validation logic:
```javascript
export const addUser = (req, res, next) => {
  try {
    // Validation already done by middleware
    const { name, email } = req.body;
    
    // Create user with sanitized data
    const newUser = createUser({ name, email });
    res.status(201).json(newUser);
  } catch (error) {
    next(error);
  }
};
```

## Validation Rules
- **Name**:
  - Required
  - String type
  - 2-100 characters after trim
  - Not just whitespace
  
- **Email**:
  - Required
  - Valid email format
  - Normalized to lowercase
  - Trimmed of whitespace

## Acceptance Criteria
- [ ] Validation runs before controller logic
- [ ] Multiple validation errors reported together
- [ ] Data sanitization (trim, lowercase)
- [ ] Meaningful error messages
- [ ] Reusable validation utilities
- [ ] Integration with error handling middleware
- [ ] 400 status for validation errors

## Test Strategy
1. Test missing fields
2. Test invalid data types
3. Test boundary conditions (min/max length)
4. Test email format validation
5. Test data sanitization
6. Test multiple errors reported together
7. Verify middleware order execution