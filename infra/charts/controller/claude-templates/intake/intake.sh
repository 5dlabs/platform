#!/bin/bash
set -e

echo "🚀 Starting Project Intake Process"
echo "================================="

# Load configuration from mounted ConfigMap
CONFIG_FILE="/intake-files/config.json"
PRD_FILE="/intake-files/prd.txt"
ARCH_FILE="/intake-files/architecture.md"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file not found at $CONFIG_FILE"
    exit 1
fi

# Parse configuration
PROJECT_NAME=$(jq -r '.project_name' "$CONFIG_FILE")
REPOSITORY_URL=$(jq -r '.repository_url' "$CONFIG_FILE")
GITHUB_APP=$(jq -r '.github_app' "$CONFIG_FILE")
MODEL=$(jq -r '.model' "$CONFIG_FILE")
NUM_TASKS=$(jq -r '.num_tasks' "$CONFIG_FILE")
EXPAND_TASKS=$(jq -r '.expand_tasks' "$CONFIG_FILE")
ANALYZE_COMPLEXITY=$(jq -r '.analyze_complexity' "$CONFIG_FILE")

# If project name is empty, try to extract from PRD
if [ -z "$PROJECT_NAME" ] || [ "$PROJECT_NAME" = "null" ]; then
    echo "📝 Extracting project name from PRD..."
    
    # Try to extract from first heading
    PROJECT_NAME=$(head -10 "$PRD_FILE" | grep -E "^#\s+" | head -1 | sed 's/^#\s*//' | \
                   sed 's/[^a-zA-Z0-9 -]//g' | tr '[:upper:]' '[:lower:]' | \
                   sed 's/ /-/g' | sed 's/--*/-/g' | sed 's/^-*//;s/-*$//')
    
    # Fallback to timestamp-based name
    if [ -z "$PROJECT_NAME" ]; then
        PROJECT_NAME="project-$(date +%Y%m%d-%H%M%S)"
    fi
    
    echo "✅ Using project name: $PROJECT_NAME"
fi

# Disable interactive Git prompts
export GIT_TERMINAL_PROMPT=0
export GIT_ASKPASS=/bin/true
export SSH_ASKPASS=/bin/true

# GitHub App authentication setup
if [ -n "$GITHUB_APP_PRIVATE_KEY" ] && [ -n "$GITHUB_APP_ID" ]; then
    echo "🔐 Setting up GitHub App authentication..."
    
    # Function to generate GitHub App token (reusing from container.sh logic)
    generate_github_token() {
        echo "Generating fresh GitHub App token..."
        
        # Create temporary private key file
        TEMP_KEY_FILE="/tmp/github-app-key.pem"
        echo "$GITHUB_APP_PRIVATE_KEY" > "$TEMP_KEY_FILE"
        chmod 600 "$TEMP_KEY_FILE"
        
        # Generate JWT token
        JWT_HEADER=$(printf '{"alg":"RS256","typ":"JWT"}' | base64 -w 0 | tr '+/' '-_' | tr -d '=')
        NOW=$(date +%s)
        EXP=$((NOW + 600))
        JWT_PAYLOAD=$(printf '{"iat":%d,"exp":%d,"iss":"%s"}' "$NOW" "$EXP" "$GITHUB_APP_ID" | base64 -w 0 | tr '+/' '-_' | tr -d '=')
        JWT_SIGNATURE=$(printf '%s.%s' "$JWT_HEADER" "$JWT_PAYLOAD" | openssl dgst -sha256 -sign "$TEMP_KEY_FILE" -binary | base64 -w 0 | tr '+/' '-_' | tr -d '=')
        JWT_TOKEN="$JWT_HEADER.$JWT_PAYLOAD.$JWT_SIGNATURE"
        
        # Get installation ID
        REPO_OWNER=$(echo "$REPOSITORY_URL" | sed -E 's|https://github.com/([^/]+)/.*|\1|')
        REPO_NAME=$(echo "$REPOSITORY_URL" | sed -E 's|https://github.com/[^/]+/([^/]+)(\.git)?|\1|')
        
        INSTALLATION_ID=$(curl -s -H "Authorization: Bearer $JWT_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/installation" | jq -r '.id')
        
        if [ "$INSTALLATION_ID" = "null" ] || [ -z "$INSTALLATION_ID" ]; then
            echo "❌ Failed to get installation ID for $REPO_OWNER/$REPO_NAME"
            return 1
        fi
        
        echo "Installation ID: $INSTALLATION_ID"
        
        # Generate installation access token
        GITHUB_TOKEN=$(curl -s -X POST \
            -H "Authorization: Bearer $JWT_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/app/installations/$INSTALLATION_ID/access_tokens" | jq -r '.token')
        
        if [ "$GITHUB_TOKEN" = "null" ] || [ -z "$GITHUB_TOKEN" ]; then
            echo "❌ Failed to generate GitHub token"
            return 1
        fi
        
        export GITHUB_TOKEN
        export GH_TOKEN="$GITHUB_TOKEN"
        
        # Configure git
        git config --global --replace-all credential.helper store
        echo "https://x-access-token:${GITHUB_TOKEN}@github.com" > ~/.git-credentials
        
        # Configure GitHub CLI
        echo "$GITHUB_TOKEN" | gh auth login --with-token
        gh auth status
        
        echo "✅ GitHub authentication configured"
        return 0
    }
    
    # Initial token generation
    generate_github_token || exit 1
