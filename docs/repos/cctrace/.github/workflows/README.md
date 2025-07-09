# GitHub Actions Workflows

This directory contains GitHub Actions workflows for the Claude Code Session Export tool.

## Workflows

### 1. Claude Assistant (`claude-assistant.yml`)
**Trigger**: When someone mentions @claude in issues or PRs
**Purpose**: Interactive AI assistance for users and contributors
- Answers questions about the export tool
- Helps troubleshoot issues
- Reviews pull requests
- Suggests improvements

### 2. Claude Automation (`claude-automation.yml`)
**Trigger**: Push to main, weekly schedule, or manual
**Purpose**: Automated tasks using Claude
- Updates documentation weekly
- Tests export functionality on each push
- Reviews code changes in PRs
- Performs automated testing

### 3. CI (`ci.yml`)
**Trigger**: Push or PR to main branch
**Purpose**: Basic continuous integration
- Tests Python syntax across multiple versions (3.7-3.12)
- Verifies script permissions
- Checks directory structure
- No Claude API required

## Setup

1. **Add API Key**:
   ```bash
   # Go to your repository settings > Secrets and variables > Actions
   # Add a new secret named ANTHROPIC_API_KEY with your API key
   ```

2. **Enable Actions**:
   - Go to Settings > Actions > General
   - Enable "Allow all actions and reusable workflows"

3. **Permissions**:
   - The Claude actions need write permissions for contents, issues, and PRs
   - These are configured in each workflow file

## Usage

### Getting Help from Claude
In any issue or PR, mention @claude with your question:
```
@claude How do I export sessions older than 24 hours?
```

### Manual Workflow Runs
Go to Actions tab > Select workflow > Run workflow

### Customization
- Edit `system_prompt` in workflows to customize Claude's behavior
- Adjust `allowed_tools` to control what Claude can do
- Modify triggers in the `on:` section

## Security Notes
- API keys are stored as GitHub secrets
- Claude only has access to tools you explicitly allow
- Review workflow permissions before enabling