# Toolman Guide for AI Tools Research Task

## Tool Selection Overview
This task requires web research capabilities and local file operations. The selected tools enable efficient market research and data management.

## Core Tools

### Brave Web Search
- **Purpose**: Primary research tool for gathering market intelligence
- **Usage**: Execute searches for AI development tools and related information
- **Best Practices**:
  - Use specific search queries for targeted results
  - Filter by recent content for up-to-date information
  - Combine multiple searches for comprehensive coverage

### Local Filesystem Tools

#### read_file
- **Purpose**: Read existing research data and documentation
- **Usage**: Access saved research notes and previous findings
- **Context**: Essential for maintaining research continuity

#### write_file
- **Purpose**: Save research findings and generate reports
- **Usage**: Create markdown reports and save structured data
- **Best Practices**: 
  - Use consistent file naming
  - Maintain proper markdown formatting
  - Include required metadata

#### list_directory
- **Purpose**: Manage research file organization
- **Usage**: Track saved artifacts and ensure complete documentation
- **When to Use**: Before saving new files or accessing existing data

#### create_directory
- **Purpose**: Set up research directory structure
- **Usage**: Organize research artifacts in logical hierarchy
- **Best Practices**: Follow project directory conventions

## Implementation Flow
1. Start with Brave Search for initial research
2. Create directories for storing findings
3. Write research data progressively
4. Generate final reports and documentation

## Best Practices
- Maintain organized file structure
- Save research progressively
- Validate data before final storage
- Follow markdown formatting guidelines

## Troubleshooting
- Verify directory permissions before file operations
- Ensure proper file extensions for markdown
- Check for duplicate files before creation