else
    echo "⚠️ GitHub App credentials not found, using default authentication"
fi

# Clone repository
echo "📦 Cloning repository: $REPOSITORY_URL"
CLONE_DIR="/tmp/repo-$(date +%s)"
git clone "$REPOSITORY_URL" "$CLONE_DIR"
cd "$CLONE_DIR"

# Configure git identity
git config user.name "Project Intake Bot"
git config user.email "intake@5dlabs.com"

# Change to project directory
cd "$PROJECT_DIR"

# Install TaskMaster globally
echo "📦 Installing TaskMaster..."
npm install -g task-master-ai@latest

# Initialize TaskMaster
echo "🚀 Initializing TaskMaster project..."
task-master init --yes \
    --name "$PROJECT_NAME" \
    --description "Auto-generated project from intake pipeline" \
    --version "0.1.0" \
    --rules "cursor"

# Copy PRD and architecture files after initialization
echo "📋 Copying PRD and architecture files..."
cp "$PRD_FILE" ".taskmaster/docs/prd.txt"
if [ -f "$ARCH_FILE" ] && [ -s "$ARCH_FILE" ]; then
    cp "$ARCH_FILE" ".taskmaster/docs/architecture.md"
fi

# Configure models
echo "🤖 Configuring AI models..."
task-master models --set-main "$MODEL"
task-master models --set-research "$MODEL"  # Use same model for research to avoid Perplexity requirement
task-master models --set-fallback "claude-3-5-sonnet-20241022"

# Parse PRD
echo "📄 Parsing PRD to generate tasks..."
task-master parse-prd \
    --input ".taskmaster/docs/prd.txt" \
    --force

# Analyze complexity if requested
if [ "$ANALYZE_COMPLEXITY" = "true" ]; then
    echo "🔍 Analyzing task complexity..."
    task-master analyze-complexity
fi

# Expand tasks if requested
if [ "$EXPAND_TASKS" = "true" ]; then
    echo "🌳 Expanding tasks with subtasks..."
    task-master expand --all --force
fi

# Generate task files
echo "📝 Generating individual task files..."
task-master generate

# Create docs directory structure
echo "📁 Creating docs directory structure..."
if [ -d ".taskmaster/tasks" ]; then
    for task_file in .taskmaster/tasks/task-*.txt; do
        if [ -f "$task_file" ]; then
            task_num=$(basename "$task_file" .txt | sed 's/task-//')
            task_dir="docs/task-$task_num"
            mkdir -p "$task_dir"
            cp "$task_file" "$task_dir/task.md"
        fi
    done
fi

# Create summary file
echo "📊 Creating project summary..."
cat > README.md <<EOF
# $PROJECT_NAME

Auto-generated project from intake pipeline.

## Project Structure

