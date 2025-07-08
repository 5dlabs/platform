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