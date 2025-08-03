# Toolman Guide: API Server Removal

## Overview
Tools selected for Rust code modification and documentation.

## Core Tools

### Rust Documentation
- `query_rust_docs`: Research API components
  - Use for understanding Axum structure
  - Verify dependency usage

### File Operations
- `read_file`: Review code before removal
- `write_file`: Update configurations
- `list_directory`: Find API components
- `move_file`: Backup code if needed

## Implementation Flow
1. Use `read_file` to analyze current code
2. `query_rust_docs` for dependency checks
3. `list_directory` to locate API files
4. `write_file` for configuration updates

## Best Practices
- Backup files before removal
- Verify each file's purpose
- Document removed components
- Test after each major change