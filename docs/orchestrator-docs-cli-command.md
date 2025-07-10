# Orchestrator Documentation Generation CLI Command

## Current Command Syntax

The basic command to generate documentation for Task Master tasks is:

```bash
orchestrator task init-docs
```

That's it! No environment variables or additional configuration needed for the default setup.

## Full Command with All Options

```bash
orchestrator task init-docs \
  --taskmaster-dir .taskmaster \     # Path to Task Master directory (default: .taskmaster)
  --model opus \                      # Claude model to use (default: opus)
  --repo <URL> \                      # Repository URL (auto-detected if not specified)
  --source-branch <BRANCH> \          # Source branch (default: main)
  --target-branch <BRANCH> \          # Target branch for PR (defaults to source branch)
  --working-dir <DIR> \               # Working directory (auto-detected from current location)
  --force \                           # Overwrite existing documentation
  --task-id <ID> \                    # Generate docs for specific task only
  --update \                          # Update existing docs
  --update-all \                      # Force update all docs
  --dry-run \                         # Preview without creating files
  --verbose                           # Show detailed progress
```

## API URL Configuration

The API URL now defaults to: `http://orchestrator.orchestrator.svc.cluster.local/api/v1`

You only need to set the environment variable if you're using a different orchestrator endpoint:

```bash
export ORCHESTRATOR_API_URL="http://your-custom-orchestrator/api/v1"
```

## Example Usage

### Generate docs for all tasks (most common usage)
```bash
cd /path/to/your/project
orchestrator task init-docs
```

### Generate docs from a specific directory
```bash
cd /path/to/your/project/subdirectory
orchestrator task init-docs --source-branch feature/my-branch
```

### Generate docs for a specific task
```bash
orchestrator task init-docs --task-id 11
```

### Dry run to preview what would be generated
```bash
orchestrator task init-docs --dry-run
```

## Key Features

1. **Auto-detection of Working Directory**: The CLI automatically detects the current working directory relative to the git repository root. You no longer need to specify `--working-dir` unless you want to override the auto-detection.

2. **Default Model is Opus**: The documentation generation defaults to using the Opus model for better quality documentation. You can still override this with `--model sonnet` if needed.

3. **Repository URL Auto-detection**: The CLI automatically detects the git repository URL from the current git remote. You only need to specify `--repo` if you want to override this.

4. **Current Branch Auto-detection**: The CLI automatically detects your current git branch and uses it as the source branch. The target branch defaults to the same as the source branch unless overridden.

5. **Auto-commit Task Master Changes**: If there are uncommitted changes in the `.taskmaster/` directory, the CLI will automatically:
   - Stage all changes in `.taskmaster/`
   - Commit them with message: "chore: auto-commit Task Master changes before docs generation"
   - Push the changes to the remote repository
   - Then proceed with the documentation generation

## Monitoring the Job

After submitting the documentation generation job, you can monitor its progress with:

```bash
kubectl -n orchestrator get taskrun
kubectl -n orchestrator logs -f job/<job-name>
```

The job will:
1. Clone the repository and checkout the specified branch
2. Read the tasks.json file from the Task Master directory
3. Generate comprehensive documentation for each task
4. Create a new branch with the documentation
5. Push the branch and create a pull request