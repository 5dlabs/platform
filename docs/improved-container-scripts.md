# Improved Container Scripts

## Init Container Script Improvements

```rust
fn build_init_script(tr: &TaskRun, _config: &ControllerConfig) -> String {
    let task_id = tr.spec.task_id;
    let mut script = String::new();

    // Clear container identification
    script.push_str("\n");
    script.push_str("echo '════════════════════════════════════════════════════════════════'\n");
    script.push_str("echo '║                    INIT CONTAINER STARTING                   ║'\n");
    script.push_str("echo '║                   (prepare-workspace)                        ║'\n");
    script.push_str("echo '════════════════════════════════════════════════════════════════'\n");
    script.push_str("\n");

    // Check tools
    script.push_str("echo '✓ Using Claude Code image with pre-installed tools'\n");
    
    // Create workspace directory
    script.push_str(&format!("mkdir -p /workspace/.task/{task_id}\n"));

    // Clone repository if specified and if not already present
    if let Some(repo) = &tr.spec.repository {
        script.push_str(&format!("echo 'Repository: {}'\n", repo.url));
        
        // Smart repository management - only clone if no .git exists
        script.push_str("cd /workspace\n");
        script.push_str("if [ ! -d '.git' ]; then\n");
        script.push_str("  echo 'No git repository found, cloning...'\n");
        script.push_str(&format!("  git clone --branch {} {} .\n", repo.branch, repo.url));
        script.push_str("else\n");
        script.push_str("  echo 'Git repository exists, pulling latest changes...'\n");
        script.push_str(&format!("  git fetch origin {}\n", repo.branch));
        script.push_str(&format!("  git checkout {}\n", repo.branch));
        script.push_str(&format!("  git pull origin {}\n", repo.branch));
        script.push_str("fi\n");
    }

    // Copy task files
    script.push_str(&format!("cp /config/* /workspace/.task/{task_id}/\n"));
    script.push_str(&format!("cp /workspace/.task/{task_id}/*.md /workspace/\n"));

    // Setup Claude configuration
    script.push_str("mkdir -p /workspace/.claude\n");
    script.push_str("cp /config/.claude.json /workspace/.claude.json\n");

    // Summary
    script.push_str("\n");
    script.push_str("echo '════════════════════════════════════════════════════════════════'\n");
    script.push_str("echo '║                   INIT CONTAINER COMPLETE                    ║'\n");
    script.push_str("echo '════════════════════════════════════════════════════════════════'\n");
    script.push_str("\n");

    script
}
```

## Main Container Script Improvements

```rust
fn build_agent_startup_script(tr: &TaskRun, config: &ControllerConfig) -> String {
    let mut script = String::new();

    // Clear container identification
    script.push_str("\n");
    script.push_str("echo '════════════════════════════════════════════════════════════════'\n");
    script.push_str("echo '║                    MAIN CONTAINER STARTING                   ║'\n");
    script.push_str("echo '║                      (claude-agent)                          ║'\n");
    script.push_str("echo '════════════════════════════════════════════════════════════════'\n");
    script.push_str("\n");

    // Configure git if needed
    script.push_str("if [ -f /workspace/.github-env ]; then\n");
    script.push_str("  . /workspace/.github-env\n");
    script.push_str("  echo '✓ GitHub authentication configured'\n");
    script.push_str("fi\n\n");

    // Quick status check
    script.push_str("echo '──────────────────────────────────────────────────────────────'\n");
    script.push_str("echo 'Environment:'\n");
    script.push_str("echo \"  • Working directory: $(pwd)\"\n");
    script.push_str(&format!("echo \"  • Task ID: {}\"\n", tr.spec.task_id));
    script.push_str("echo \"  • Git branch: $(git branch --show-current 2>/dev/null || echo 'N/A')\"\n");
    script.push_str("echo '──────────────────────────────────────────────────────────────'\n");
    script.push_str("\n");

    // Start Claude with minimal output
    let command = config.agent.command.join(" ");
    script.push_str(&format!("echo 'Starting Claude Code v$({command} --version 2>/dev/null)...'\n"));
    script.push_str("\n");

    // Add the actual Claude command
    let args = config.agent.args.join(" ");
    let model_arg = if let Some(model_mapping) = &config.agent.model_mapping {
        // ... model mapping logic ...
    }
    
    script.push_str(&format!("{command} {model_arg} {args}\n"));

    script
}
```

## Key Improvements

1. **Clear Container Separation**
   - Big banner headers for INIT and MAIN containers
   - Clear "complete" messages

2. **Reduced Debug Output**
   - Removed directory tree dumps
   - Removed environment variable dumps
   - Removed settings.json searches
   - Removed DEVCONTAINER logic

3. **Smart Repository Management**
   - Only clone if no .git directory exists
   - Otherwise just pull latest changes
   - Prevents data loss on subsequent runs

4. **Minimal Status Info**
   - Just the essentials: working dir, task ID, git branch
   - Single line Claude version
   - Clean, readable output

5. **Simplified Flow**
   - Init: Setup workspace → Clone/pull repo → Copy files → Done
   - Main: Configure auth → Quick status → Start Claude