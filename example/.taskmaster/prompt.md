# Documentation Generation Task

## Repository Information
- **Repository**: https://github.com/5dlabs/agent-platform.git
- **Working Directory**: example
- **Source Branch**: feature/example-project-and-cli
- **Target Branch**: feature/example-project-and-cli

## Task Details
- **Generate docs for**: all tasks
- **Model**: sonnet
- **Force overwrite**: false
- **Dry run**: false

## Instructions

You are tasked with generating comprehensive documentation for Task Master tasks. 

IMPORTANT: You are already in a workspace with access ONLY to the Task Master directory. DO NOT clone any repositories or navigate outside the current directory.

Follow these steps:

1. You are already in the correct directory - no need to clone or navigate
2. Read the `.taskmaster/tasks/tasks.json` file (it's in the current directory)
3. For each task (all tasks), generate the following documentation files in `.taskmaster/docs/task-{id}/`:
   - `task.md`: Comprehensive task overview and implementation guide
   - `prompt.md`: Autonomous prompt for AI agents
   - `design-spec.md`: Technical design specification
   - `acceptance-criteria.md`: Clear acceptance criteria and test cases

4. After generating all documentation:
   - Stage all changes: `git add .`
   - Commit with message: `docs: auto-generate Task Master documentation for all tasks`
   - Push the branch: `git push origin HEAD`
   - Create a PR using: `gh pr create --title "docs: auto-generate Task Master documentation" --body "Auto-generated documentation for Task Master tasks"`

## Important Notes

- Each document should be well-structured, comprehensive, and actionable
- Include code examples, commands, and implementation details where relevant
- Maintain consistency across all generated documents
- Ensure all markdown is properly formatted
- Generate ALL FOUR files (task.md, prompt.md, design-spec.md, acceptance-criteria.md) for EACH task
- Working directory is relative to repository root
