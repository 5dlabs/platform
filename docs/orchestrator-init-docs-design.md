# Orchestrator CLI Enhancement: AI-Powered Task Documentation Generation

## Overview

This design document outlines the addition of an `init-docs` command to the orchestrator CLI that leverages Claude to automatically generate comprehensive, task-specific documentation from Task Master's task definitions.

## Problem Statement

Currently, when submitting tasks to Claude agents:
- Task documentation is manually created and maintained
- Generic documentation covers all tasks, not task-specific
- The orchestrator generates minimal task.md from JSON data
- Rich context and implementation details are missing
- Documentation can become out of sync with task definitions

## Proposed Solution

Add an `orchestrator task init-docs` command that:
1. Reads task definitions from Task Master's `tasks.json`
2. Uses the local Claude CLI (`claude` command) to generate intelligent, contextual documentation
3. Creates a structured folder hierarchy for task-specific docs
4. Integrates seamlessly with the existing task submission workflow

## Design

### Command Interface

```bash
orchestrator task init-docs [OPTIONS]

Options:
  -d, --taskmaster-dir <DIR>     Path to Task Master directory [default: .taskmaster]
  -m, --model <MODEL>            Claude model to use (sonnet, opus) [default: sonnet]
  -f, --force                    Overwrite existing documentation
  -t, --task-id <ID>            Generate docs for specific task only
  -u, --update                   Update existing docs (regenerate from current tasks.json)
  --update-all                   Force update all docs regardless of changes
  --dry-run                      Preview what would be generated without creating files
  -v, --verbose                  Show detailed generation progress
```

### Directory Structure

```
.taskmaster/
├── tasks/
│   └── tasks.json          # Source task definitions
└── docs/
    ├── CLAUDE.md           # Global instructions (unchanged)
    ├── git-guidelines.md   # Global git rules (unchanged)
    ├── task-11/
    │   ├── task.md         # Rich task description
    │   ├── prompt.md       # Implementation instructions
    │   ├── design-spec.md  # Technical design
    │   └── acceptance-criteria.md
    ├── task-12/
    │   └── ...
    └── task-{id}/
        └── ...
```

### Document Generation Strategy

#### 1. task.md - Comprehensive Task Overview
**Purpose**: Provide rich context about what needs to be accomplished

**Claude Prompt Template**:
```
Given this task definition from a Task Master project:
{task_json}

Generate a comprehensive task.md that:
1. Explains the task's purpose and its role in the larger project
2. Provides a narrative overview of what needs to be accomplished
3. Breaks down each subtask with context about why it's important
4. Shows how subtasks relate to and depend on each other
5. Includes relevant commands, code snippets, and examples
6. Maintains a helpful, instructional tone

Format the output as a well-structured Markdown document.
```

#### 2. prompt.md - Step-by-Step Implementation Guide
**Purpose**: Detailed instructions for completing the task

**Claude Prompt Template**:
```
Given this task definition:
{task_json}

Generate a prompt.md with step-by-step implementation instructions that:
1. Provides exact commands to run for each subtask
2. Includes code examples and boilerplate where helpful
3. Explains common pitfalls and how to avoid them
4. Shows how to verify each step is completed correctly
5. Integrates with Task Master commands (set-status, next, etc.)
6. Concludes with instructions for creating a pull request

The target audience is an AI agent (Claude) that will implement these instructions autonomously.
```

#### 3. design-spec.md - Technical Design Document
**Purpose**: Architecture and technical details

**Claude Prompt Template**:
```
Given this task definition:
{task_json}

Generate a design-spec.md that covers:
1. Technical architecture for this component
2. Data models and structures
3. API contracts (if applicable)
4. File structure and organization
5. Integration points with other components
6. Technology choices and rationale
7. Performance and security considerations

Focus on providing clear technical guidance for implementation.
```

#### 4. acceptance-criteria.md - Success Metrics
**Purpose**: Define completion criteria and testing

**Claude Prompt Template**:
```
Given this task definition:
{task_json}

Generate an acceptance-criteria.md that includes:
1. Checklist of completion criteria
2. Specific test cases with expected outcomes
3. Manual testing steps with example commands
4. Automated test requirements (if applicable)
5. Performance benchmarks (if relevant)
6. Documentation requirements
7. Git and PR requirements

Format as actionable checklists and test procedures.
```

### Integration with Task Submission

When submitting a task:
```bash
orchestrator task submit 11 --service user-api
```

