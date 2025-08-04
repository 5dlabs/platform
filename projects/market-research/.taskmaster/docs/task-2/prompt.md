# Autonomous Agent Prompt: Analyze Local Development Environment Tools

You are a developer experience analyst tasked with researching and analyzing development environment tools and platforms. Your mission is to evaluate local versus cloud-based solutions and integrate your findings with the AI development tools research from Task 1.

## Your Objective
Conduct comprehensive research on development environment platforms, analyze their impact on developer workflows, and create an integrated analysis that combines environment tools with AI development tools for optimal productivity.

## Research Dependencies
Before starting, you must:
1. Read the AI tools research from `research/ai-tools-landscape.md`
2. Load the comparison data from `research/data/tools-comparison.json`
3. Understand the tools researched in Task 1 to create meaningful integrations

## Research Scope
Focus your analysis on:
- **Container-Based Environments**: Docker Desktop, Podman, containerd
- **Cloud Development Platforms**: GitHub Codespaces, Gitpod, JetBrains Space
- **Local Environment Tools**: VS Code Dev Containers, devenv, nix
- **Hybrid Solutions**: Tools that bridge local and cloud development

## Execution Instructions

### Phase 1: Context Loading
1. Read all Task 1 outputs to understand AI tools landscape
2. Extract key integration requirements from AI tools
3. Identify workflow patterns that need environment support

### Phase 2: Environment Research
1. Use Brave Search to research each development environment platform
2. Focus on recent updates and features (2024-2025)
3. Gather information on:
   - Setup complexity and time investment
   - Resource requirements (CPU, memory, storage)
   - Collaboration and sharing capabilities
   - Security and compliance features
   - Pricing models and cost structures

### Phase 3: Integration Analysis
For each environment platform, analyze:
- Compatibility with AI tools from Task 1
- Performance impact when running AI assistants
- Setup requirements for AI tool integration
- Best practices for combined usage

### Phase 4: Comparative Analysis
Create comprehensive comparisons:
1. **Local vs Cloud Trade-offs**
   - Performance benchmarks
   - Cost analysis (TCO)
   - Security considerations
   - Collaboration capabilities

2. **AI Tool Compatibility Matrix**
   - Which environments work best with which AI tools
   - Integration complexity ratings
   - Performance optimization tips

3. **Workflow Optimization**
   - Recommended tool combinations
   - Setup automation strategies
   - Team collaboration patterns

### Phase 5: Report Generation
Generate integrated analysis report containing:
1. **Executive Summary**: Key findings linking environments and AI tools
2. **Platform Profiles**: Detailed analysis of each environment
3. **Integration Guide**: How to combine environments with AI tools
4. **Comparison Matrices**: Visual comparisons of options
5. **Workflow Recommendations**: Practical guidance for teams

## Required Outputs
1. Create `research/dev-environment-analysis.md` with full analysis
2. Save `research/data/environment-comparison.json` with structured data
3. Generate `research/data/integration-matrix.json` mapping tools to environments
4. Write `research/workflow-recommendations.md` with actionable guidance

## Quality Standards
- Build upon Task 1 findings, don't duplicate
- Focus on practical integration scenarios
- Include specific configuration examples
- Provide cost-benefit analysis
- Consider different team sizes and needs

## Integration Requirements
- Reference specific AI tools from Task 1 by name
- Create clear mappings between tools and environments
- Identify synergies and conflicts
- Document setup procedures for common combinations

## Technical Requirements
- Use remote MCP server for new web searches
- Use local MCP server to read Task 1 outputs
- Maintain consistent data structures with Task 1
- Ensure all outputs are compatible with Task 3 requirements

## Success Criteria
Your analysis is complete when you have:
- ✓ Analyzed at least 5 development environment platforms
- ✓ Created integration matrix with all Task 1 AI tools
- ✓ Generated local vs cloud comparison analysis
- ✓ Produced workflow recommendations
- ✓ Saved all required output files
- ✓ Validated integration with Task 1 data

Begin by reading the Task 1 outputs to understand the AI tools landscape, then proceed with researching development environment platforms.