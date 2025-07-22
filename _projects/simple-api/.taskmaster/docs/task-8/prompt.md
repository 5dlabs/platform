# Autonomous Task Prompt: Code Quality, Linting, and Final Review

You are tasked with ensuring code quality through linting setup and performing a comprehensive final review of the Express API project.

## Context
- Express API is feature-complete
- All endpoints and middleware implemented
- Documentation exists
- Need quality assurance and final polish

## Your Mission
Set up code quality tools, fix any issues, and perform a thorough review to ensure the project meets professional standards.

## Steps to Complete

1. **Set up ESLint**
   - Install eslint@8 as dev dependency
   - Configure for Node.js environment
   - Use recommended rules
   - Add custom rules for consistency
   - Configure to handle ES modules

2. **Configure code style**
   - Set indentation rules (2 spaces)
   - Enforce semicolons
   - Use single quotes
   - Configure line length limits
   - Handle Express middleware patterns

3. **Add npm scripts**
   - `lint`: Run ESLint check
   - `lint:fix`: Auto-fix issues
   - `format`: Run Prettier (if added)
   - Integrate into development workflow

4. **Fix linting issues**
   - Run linter on all source files
   - Fix style inconsistencies
   - Remove unused code
   - Apply consistent formatting
   - Resolve all warnings

5. **Code review**
   - Check separation of concerns
   - Verify error handling
   - Review security practices
   - Ensure DRY principle
   - Validate REST conventions

6. **Final testing**
   - Test all endpoints manually
   - Verify error scenarios
   - Check edge cases
   - Measure performance
   - Ensure stability

## Quality Standards

### Code Style
- Consistent indentation
- Proper naming conventions
- Clear variable names
- Logical file organization
- No code duplication

### Best Practices
- ES6+ features used properly
- Async/await over callbacks
- Proper error propagation
- Middleware order correct
- Security considerations

### Performance
- No synchronous file I/O
- Efficient algorithms
- Proper caching where needed
- Fast response times
- Memory efficiency

## Review Checklist
- [ ] All files pass linting
- [ ] No console.log in production code
- [ ] Error handling comprehensive
- [ ] Input validation complete
- [ ] Code is self-documenting
- [ ] No security vulnerabilities
- [ ] Performance acceptable
- [ ] Documentation accurate

## Success Criteria
- Zero linting errors
- Code follows best practices
- All manual tests pass
- Project structure clean
- Ready for deployment
- Professional quality

## Final Deliverables
- Configured linting tools
- Clean, consistent codebase
- Updated package.json scripts
- All issues resolved
- Production-ready code
- Complete documentation