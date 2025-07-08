# Acceptance Criteria - User API Service

## Overall Success Criteria
- [ ] Express server starts without errors on port 3000
- [ ] All endpoints respond with correct status codes
- [ ] TypeScript compiles without errors
- [ ] README includes clear setup and usage instructions

## Endpoint Testing

### Health Check
- [ ] GET /health returns 200 OK
- [ ] Response includes status "ok"
- [ ] Response includes ISO timestamp

### User Management
- [ ] GET /users returns empty array initially
- [ ] GET /users returns 200 OK status
- [ ] POST /users creates new user with generated ID
- [ ] POST /users returns 201 Created status
- [ ] POST /users validates required fields (name, email)
- [ ] POST /users returns 400 for missing fields
- [ ] Created users appear in GET /users response
- [ ] User data persists between requests (during runtime)

## Code Quality
- [ ] TypeScript strict mode enabled
- [ ] Proper error handling middleware
- [ ] Consistent response formats
- [ ] Clean project structure

## Documentation
- [ ] README includes project overview
- [ ] API endpoints are documented with examples
- [ ] Setup instructions are clear and complete
- [ ] Curl examples work as documented

## Git & Pull Request
- [ ] All changes committed to a feature branch (NOT main)
- [ ] Descriptive commit message following project conventions
- [ ] Feature branch pushed to origin
- [ ] Pull request created via GitHub CLI
- [ ] Pull request has descriptive title and body
- [ ] Pull request URL provided as final output

## Testing Commands

```bash
# Health check
curl http://localhost:3000/health

# Get all users
curl http://localhost:3000/users

# Create a user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}'

# Test validation (should return 400)
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe"}'
```

## CRITICAL: Task Completion

**This task is ONLY considered complete when:**
1. All code implementation criteria above are met
2. A pull request has been created on GitHub
3. The PR URL is provided in the final output