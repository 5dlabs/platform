# Pre-Commit Checklist for Orchestrator

Before committing any changes to the orchestrator, ensure you run ALL the same commands that are in the CI/CD pipeline. This ensures your changes will pass GitHub Actions.

## Required Commands (in order)

1. **Run all tests in verbose mode**
   ```bash
   cargo test --verbose
   ```

2. **Run clippy with all targets and features**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Check code formatting**
   ```bash
   cargo fmt --all -- --check
   ```
   
   If formatting issues are found, fix them with:
   ```bash
   cargo fmt --all
   ```

4. **Build the release binary** (optional, but recommended)
   ```bash
   cargo build --release --package orchestrator-core
   ```

## Quick Script

Save this as `check.sh` in the orchestrator directory:

```bash
#!/bin/bash
set -e

echo "🧪 Running tests..."
cargo test --verbose

echo "📋 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🎨 Checking formatting..."
cargo fmt --all -- --check

echo "✅ All checks passed!"
```

Make it executable: `chmod +x check.sh`

## Common Issues

- **Clippy failures**: The most common are:
  - Format string interpolation: Use `format!("{var}")` instead of `format!("{}", var)`
  - Collapsible if statements: Combine nested conditions
  - Disallowed macros: `println!` is not allowed except in interactive modules
  
- **Format failures**: Run `cargo fmt --all` to auto-fix

- **Test failures**: Check recent changes and ensure all mocks are properly set up

## Notes

- The GitHub Actions workflow runs these exact commands - if they pass locally, they'll pass in CI
- Always run these checks before pushing to avoid failed builds
- Consider setting up a git pre-commit hook to run these automatically