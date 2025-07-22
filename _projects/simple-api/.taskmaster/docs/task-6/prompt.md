# Autonomous Task Prompt: Add Input Validation Middleware

You are tasked with creating reusable validation middleware for the Express API that validates and sanitizes user input before it reaches controllers.

## Context
- User endpoints exist with basic validation in controllers
- Need to separate validation logic into middleware
- Must provide detailed validation errors
- Should sanitize input data

## Your Mission
Create a comprehensive validation middleware system that validates user input, provides meaningful errors, and sanitizes data before processing.

## Steps to Complete

1. **Create validation middleware**
   - Build `/src/middleware/validateUser.js`
   - Implement validateCreateUser function
   - Check all required fields
   - Validate data types and formats
   - Collect all errors before responding

2. **Build validation utilities**
   - Create `/src/utils/validators.js`
   - Generic string validation helper
   - Email validation helper
   - Reusable validation functions
   - Support configurable rules

3. **Implement validation rules**
   - Name: required, string, 2-100 chars
   - Email: required, valid format
   - Trim whitespace from inputs
   - Normalize email to lowercase
   - Check for empty strings

4. **Integrate with routes**
   - Add middleware to POST /api/users
   - Ensure runs before controller
   - Pass sanitized data forward
   - Handle validation errors properly

5. **Update controllers**
   - Remove validation logic
   - Trust middleware-validated data
   - Simplify controller code
   - Maintain error handling

## Validation Requirements

### Name Field
- Must be present in request
- Must be a string type
- Minimum 2 characters (after trim)
- Maximum 100 characters
- Cannot be only whitespace

### Email Field  
- Must be present in request
- Must be valid email format
- Should be normalized (lowercase, trimmed)
- Use validator library for validation

### Error Response
When validation fails:
```json
{
  "error": "ValidationError",
  "message": "Name must be at least 2 characters long, Email must be a valid email address"
}
```

## Implementation Guidelines
- Collect ALL validation errors
- Don't stop at first error
- Provide specific error messages
- Sanitize valid input
- Use existing error classes
- Make validators reusable

## Success Criteria
- Validation prevents invalid data
- All errors reported together
- Data properly sanitized
- Middleware integrates cleanly
- Controllers simplified
- Error messages helpful

## Best Practices
- Validate early in request pipeline
- Sanitize data after validation
- Keep validation rules centralized
- Make error messages user-friendly
- Consider i18n for messages
- Log validation failures for monitoring