# MCP Tools for Task 12: Research Express.js Best Practices

## Tool Selection Reasoning
This task involves researching Express.js best practices online and documenting findings. I selected:
- **brave_web_search**: Essential for researching current Express.js best practices and patterns
- **filesystem**: Required for creating the documentation directory and writing the best practices markdown file

## Selected Tools

### brave_web_search (Remote Tool)
**Description**: Performs web searches using the Brave Search API, ideal for researching current best practices and documentation
**Why Selected**: This task requires researching current Express.js patterns, which requires web search capabilities
**Task-Specific Usage**: 
- Search for Express.js error handling patterns
- Research middleware organization best practices
- Find current security recommendations
- Discover logging strategies

### filesystem (Local Tool)
**Description**: File system operations including read, write, and directory management
**Why Selected**: Required to create the docs directory and write the research findings to a markdown file
**Task-Specific Usage**: 
- Use `create_directory` to create the docs folder
- Use `write_file` to create and write best-practices.md
- Use `read_file` to verify the content if needed

## Tool Usage Guidelines for This Task

### Research Phase
1. Use `brave_web_search` for each research topic:
   - "Express.js error handling best practices 2024"
   - "Express.js middleware organization patterns"
   - "Express.js request logging Morgan Winston"
   - "Express.js security Helmet CORS"

2. Focus on recent articles (2023-2024)
3. Look for official documentation and reputable sources
4. Gather code examples and implementation patterns

### Documentation Phase
1. Use `create_directory` to ensure docs folder exists
2. Use `write_file` to create best-practices.md
3. Structure the document with clear sections
4. Include practical code examples from research

## Example Tool Usage

```javascript
// Research phase - searching for best practices
const errorHandlingSearch = await brave_web_search({
  query: "Express.js error handling best practices 2024 async await",
  count: 10
});

const middlewareSearch = await brave_web_search({
  query: "Express.js middleware order organization patterns",
  count: 10
});

const loggingSearch = await brave_web_search({
  query: "Express.js Morgan Winston structured logging",
  count: 10
});

const securitySearch = await brave_web_search({
  query: "Express.js security Helmet CORS rate limiting 2024",
  count: 10
});

// Create documentation directory
await filesystem.create_directory({
  path: "hello-world-api/docs"
});

// Write the compiled research to markdown file
const bestPracticesContent = `# Express.js Best Practices

## Table of Contents
1. [Introduction](#introduction)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Middleware Organization](#middleware-organization)
4. [Request Logging](#request-logging)
5. [Security Best Practices](#security-best-practices)
6. [References](#references)

## Introduction
This document outlines current best practices for Express.js applications...

## Error Handling Patterns
[Research findings with code examples]

## Middleware Organization
[Research findings with code examples]

## Request Logging
[Research findings with code examples]

## Security Best Practices
[Research findings with code examples]

## References
[List of sources from research]
`;

await filesystem.write_file({
  path: "hello-world-api/docs/best-practices.md",
  content: bestPracticesContent
});
```

## Important Notes
- Focus searches on recent content (2023-2024)
- Prioritize official Express.js documentation
- Include practical, implementable code examples
- Organize findings in a clear, structured manner
- Cite sources in the references section
- Ensure all code examples are syntactically correct
- Keep the documentation relevant to REST API development