# Toolman Guide: Architecture Simplification

## Overview
This guide explains the tools selected for implementing the architecture simplification phase, focusing on code removal and updates.

## Core Tools

### File Operations
- **read_file**: Read existing code files to understand what needs to be removed/modified
  - Use for examining main.rs, handlers, and other target files
  - Example: `read_file` orchestrator/core/src/main.rs

- **edit_file**: Make precise edits to update code and remove components
  - Use for removing API code and updating dependencies
  - Example: `edit_file` to remove API routes from main.rs

- **read_multiple_files**: Analyze multiple files simultaneously
  - Use for understanding dependencies and relationships
  - Example: Check all handler files at once

- **search_files**: Find files containing API/CLI code
  - Use to ensure complete removal of components
  - Example: Search for remaining API references

### Documentation
- **brave_web_search**: Research Argo Workflows integration
  - Use for understanding best practices
  - Example: Search for Argo Workflows Rust client examples

### Rust Tools
- **query_rust_docs**: Get Rust documentation help
  - Use for handling dependency updates
  - Example: Check trait implementations for controller

## Implementation Flow
1. Use `search_files` to identify all API/CLI code
2. Use `read_multiple_files` to analyze dependencies
3. Use `edit_file` to remove components
4. Use `brave_web_search` for Argo integration guidance
5. Use `query_rust_docs` for Rust-specific questions

## Best Practices
- Always read files before editing
- Use search to ensure complete removal
- Verify changes with tests after edits
- Document significant changes

## Troubleshooting
- If edits fail, check file permissions
- If search misses files, try different patterns
- For Rust errors, consult documentation
- For integration issues, check Argo docs