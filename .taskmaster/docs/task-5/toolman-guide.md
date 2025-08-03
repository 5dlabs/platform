# Toolman Guide: Handler Layer Removal

## Overview
Tools selected for Rust code migration and cleanup.

## Core Tools

### Rust Documentation
- `query_rust_docs`: Research code structure
  - Analyze module organization
  - Check dependencies

### File Operations
- `read_file`: Review handler code
- `write_file`: Update modules
- `list_directory`: Find handlers
- `move_file`: Migrate logic

## Implementation Flow
1. Use `read_file` to analyze handlers
2. `query_rust_docs` for code structure
3. `list_directory` to locate files
4. `move_file` for code migration

## Best Practices
- Preserve business logic
- Check all dependencies
- Maintain test coverage
- Document changes