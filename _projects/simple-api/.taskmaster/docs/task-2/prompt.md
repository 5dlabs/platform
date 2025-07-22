# Autonomous Task Prompt: Implement Express Server and Middleware

You are tasked with creating the core Express.js server with proper middleware configuration and startup logging.

## Context
- Project has been initialized with express@5 and dotenv@16
- Environment configuration exists in .env file
- Need to create a production-ready Express server

## Your Mission
Implement a clean, well-structured Express server that handles JSON requests and provides clear startup logging.

## Steps to Complete

1. **Create the main server file** (`src/index.js`)
   - Import express and dotenv
   - Configure dotenv to load environment variables
   - Create Express application instance
   - Set up JSON parsing middleware

2. **Configure server startup**
   - Use PORT from environment or default to 3000
   - Add informative startup logging with timestamps
   - Display current environment (development/production)

3. **Update package.json**
   - Add `"type": "module"` for ES modules
   - Define start script: `"start": "node src/index.js"`
   - Define dev script: `"dev": "nodemon src/index.js"`

4. **Implement best practices**
   - Proper middleware ordering
   - Clean code structure
   - Prepare for future middleware additions
   - Handle basic server errors

5. **Test the implementation**
   - Verify server starts successfully
   - Check environment variable loading
   - Confirm JSON parsing works
   - Test both start and dev scripts

## Success Criteria
- Server starts without errors
- Logs show correct port and environment
- JSON middleware processes requests correctly
- Nodemon restarts server on file changes
- Code follows Express.js 5 best practices

## Technical Requirements
- Use ES6 modules (import/export)
- Implement proper error handling
- Follow Express middleware patterns
- Keep code modular and extensible

## Notes
- Focus on creating a solid foundation
- Prepare structure for routes and error handlers
- Ensure compatibility with Express 5.x
- Keep startup logs informative but concise