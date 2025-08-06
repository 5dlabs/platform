# Project Intake Workflow

## Overview

The Project Intake workflow automates the process of converting project requirements into TaskMaster tasks. It reads a PRD from your local `intake/` folder and creates a pull request with a complete TaskMaster project structure.

## Quick Start

### 1. Create Your Intake Files

In any repository, create an `intake` folder with your requirements:

```bash
# Navigate to your repository
cd ~/projects/my-repo

# Create intake folder
mkdir intake

# Add your PRD
cat > intake/prd.txt << 'EOF'
Project: Customer Support System

## Overview
Build an AI-powered customer support system.

## Requirements
- REST API for ticket management
- Integration with Claude for responses
- PostgreSQL database
- Real-time updates via WebSockets
- Admin dashboard

## Technical Stack
- Node.js/TypeScript
- PostgreSQL
- Redis for caching
EOF

# Optional: Add architecture details
cat > intake/architecture.md << 'EOF'
# Architecture Overview

## Services
- API Service (Node.js)
- Database (PostgreSQL)
- Cache (Redis)
- WebSocket Server

## Deployment
- Kubernetes
- GitHub Actions CI/CD
EOF
```

### 2. Run the Intake Tool

```bash
# Simple usage - just provide a project name
cto intake --project-name="customer-support"

# The tool will:
# - Read intake/prd.txt
# - Read intake/architecture.md (if exists)
# - Auto-detect your repository from git
# - Generate TaskMaster tasks
# - Create a PR in the same repository
```

## How It Works

### Automatic Detection

The intake tool automatically:
- **Reads files** from `./intake/` folder
- **Detects repository** from your git remote
- **Uses current branch** as the base
- **Creates project** with the name you provide

### File Structure Created

```
customer-support/
├── .taskmaster/
│   ├── config.json           # TaskMaster configuration
│   ├── tasks/
│   │   └── tasks.json        # Generated tasks
│   └── docs/
│       ├── task-1/           # Individual task documentation
│       │   ├── prompt.md
│       │   └── acceptance-criteria.md
│       ├── task-2/
│       └── ...
├── README.md                 # Project documentation
└── package.json             # If Node.js project
```

### The Pull Request

The workflow creates a PR with:
- ✅ Complete TaskMaster structure
- ✅ All generated tasks
- ✅ Documentation for each task
- ✅ Ready for implementation

## Simplified Workflow

### Your Only Requirements:

1. **Create intake folder** in your repo
2. **Add prd.txt** with requirements  
3. **Run intake command** with project name
4. **Review PR** and merge

That's it! No need to specify:
- ❌ Repository (auto-detected)
- ❌ Branch (uses current)
- ❌ Project path (uses project name)
- ❌ File paths (always uses intake/)

## Examples

### Basic Usage
```bash
# Minimal command - just the project name
cto intake --project-name="payment-api"
```

### With Options
```bash
# Customize task generation
cto intake --project-name="analytics" \
          --num-tasks=30 \
          --model="claude-opus"
```

### Override Repository
```bash
# Target a different repository (rare case)
cto intake --project-name="mobile-app" \
          --repository="5dlabs/mobile"
```

## File Requirements

### intake/prd.txt (Required)
- Plain text or markdown format
- Should include:
  - Project overview
  - Requirements
  - Technical constraints
  - Success criteria

### intake/architecture.md (Optional)
- Technical architecture details
- System design decisions
- Infrastructure requirements
- Integration points

## Workflow Steps

1. **You**: Create `intake/prd.txt`
2. **Tool**: Reads local files
3. **Tool**: Submits to Argo workflow
4. **Argo**: Processes PRD with Claude
5. **Argo**: Generates TaskMaster structure
6. **Argo**: Creates feature branch
7. **Argo**: Opens pull request
8. **You**: Review and merge PR

## Benefits

- **Simple**: Just two files and one command
- **Local**: Files stay in your repo
- **Automatic**: Repository detection
- **Integrated**: Uses existing K8s infrastructure
- **Consistent**: Same process every time

## Cleanup

After the PR is created, you can delete the intake files:

```bash
# Manual cleanup (if desired)
rm -rf intake/
```

The files are only needed during intake processing, not for ongoing development.

## Troubleshooting

### "No PRD found"
- Ensure `intake/prd.txt` exists in current directory
- Check you're in the repository root

### "Failed to get git repository"
- Ensure you're in a git repository
- Check `git remote -v` shows origin

### "Project name is required"
- Always provide `--project-name` parameter

### Workflow Status
```bash
# Check workflow status
argo get @latest -n agent-platform

# View logs
argo logs @latest -n agent-platform
```

## Advanced Features

While the simplified workflow covers most cases, you can still:
- Override PRD content with `--prd-content`
- Specify different repository with `--repository`
- Control task generation with `--num-tasks`
- Disable complexity analysis with `--analyze-complexity=false`

But for most projects, you just need:
```bash
mkdir intake
echo "Your requirements..." > intake/prd.txt
cto intake --project-name="your-project"
```