- **.taskmaster/** - TaskMaster configuration and tasks
  - **docs/** - Source documents (PRD, architecture)
  - **tasks/** - Generated task definitions
- **docs/** - Individual task documentation

## Getting Started

1. Review the generated tasks in \`.taskmaster/tasks/tasks.json\`
2. Use \`task-master list\` to view all tasks
3. Use \`task-master next\` to get the next task to work on
4. Implement tasks using the orchestrator workflow

## Generated Statistics

- Total tasks: $(jq '.tasks | length' .taskmaster/tasks/tasks.json 2>/dev/null || echo "N/A")
- Model used: $MODEL
- Generated on: $(date)

## Source Documents

- [Product Requirements](/.taskmaster/docs/prd.txt)
$([ -f ".taskmaster/docs/architecture.md" ] && echo "- [Architecture](/.taskmaster/docs/architecture.md)")
EOF

# Commit changes
echo "💾 Committing project structure..."
cd "$CLONE_DIR"
git add -A
git commit -m "🚀 Initialize project: $PROJECT_NAME

Automated project intake:
- Parsed PRD and architecture documents
- Generated TaskMaster tasks
- Created project structure
- Model: $MODEL
- Tasks: $NUM_TASKS targets
$([ "$EXPAND_TASKS" = "true" ] && echo "- Expanded with subtasks")
$([ "$ANALYZE_COMPLEXITY" = "true" ] && echo "- Complexity analysis performed")
"

# Create branch and push
BRANCH_NAME="intake/$PROJECT_NAME-$(date +%Y%m%d-%H%M%S)"
echo "🌿 Creating branch: $BRANCH_NAME"
git checkout -b "$BRANCH_NAME"
git push -u origin "$BRANCH_NAME"

# Create pull request
echo "🔀 Creating pull request..."
PR_BODY="## 🎉 Project Intake: $PROJECT_NAME

This PR contains the auto-generated project structure and tasks.

### 📋 What was processed:
- ✅ PRD document parsed
$([ -f "$PROJECT_DIR/.taskmaster/docs/architecture.md" ] && echo "- ✅ Architecture document included")
- ✅ TaskMaster initialized
- ✅ Tasks generated (target: $NUM_TASKS)
$([ "$ANALYZE_COMPLEXITY" = "true" ] && echo "- ✅ Complexity analysis performed")
$([ "$EXPAND_TASKS" = "true" ] && echo "- ✅ Tasks expanded with subtasks")
- ✅ Project structure created

### 🏗️ Generated Structure:
\`\`\`
$PROJECT_DIR/
├── .taskmaster/
│   ├── docs/
│   │   ├── prd.txt
│   │   └── architecture.md
│   ├── tasks/
│   │   └── tasks.json
│   └── config.json
├── docs/
│   ├── task-1/
│   │   └── task.md
│   └── ...
└── README.md
\`\`\`

### 🤖 Configuration:
- **Model**: $MODEL
- **Tasks Generated**: $(jq '.tasks | length' "$PROJECT_DIR/.taskmaster/tasks/tasks.json" 2>/dev/null || echo "N/A")
- **Complexity Analysis**: $ANALYZE_COMPLEXITY
- **Task Expansion**: $EXPAND_TASKS

### 🎯 Next Steps:
1. Review the generated tasks
2. Merge this PR to add the project
3. Use orchestrator workflows to implement tasks
"

# Refresh GitHub token before PR creation
if [ -n "$GITHUB_APP_PRIVATE_KEY" ]; then
    echo "🔄 Refreshing GitHub token for PR creation..."
    generate_github_token
fi

gh pr create \
    --title "🚀 Project Intake: $PROJECT_NAME" \
    --body "$PR_BODY" \
    --head "$BRANCH_NAME" \
    --base main || {
        echo "⚠️ Failed to create PR, but branch has been pushed"
        echo "Branch: $BRANCH_NAME"
        echo "You can create the PR manually"
    }

echo "✅ Project intake complete!"
echo "================================="
echo "Project: $PROJECT_NAME"
echo "Location: $PROJECT_DIR"
echo "Branch: $BRANCH_NAME"
echo "Repository: $REPOSITORY_URL"
