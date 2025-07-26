# MCP Tools for Task 13: Create README Documentation

## Tool Selection Reasoning
This task involves creating comprehensive README documentation for the project. The operations required are:
- Writing the README.md file to the project root
- Potentially reading existing files for accurate documentation
- Verifying the created documentation

The filesystem tool provides all necessary capabilities for creating and managing the README file. No remote services or external APIs are involved in creating project documentation.

## Selected Tools

### filesystem (Local Tool)
**Description**: File system operations for reading, writing, and managing files

**Why Selected**: Essential for creating the README.md file and potentially reading existing project files to ensure documentation accuracy. The tool enables both writing new documentation and verifying existing project structure.

**Available Operations**:
- `write_file`: Create the README.md file
- `read_file`: Review existing files for accurate documentation
- `list_directory`: Verify project structure
- `get_file_info`: Confirm README creation

**Task-Specific Usage Examples**:

1. **Create README.md File**:
```javascript
write_file({
  path: "hello-world-api/README.md",
  content: `# Hello World API

A simple Express.js API that demonstrates basic REST endpoint implementation with health checking and error handling.

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

[... full content continues ...]`
})
```

2. **Verify Project Structure**:
```javascript
// List project files to document structure
const projectFiles = list_directory({ path: "hello-world-api" })
const srcFiles = list_directory({ path: "hello-world-api/src" })

// Use this information to create accurate project structure documentation
```

3. **Read Package.json for Accuracy**:
```javascript
// Read package.json to document correct scripts and dependencies
const packageJson = JSON.parse(read_file({ path: "hello-world-api/package.json" }))

// Extract scripts for documentation
const scripts = packageJson.scripts
// Document as: npm start, npm run dev, etc.
```

4. **Read Server File for Endpoint Details**:
```javascript
// Read server implementation to document endpoints accurately
const serverCode = read_file({ path: "hello-world-api/src/index.js" })

// Extract endpoint information for API documentation section
// Ensure documented endpoints match actual implementation
```

5. **Build README in Sections**:
```javascript
// Build README content progressively
const sections = {
  header: `# Hello World API\n\nA simple Express.js API...`,
  toc: `\n## Table of Contents\n- [Overview](#overview)...`,
  overview: `\n## Overview\n\nThis is a minimal Express.js API...`,
  installation: `\n## Installation\n\n1. Clone the repository...`,
  usage: `\n## Usage\n\n### Starting the Server...`,
  api: `\n## API Documentation\n\n### Endpoints...`,
  development: `\n## Development\n\n### Available Scripts...`,
  footer: `\n## License\n\nThis project is licensed...`
}

// Combine all sections
const readmeContent = Object.values(sections).join('\n')

// Write complete README
write_file({
  path: "hello-world-api/README.md",
  content: readmeContent
})
```

6. **Verify README Creation**:
```javascript
// Confirm file was created
get_file_info({ path: "hello-world-api/README.md" })

// Read back to verify content
const readmeCheck = read_file({ path: "hello-world-api/README.md" })
```

## Tool Usage Guidelines for This Task

### Documentation Creation Strategy
1. **Accuracy First**: Read actual files to ensure documentation matches implementation
2. **Structure**: Create well-organized sections with clear headings
3. **Examples**: Include working code examples that users can copy
4. **Completeness**: Cover all aspects from installation to API usage

### Best Practices
1. **Read Before Writing**: Check actual files for accurate information
2. **Progressive Building**: Create sections incrementally
3. **Markdown Formatting**: Use proper Markdown syntax
4. **Code Blocks**: Include language hints for syntax highlighting
5. **Testing Examples**: Ensure all commands and examples work

### Content Verification Pattern
1. Read project files → Extract accurate information → Write documentation → Verify creation
2. Check package.json → Document correct scripts and dependencies
3. Review server code → Document actual endpoints and responses
4. List directories → Create accurate project structure diagram

## Integration Considerations
- Ensure README reflects the current state of the project
- Include all implemented endpoints (/, /health)
- Document actual error responses
- Reference the correct default port (3000)
- Include environment variable options

## Quality Assurance
1. **Command Accuracy**: All npm commands should match package.json
2. **Endpoint Accuracy**: API documentation should match implementation
3. **Example Validity**: All curl examples should work
4. **Path Correctness**: File paths and URLs should be accurate
5. **Formatting Check**: Markdown should render correctly

## Common Patterns
1. Read implementation → Document behavior → Provide examples
2. Extract from package.json → Document scripts → Add usage instructions
3. Check file structure → Create structure diagram → Explain organization
4. Gather endpoint details → Format API docs → Include curl examples

## Error Prevention
- **Version Matching**: Ensure documented features match implementation
- **Command Testing**: Verify all commands before documenting
- **URL Accuracy**: Use correct localhost URLs and ports
- **JSON Validity**: Ensure all JSON examples are properly formatted
- **Markdown Syntax**: Close all code blocks and format lists correctly