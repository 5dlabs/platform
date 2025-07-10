# Task Submission File Hierarchy Update

## Current Behavior
The CLI currently checks if a task-specific docs directory exists and uses either all task-specific files or all root files.

## Desired Behavior
Implement a hierarchical structure where:

1. **Always from root** (project-wide defaults):
   - `CLAUDE.md` - General Claude context and instructions
   - `git-guidelines.md` - Git workflow rules that apply to all tasks

2. **Task-specific with root fallback**:
   - `design-spec.md` - Check task directory first, use root if not found

3. **Always task-specific** (no fallback):
   - `task.md` - Task description
   - `prompt.md` - Task implementation prompt
   - `acceptance-criteria.md` - Task acceptance criteria

## Implementation Changes

Replace the current logic in `orchestrator-cli/src/commands.rs` around lines 45-195:

```rust
// Check for task-specific docs directory
let task_docs_dir = Path::new(taskmaster_dir).join("docs").join(format!("task-{task_id}"));
let root_docs_dir = Path::new(taskmaster_dir).join("docs");

// Prepare markdown files
let mut markdown_files = vec![];

// 1. Task-specific files (required to be in task directory)
// task.md - Always from task directory or generated
let task_md_path = task_docs_dir.join("task.md");
if task_md_path.exists() {
    let task_md = fs::read_to_string(&task_md_path).with_context(|| {
        format!("Failed to read task.md: {}", task_md_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: task_md,
        filename: "task.md".to_string(),
        file_type: "task".to_string(),
    });
} else {
    // Generate from JSON if no task.md exists
    markdown_files.push(MarkdownPayload {
        content: task_to_markdown(&task),
        filename: "task.md".to_string(),
        file_type: "task".to_string(),
    });
}

// prompt.md - Task-specific only
let prompt_path = task_docs_dir.join("prompt.md");
if prompt_path.exists() {
    let prompt = fs::read_to_string(&prompt_path)
        .with_context(|| format!("Failed to read prompt: {}", prompt_path.display()))?;
    markdown_files.push(MarkdownPayload {
        content: prompt,
        filename: "prompt.md".to_string(),
        file_type: "prompt".to_string(),
    });
}

// acceptance-criteria.md - Task-specific only
let acceptance_criteria_path = task_docs_dir.join("acceptance-criteria.md");
if acceptance_criteria_path.exists() {
    let criteria = fs::read_to_string(&acceptance_criteria_path).with_context(|| {
        format!("Failed to read acceptance criteria: {}", acceptance_criteria_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: criteria,
        filename: "acceptance-criteria.md".to_string(),
        file_type: "acceptance-criteria".to_string(),
    });
}

// 2. Files with fallback (check task-specific first, then root)
// design-spec.md - Task-specific with root fallback
let task_design_spec_path = task_docs_dir.join("design-spec.md");
let root_design_spec_path = root_docs_dir.join("design-spec.md");

if task_design_spec_path.exists() {
    let design_spec = fs::read_to_string(&task_design_spec_path).with_context(|| {
        format!("Failed to read task-specific design spec: {}", task_design_spec_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: design_spec,
        filename: "design-spec.md".to_string(),
        file_type: "design-spec".to_string(),
    });
    output.info(&format!("Using task-specific design-spec.md from: {}", task_design_spec_path.display()))?;
} else if root_design_spec_path.exists() {
    let design_spec = fs::read_to_string(&root_design_spec_path).with_context(|| {
        format!("Failed to read root design spec: {}", root_design_spec_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: design_spec,
        filename: "design-spec.md".to_string(),
        file_type: "design-spec".to_string(),
    });
    output.info("Using root-level design-spec.md")?;
}

// 3. Always from root (project-wide files)
// CLAUDE.md - Always from root
let claude_md_path = root_docs_dir.join("CLAUDE.md");
if claude_md_path.exists() {
    let claude_md = fs::read_to_string(&claude_md_path).with_context(|| {
        format!("Failed to read CLAUDE.md: {}", claude_md_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: claude_md,
        filename: "CLAUDE.md".to_string(),
        file_type: "claude".to_string(),
    });
}

// git-guidelines.md - Always from root
let git_guidelines_path = root_docs_dir.join("git-guidelines.md");
if git_guidelines_path.exists() {
    let git_guidelines = fs::read_to_string(&git_guidelines_path).with_context(|| {
        format!("Failed to read git-guidelines.md: {}", git_guidelines_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: git_guidelines,
        filename: "git-guidelines.md".to_string(),
        file_type: "context".to_string(),
    });
}

// regression-testing.md - From root if exists (optional)
let regression_testing_path = root_docs_dir.join("regression-testing.md");
if regression_testing_path.exists() {
    let regression_guide = fs::read_to_string(&regression_testing_path).with_context(|| {
        format!("Failed to read regression testing guide: {}", regression_testing_path.display())
    })?;
    markdown_files.push(MarkdownPayload {
        content: regression_guide,
        filename: "regression-testing.md".to_string(),
        file_type: "context".to_string(),
    });
}
```

## Benefits

1. **Reduces duplication**: No need to copy CLAUDE.md and git-guidelines.md to every task directory
2. **Allows customization**: Tasks can override design-spec.md if needed
3. **Clear hierarchy**: Easy to understand which files come from where
4. **Flexible**: Can add more files to any category as needed

## Future Enhancements

Could add more files to the hierarchy:
- `testing-strategy.md` - Could be root with task-specific override
- `architecture.md` - Usually root-level
- `dependencies.md` - Could be task-specific