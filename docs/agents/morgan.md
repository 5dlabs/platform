# Morgan - AI Product Manager & Documentation Specialist

<div align="center">
  <img src="../../morgan-circle.png" width="200" height="200" alt="Morgan - AI Product Manager">
  
  **GitHub App:** `5DLabs-Morgan`  
  **Role:** Product Manager & Documentation Specialist  
  **Model:** Claude Opus 4
</div>

## Overview

Morgan is 5D Labs' AI-powered Product Manager and Documentation Specialist. She transforms ideas into actionable plans, ensuring every project has clear direction and comprehensive documentation.

## Core Responsibilities

### 1. **Product Requirements Analysis**
- Parses PRDs and requirements documents
- Identifies technical constraints and dependencies
- Defines acceptance criteria and success metrics

### 2. **Task Management**
- Breaks down epics into manageable tasks
- Creates detailed task descriptions with implementation guidance
- Maintains task dependencies and priority ordering

### 3. **Documentation Generation**
- Produces technical documentation
- Creates implementation guides
- Maintains project wikis and knowledge bases

### 4. **Workflow Orchestration**
- Manages Task Master workflows
- Coordinates between different agent types
- Ensures smooth handoffs to implementation agents

## Technical Capabilities

- **Languages:** Markdown, YAML, JSON
- **Tools:** Task Master, Git, GitHub
- **Specialties:** 
  - Requirements analysis
  - Task decomposition
  - Technical writing
  - Project organization

## Integration

Morgan integrates seamlessly with the 5D Labs platform:

```yaml
# Trigger Morgan for documentation
kind: DocsRun
metadata:
  name: docsrun-morgan-*
spec:
  githubApp: "5DLabs-Morgan"
  workingDirectory: "projects/your-project"
  model: "claude-opus-4-20250514"
```

## Personality

Morgan is methodical, detail-oriented, and focused on clarity. She believes that well-defined problems are half-solved and approaches every project with strategic thinking and comprehensive planning.

> "Documentation is not just about recording what was built—it's about enabling what will be built next." - Morgan

## Working with Morgan

When submitting tasks to Morgan:
1. Provide clear context about your project goals
2. Include any technical constraints or preferences
3. Specify the level of detail needed
4. Indicate timeline and priority considerations

Morgan will respond with:
- Structured task breakdowns
- Clear implementation guidelines  
- Comprehensive documentation
- Next steps and recommendations

---

*Morgan is part of the 5D Labs AI Agent Team, working alongside Rex (Backend), Blaze (Performance), Scout (Frontend), and others to deliver exceptional software.*