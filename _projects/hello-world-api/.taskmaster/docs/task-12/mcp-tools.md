# MCP Tools for Task 12: Research Express.js Best Practices

## Tool Selection Reasoning
This task involves researching Express.js best practices using web search and creating comprehensive documentation. I selected:
- **brave_web_search**: Essential for researching current Express.js best practices, patterns, and recommendations
- **filesystem**: Required for creating the documentation directory and writing the best practices guide

## Selected Tools

### brave_web_search (Remote Tool)
**Description**: Performs web searches using the Brave Search API for general queries, articles, and online content  
**Why Selected**: This task specifically requires researching current best practices, which necessitates searching for recent articles, documentation, and community recommendations  
**Task-Specific Usage**: 
- Search for Express.js error handling patterns
- Find middleware organization best practices
- Research logging libraries and configurations
- Discover security recommendations and implementations

**Example Searches**:
- "Express.js error handling best practices 2024"
- "Express.js middleware organization patterns"
- "Express.js production logging Morgan Winston"
- "Express.js security Helmet CORS setup"

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files  
**Why Selected**: Required to create the documentation file and potentially the docs directory  
**Task-Specific Usage**: 
- Use `create_directory` to ensure docs/ directory exists
- Use `write_file` to create docs/best-practices.md
- Use `read_file` to verify the documentation was created correctly

**Key Operations**:
1. Create `docs/` directory if it doesn't exist
2. Write the compiled research findings to `docs/best-practices.md`
3. Verify the file was created successfully

## Tool Usage Guidelines for This Task

### Research Phase
```javascript
// 1. Search for error handling best practices
brave_web_search({
  query: "Express.js error handling best practices 2024 async await",
  count: 20
})

// 2. Search for middleware patterns
brave_web_search({
  query: "Express.js middleware organization order patterns",
  count: 20
})

// 3. Search for logging approaches
brave_web_search({
  query: "Express.js logging Morgan Winston Pino production setup",
  count: 20
})

// 4. Search for security practices
brave_web_search({
  query: "Express.js security best practices Helmet CORS rate limiting 2024",
  count: 20
})
```

### Documentation Phase
```javascript
// 1. Ensure docs directory exists
create_directory("hello-world-api/docs")

// 2. Write the compiled best practices document
write_file("hello-world-api/docs/best-practices.md", compiledMarkdownContent)

// 3. Verify the file was created
read_file("hello-world-api/docs/best-practices.md")
```

## Best Practices for This Task

1. **Research Strategy**: Use multiple search queries to get comprehensive coverage
2. **Source Evaluation**: Prioritize recent (2023-2024) and authoritative sources
3. **Content Organization**: Structure findings clearly with sections and examples
4. **Code Examples**: Include practical, working code snippets in the documentation

## Research Workflow

### Phase 1: Gather Information
- Perform targeted searches for each topic area
- Collect information from multiple sources
- Cross-reference findings for accuracy
- Note version-specific information

### Phase 2: Compile and Structure
- Organize findings by topic
- Create clear section headings
- Add practical code examples
- Include security and performance notes

### Phase 3: Document Creation
- Write comprehensive markdown documentation
- Include table of contents
- Add references and sources
- Ensure actionable guidance

## Common Pitfalls to Avoid

1. **Don't rely** on outdated information (pre-2023)
2. **Avoid** generic Node.js practices that don't apply to Express
3. **Include** version compatibility information
4. **Focus** on practical, implementable recommendations

## Integration Notes

The research findings should:
- Be relevant to the Hello World API project
- Provide actionable improvements
- Cover security, performance, and maintainability
- Include modern Express.js patterns

This tool combination enables comprehensive research through web search while providing the file system capabilities needed to create lasting documentation of the findings.