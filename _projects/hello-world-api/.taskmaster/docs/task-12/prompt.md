# Autonomous Agent Prompt: Research Express.js Best Practices

## Context
You are tasked with researching and documenting current Express.js best practices. The Hello World API project needs comprehensive documentation on modern Express.js patterns for error handling, middleware organization, logging, and security. This research will guide future development decisions and improvements.

## Objective
Conduct thorough research using web search to gather current Express.js best practices and compile them into a well-structured markdown document (docs/best-practices.md) that provides actionable guidance for the development team.

## Task Requirements

### 1. Research Focus Areas
Conduct web searches on the following topics:
- **Error Handling**: Global handlers, async patterns, error formatting
- **Middleware**: Organization, ordering, custom patterns
- **Logging**: Libraries, configuration, best practices
- **Security**: Helmet.js, CORS, rate limiting, validation

### 2. Search Strategy
Use these search queries:
- "Express.js error handling best practices 2024"
- "Express.js middleware organization patterns"
- "Express.js production logging Morgan Winston"
- "Express.js security checklist Helmet CORS"
- "Express.js async await error handling"
- "Express.js project structure best practices"

### 3. Documentation Creation
Create `docs/best-practices.md` with:
- Table of contents
- Introduction section
- Detailed sections for each topic
- Code examples
- References and sources

## Research Execution Plan

### Step 1: Error Handling Research
```
Search for:
- Express error middleware patterns
- Async/await error handling in Express
- Error response formatting standards
- Centralized error handling
- Production error logging

Document:
- Global error handler implementation
- Async route wrapper patterns
- Error class hierarchies
- Client-friendly error responses
```

### Step 2: Middleware Organization Research
```
Search for:
- Express middleware order best practices
- Application vs router middleware
- Middleware composition patterns
- Performance considerations
- Testing middleware

Document:
- Recommended middleware order
- Categorization strategies
- Custom middleware templates
- Middleware testing approaches
```

### Step 3: Logging Research
```
Search for:
- Express logging libraries comparison
- Morgan advanced configuration
- Winston/Bunyan/Pino setup
- Log rotation strategies
- Structured logging

Document:
- Library recommendations
- Environment-specific configs
- Log formatting standards
- Performance impact
```

### Step 4: Security Research
```
Search for:
- Express security middleware
- Helmet.js configuration
- CORS best practices
- Rate limiting strategies
- Input validation libraries

Document:
- Essential security middleware
- Configuration examples
- Common vulnerabilities
- Security checklist
```

## Documentation Structure

```markdown
# Express.js Best Practices Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Middleware Organization](#middleware-organization)
4. [Request Logging](#request-logging)
5. [Security Best Practices](#security-best-practices)
6. [Performance Optimization](#performance-optimization)
7. [Testing Strategies](#testing-strategies)
8. [References](#references)

## Introduction
This guide documents current best practices for Express.js applications...

## Error Handling Patterns

### Global Error Handler
```javascript
// Example implementation
```

### Async Error Handling
```javascript
// Example with express-async-errors
```

[Continue for each section...]
```

## Quality Guidelines

### For Each Topic
1. **Current Information**: Focus on 2023-2024 content
2. **Multiple Sources**: Cross-reference at least 3 sources
3. **Code Examples**: Provide working examples
4. **Pros and Cons**: Discuss tradeoffs
5. **Our Context**: Relate to Hello World API

### Code Example Standards
- Use ES6+ syntax
- Include comments
- Show both basic and advanced usage
- Highlight security considerations
- Note version compatibility

## Validation Criteria

### Research Completeness
- [ ] All 4 main topics researched
- [ ] Minimum 3 sources per topic
- [ ] Recent information (2023-2024)
- [ ] Official documentation referenced
- [ ] Community best practices included

### Documentation Quality
- [ ] Clear table of contents
- [ ] Well-structured sections
- [ ] Practical code examples
- [ ] Security considerations noted
- [ ] Performance implications discussed
- [ ] References properly cited

## Expected Deliverable

Create `docs/best-practices.md` containing:
1. **Comprehensive Coverage**: All research topics addressed
2. **Actionable Guidance**: Can be immediately applied
3. **Code Examples**: Working implementations
4. **Modern Practices**: Current Express.js patterns
5. **Security Focus**: Security considered throughout
6. **References**: Links to sources and further reading

## Important Notes

- Focus on Express.js 4.x (current stable version)
- Prioritize official documentation and recognized experts
- Include both basic and advanced patterns
- Consider our simple API context but document for scalability
- Note any controversial or debated practices
- Highlight security implications throughout

## Tools Required
- Web search capability (Brave Search)
- File system access to create docs/best-practices.md
- Markdown formatting knowledge
- Express.js understanding

Proceed with researching each topic systematically, gathering the most current and relevant information, and compiling it into a comprehensive best practices guide that will serve as a valuable reference for the project.