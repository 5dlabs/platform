# Autonomous AI Agent Prompt: Research Express.js Best Practices

## Task Overview
You need to research current Express.js best practices using web search, focusing on error handling, middleware organization, request logging, and security. Document your findings in a structured markdown file at `docs/best-practices.md`.

## Detailed Instructions

### Step 1: Create Documentation Directory
```bash
mkdir -p hello-world-api/docs
```

### Step 2: Research Error Handling Patterns
Search for:
- "Express.js error handling best practices 2024"
- "Express async error handling patterns"
- "Express.js global error handler examples"

Document findings including:
- Global error handler middleware patterns
- Async/await error handling approaches
- Error response formatting standards
- Error logging strategies

### Step 3: Research Middleware Organization
Search for:
- "Express.js middleware order best practices"
- "Express middleware organization patterns"
- "Express.js middleware composition"

Document findings including:
- Recommended middleware order
- Application vs router-level middleware
- Custom middleware patterns
- Middleware categorization strategies

### Step 4: Research Request Logging
Search for:
- "Express.js Morgan configuration best practices"
- "Express.js structured logging Winston"
- "Express.js request correlation ID"

Document findings including:
- Morgan configuration examples
- Winston integration patterns
- Log rotation strategies
- Sensitive data protection in logs

### Step 5: Research Security Best Practices
Search for:
- "Express.js security best practices Helmet"
- "Express.js CORS configuration"
- "Express.js rate limiting strategies"

Document findings including:
- Helmet.js configuration
- CORS setup patterns
- Rate limiting implementation
- Input validation approaches
- Security headers recommendations

### Step 6: Create the Documentation File
Create `docs/best-practices.md` with this structure:

```markdown
# Express.js Best Practices

## Table of Contents
1. [Introduction](#introduction)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Middleware Organization](#middleware-organization)
4. [Request Logging](#request-logging)
5. [Security Best Practices](#security-best-practices)
6. [References](#references)

## Introduction
This document outlines current best practices for Express.js applications based on industry standards and community recommendations as of 2024.

## Error Handling Patterns

### Global Error Handler
```javascript
// Example code here
```

### Async Error Handling
```javascript
// Example code here
```

[Continue with other sections...]

## References
- [List of sources]
```

## Expected Outcomes

### Documentation File
- Location: `hello-world-api/docs/best-practices.md`
- Well-structured markdown with table of contents
- Code examples for each best practice
- Clear explanations and rationale
- Current references (2023-2024)

### Content Coverage
1. **Error Handling**: At least 3 patterns with examples
2. **Middleware**: Organization strategies and order
3. **Logging**: Configuration examples for Morgan/Winston
4. **Security**: Helmet, CORS, and rate limiting setup

## Validation Steps

1. **File Creation**
   ```bash
   test -f hello-world-api/docs/best-practices.md && echo "File created" || echo "File missing"
   ```

2. **Content Verification**
   - Check all sections are present
   - Verify code examples are included
   - Ensure references are provided

3. **Markdown Validation**
   - Proper heading structure
   - Code blocks properly formatted
   - Links in table of contents work

## Research Guidelines

### Source Selection
- Prioritize official Express.js documentation
- Use recent articles (2023-2024)
- Include reputable tech blogs (e.g., LogRocket, DigitalOcean)
- Reference Stack Overflow accepted answers

### Code Example Requirements
- Examples should be practical and implementable
- Include comments explaining key concepts
- Show both basic and advanced patterns
- Ensure code is syntactically correct

### Documentation Style
- Use clear, concise language
- Explain the "why" not just the "how"
- Include pros/cons where relevant
- Keep examples relevant to API development

## Common Issues and Solutions

### Issue: Outdated Information
**Solution**: Cross-reference multiple sources and check dates

### Issue: Conflicting Best Practices
**Solution**: Document multiple approaches with context

### Issue: Complex Examples
**Solution**: Start with simple examples, then show advanced patterns

## Important Notes

- Focus on practices relevant to REST APIs
- Avoid practices specific to full-stack applications
- Keep security recommendations practical
- Include performance considerations where relevant
- Document practices that scale from small to large applications
- Ensure all code examples use ES6+ syntax
- Include error handling in all code examples

## Quality Checklist

- [ ] All four main topics covered comprehensively
- [ ] At least 2-3 code examples per section
- [ ] References include publication dates
- [ ] Table of contents with working links
- [ ] Clear explanation of each practice
- [ ] Practical implementation guidance
- [ ] No outdated patterns (pre-2023)
- [ ] Security practices are current
- [ ] Examples tested for syntax correctness
- [ ] Documentation is well-formatted