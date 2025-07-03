# Git Hooks

This directory contains git hooks to help prevent accidental commits of secrets and sensitive data.

## Setup

The repository is already configured to use these hooks. If you've just cloned the repo, run:

```bash
git config core.hooksPath .githooks
```

## Hooks

### pre-commit

The pre-commit hook scans for potential secrets before allowing commits, including:

- JWT tokens
- AWS credentials
- GitHub tokens
- API keys
- Private keys
- Password patterns
- Common secret file names

## Bypassing Hooks

If you're certain a detection is a false positive, you can bypass the hook with:

```bash
git commit --no-verify
```

**⚠️ Use with caution!** Only bypass if you're absolutely sure there are no secrets.

## Testing the Hook

To test if the hook is working:

```bash
echo "test_api_key=sk_test_1234567890abcdef1234567890abcdef" > test.txt
git add test.txt
git commit -m "Test commit"
# This should be blocked
rm test.txt
```

## Adding New Patterns

To add new secret patterns, edit `.githooks/pre-commit` and add patterns to the `SECRET_PATTERNS` array.

## Troubleshooting

If the hook isn't running:
1. Ensure it's executable: `chmod +x .githooks/pre-commit`
2. Verify git config: `git config core.hooksPath` should return `.githooks`
3. Check that you're in the repository root when committing