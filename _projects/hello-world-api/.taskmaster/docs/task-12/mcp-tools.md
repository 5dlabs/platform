# MCP Tools for Task 12: Research Express.js Best Practices

## Tool Selection Reasoning
This task is primarily focused on research and documentation creation. The main operations involve:
- Creating a documentation directory if needed
- Writing a comprehensive best practices document
- Potentially reading existing project files for context
- Organizing and formatting the research findings

The filesystem tool provides all necessary capabilities for creating and managing documentation files. While web research might be conducted, the final deliverable is a local documentation file.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Essential for creating the documentation directory and writing the best practices guide. Also useful for reading existing project files to provide context-specific recommendations.

**Available Operations**:
- `create_directory`: Create the docs directory if it doesn't exist
- `write_file`: Create the best-practices.md file
- `read_file`: Review existing code for context
- `edit_file`: Update documentation as needed
- `list_directory`: Verify documentation structure

**Task-Specific Usage Examples**:

1. **Create Documentation Directory**:
```javascript
// Ensure docs directory exists
create_directory({ path: "hello-world-api/docs" })
```

2. **Create Best Practices Document**:
```javascript
write_file({
  path: "hello-world-api/docs/best-practices.md",
  content: `# Express.js Best Practices Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Middleware Organization](#middleware-organization)
4. [Request Logging](#request-logging)
5. [Security Best Practices](#security-best-practices)
6. [Performance Optimization](#performance-optimization)
7. [References](#references)

## Introduction
This document outlines current best practices for Express.js applications...

[Full content continues...]`
})
```

3. **Read Current Implementation for Context**:
```javascript
// Review current server implementation
const serverCode = read_file({ path: "hello-world-api/src/index.js" })

// Review package.json for dependencies
const packageJson = read_file({ path: "hello-world-api/package.json" })

// Use this context to make specific recommendations
```

4. **Create Structured Documentation**:
```javascript
// Build documentation in sections
const sections = {
  introduction: "This document outlines...",
  errorHandling: "## Error Handling Patterns\n\n...",
  middleware: "## Middleware Organization\n\n...",
  logging: "## Request Logging\n\n...",
  security: "## Security Best Practices\n\n...",
  references: "## References\n\n..."
}

// Combine sections
const fullDocument = Object.values(sections).join('\n\n')

write_file({
  path: "hello-world-api/docs/best-practices.md",
  content: fullDocument
})
```

5. **Update Documentation Iteratively**:
```javascript
// Add new findings to existing document
edit_file({
  path: "hello-world-api/docs/best-practices.md",
  edits: [{
    oldText: "## References",
    newText: `## Performance Optimization

### Compression
\`\`\`javascript
const compression = require('compression');
app.use(compression());
\`\`\`

### Caching Strategies
...

## References`
  }]
})
```

## Tool Usage Guidelines for This Task

### Documentation Creation Strategy
1. **Structure First**: Create outline before filling content
2. **Code Examples**: Include practical, working examples
3. **Formatting**: Use proper Markdown syntax
4. **Organization**: Logical flow from basics to advanced

### Best Practices for Documentation
1. **Clarity**: Write for developers of all levels
2. **Completeness**: Cover all required topics thoroughly
3. **Practicality**: Include real-world applicable examples
4. **Currency**: Focus on current Express.js versions
5. **References**: Cite authoritative sources

### Research Integration
1. Gather information from multiple sources
2. Synthesize findings into coherent sections
3. Provide code examples for each concept
4. Include pros/cons where applicable
5. Make specific recommendations for the project

## Content Organization Pattern
```javascript
// Recommended structure for each section
const sectionTemplate = `
## Section Title

### Overview
Brief introduction to the topic

### Current Approach
How it's currently done in the project

### Best Practice Recommendation
Industry standard approach with example

### Implementation Example
\`\`\`javascript
// Practical code example
\`\`\`

### Benefits
- Performance improvements
- Security enhancements
- Maintainability

### Considerations
- Migration effort
- Compatibility issues
- Trade-offs
`;
```

## Integration Considerations
- Link best practices to specific files in the project
- Provide migration paths for implementing recommendations
- Consider the project's current state and constraints
- Balance ideal practices with practical implementation
- Include both immediate and long-term improvements

## Quality Assurance
1. **Verify all code examples compile/run**
2. **Check Markdown formatting renders correctly**
3. **Ensure all links are valid**
4. **Confirm completeness of all sections**
5. **Review for clarity and consistency**

## Common Patterns
1. Research topic → Write section → Add code examples → Include references
2. Review current code → Identify gaps → Document improvements
3. Create outline → Fill sections → Review and refine → Add TOC
4. Write examples → Test code → Document results → Add to guide