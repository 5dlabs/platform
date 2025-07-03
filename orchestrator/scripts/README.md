# Orchestrator Scripts

## CI Checks Script

The `ci-checks.sh` script runs all the same checks that will be performed in GitHub Actions CI. This allows you to verify your code will pass CI before pushing.

### Usage

```bash
./scripts/ci-checks.sh
```

### What it checks

1. **Code Formatting** - Ensures all code is properly formatted with `cargo fmt`
2. **Clippy Linting** - Runs Clippy with strict warnings to catch common mistakes
3. **Build** - Verifies the project builds successfully
4. **Tests** - Runs all unit and integration tests
5. **Documentation** - Ensures documentation builds without errors
6. **Cargo.lock** - Checks for uncommitted changes to dependencies
7. **Security Audit** - Runs cargo-audit to check for known vulnerabilities
8. **TODO/FIXME** - Reports any TODO or FIXME comments in the code

### Exit Codes

- `0` - All checks passed
- `1` - One or more checks failed

### Fixing Issues

If formatting fails:
```bash
cargo fmt --all
```

If Clippy fails:
```bash
cargo clippy --all-targets --all-features
# Review the warnings and fix them
```

### Running Individual Checks

You can also run individual checks manually:

```bash
# Format check only
cargo fmt --all -- --check

# Clippy only
cargo clippy --all-targets --all-features -- -D warnings

# Tests only
cargo test --all-features --all-targets

# Documentation only
cargo doc --no-deps --all-features
```