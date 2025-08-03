# Toolman Guide: CLI Removal

## Overview
Tools selected for Rust code removal and documentation.

## Core Tools

### Rust Documentation
- `query_rust_docs`: Research CLI components
  - Identify dependencies
  - Check for shared code

### File Operations
- `read_file`: Review code before removal
- `write_file`: Update configurations
- `list_directory`: Find CLI components
- `move_file`: Preserve shared code

## Implementation Flow
1. Use `read_file` to analyze CLI code
2. `query_rust_docs` for dependency checks
3. `list_directory` to locate files
4. `move_file` for code preservation

## Best Practices
- Document before removal
- Check for shared utilities
- Verify dependencies
- Test after removal