The orchestrator will:
1. Look for docs in `.taskmaster/docs/task-11/`
2. If found, use task-specific documentation
3. If not found, fall back to current behavior
4. Include all markdown files from the task folder in the ConfigMap

### Implementation Architecture

```rust
// New module: orchestrator-cli/src/docs_generator.rs
pub struct DocsGenerator {
    taskmaster_path: PathBuf,
    claude_binary: PathBuf, // Path to claude CLI
}

impl DocsGenerator {
    pub async fn generate_all_docs(&self, force: bool) -> Result<()>
    pub async fn generate_task_docs(&self, task_id: u32, force: bool) -> Result<()>
    async fn generate_document(&self, task: &Task, doc_type: DocType) -> Result<String>
    // Uses Command::new(&self.claude_binary) to invoke claude CLI
}

// Update: orchestrator-cli/src/commands.rs
pub async fn init_docs(args: InitDocsArgs) -> Result<()>

// Update: orchestrator-cli/src/commands.rs - submit function
// Check for task-specific docs in .taskmaster/docs/task-{id}/
```

### Claude CLI Integration

The generator will use the local `claude` command via zsh to ensure proper environment loading:

```rust
// Execute through zsh to load profile with ANTHROPIC_API_KEY
let output = Command::new("zsh")
    .arg("-l")  // Login shell to load .zshrc
    .arg("-c")
    .arg("claude -p --no-color")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Write prompt to stdin
let prompt = format!("Given this task definition:\n{}\n\n{}", 
    serde_json::to_string_pretty(&task)?, 
    doc_type.get_prompt()
);
output.stdin.write_all(prompt.as_bytes())?;
```

Benefits of using Claude CLI:
- Uses ANTHROPIC_API_KEY from user's zsh profile
- Consistent with how agents use Claude
- Supports all Claude features (MCP, etc.)
- Proper environment configuration

### Task Submission Flow Changes

**Current Flow:**
1. Orchestrator reads tasks.json
2. Generates minimal task.md from JSON
3. Submits to Claude agent

**New Flow:**
1. Orchestrator reads pre-generated rich markdown from `.taskmaster/docs/task-{id}/`
2. No JSON->markdown conversion needed during submission
3. Claude agent receives comprehensive, context-aware documentation

This means `init-docs` becomes a **prerequisite step** before task submission, handling all JSON->markdown conversion upfront.

### Error Handling

1. **Missing tasks.json**: Clear error with path to expected file
2. **Claude CLI not found**: Check PATH and provide installation instructions
3. **Claude CLI failures**: Capture stderr and provide helpful error messages
4. **Existing files**: Skip unless --force flag is provided
5. **Invalid task structure**: Validate before sending to Claude
6. **Partial generation**: Track which docs were created successfully

### Success Metrics

1. **Generation Quality**: Claude produces actionable, context-aware documentation
2. **Task Coverage**: All tasks get complete documentation sets
3. **Integration**: Seamless usage in task submission workflow
4. **Maintenance**: Easy to regenerate when tasks change
5. **Agent Performance**: Improved success rate for Claude agents

### Future Enhancements

1. **Template Customization**: Allow custom prompt templates
2. **Incremental Updates**: Only regenerate changed tasks
3. **Version Control**: Track documentation versions
4. **Multi-language Support**: Generate docs in different languages
5. **Project Types**: Specialized generation for different project types

## Implementation Plan

### Phase 1: Core Implementation
- Add `init-docs` command structure
- Implement document generation with Claude
- Create file management utilities
- Add progress reporting

### Phase 2: Integration
- Update task submission to use task-specific docs
- Add validation and error handling
- Implement dry-run mode
- Add comprehensive logging

### Phase 3: Testing & Refinement
- Test with various task types
- Refine Claude prompts based on output quality
- Add integration tests
- Update documentation

## Risks and Mitigations

1. **Risk**: Claude CLI may have rate limits or usage restrictions
   - **Mitigation**: Add delays between generations, progress indicators

2. **Risk**: Generated docs may need manual refinement
   - **Mitigation**: Design for easy editing, version control friendly

3. **Risk**: Breaking changes to Task Master format
   - **Mitigation**: Version checking, graceful degradation

4. **Risk**: Claude CLI output format changes
   - **Mitigation**: Parse output robustly, handle different response formats

## Conclusion

This enhancement will significantly improve the orchestrator's ability to provide rich, contextual documentation to Claude agents, leading to better task implementation success rates and reduced manual documentation maintenance.