# Task 1: Research AI Development Tools Landscape - Toolman Guide

## Overview
This guide explains how to use the selected tools for implementing Task 1, which focuses on web research of AI development tools and local report generation. The tool selection prioritizes efficient web search capabilities combined with robust file system operations.

## Core Tools

### 1. brave_web_search (Remote Tool)
**Purpose**: Primary research tool for gathering information about AI development tools

**When to Use**:
- Initial searches for each AI development tool
- Gathering pricing and feature information
- Finding user reviews and comparisons
- Researching market trends and adoption

**How to Use**:
```javascript
// Search for general information about a tool
brave_web_search({
  query: "GitHub Copilot features pricing 2024",
  count: 10,  // Limit results to manage API usage
  freshness: "py"  // Prefer recent results (past year)
})

// Search for comparisons
brave_web_search({
  query: "Cursor vs GitHub Copilot vs Claude Code comparison",
  count: 15
})

// Search for user feedback
brave_web_search({
  query: "Claude Code user reviews developer experience",
  count: 10
})
```

**Arguments**:
- `query` (required): Search query string
- `count`: Number of results (max 20, default 10)
- `offset`: Pagination offset for additional results
- `freshness`: Time filter (pd=past day, pw=past week, pm=past month, py=past year)
- `text_decorations`: Include text formatting (default true)

### 2. brave_local_search (Remote Tool)
**Purpose**: Supplementary tool for finding local businesses or services related to AI development tools

**When to Use**:
- Searching for local AI tool training or consulting services
- Finding regional pricing or availability information
- Discovering location-specific tool implementations

**How to Use**:
```javascript
brave_local_search({
  query: "AI coding assistant training",
  count: 5
})
```

## Supporting Tools

### 3. create_directory (Local Tool)
**Purpose**: Set up the file system structure for research outputs

**When to Use**:
- At the beginning of the task to create research directories
- When organizing different types of output files

**How to Use**:
```javascript
create_directory({
  path: "research/raw-data"
})
```

### 4. write_file (Local Tool)
**Purpose**: Save research findings and generate reports

**When to Use**:
- Saving the main markdown report
- Storing raw search results
- Creating structured JSON data files

**How to Use**:
```javascript
// Save markdown report
write_file({
  path: "research/ai-tools-landscape.md",
  contents: markdownContent
})

// Save JSON data
write_file({
  path: "research/raw-data/tool-profiles.json",
  contents: JSON.stringify(toolData, null, 2)
})
```

### 5. read_text_file (Local Tool)
**Purpose**: Read existing files for reference or updating

**When to Use**:
- Reading previous research data
- Checking existing reports before updating
- Loading configuration or template files

**How to Use**:
```javascript
read_text_file({
  path: "research/ai-tools-landscape.md",
  head: 50  // Read first 50 lines for quick preview
})
```

### 6. list_directory (Local Tool)
**Purpose**: Verify file creation and directory structure

**When to Use**:
- Confirming all output files are created
- Checking directory organization
- Validating task completion

**How to Use**:
```javascript
list_directory({
  path: "research"
})
```

## Implementation Flow

### Phase 1: Setup
1. Use `create_directory` to establish file structure:
   ```
   research/
   ├── raw-data/
   └── summaries/
   ```

### Phase 2: Research
1. Use `brave_web_search` for each AI tool:
   - Start with general searches
   - Follow up with specific feature/pricing queries
   - Gather comparison data
2. Limit searches to 10-15 results per query to manage tokens

### Phase 3: Data Processing
1. Parse search results in memory
2. Structure data into JSON format
3. Use `write_file` to save:
   - Raw search results
   - Structured tool profiles
   - Intermediate findings

### Phase 4: Report Generation
1. Synthesize findings into markdown format
2. Use `write_file` to create final report
3. Use `list_directory` to verify all outputs

## Best Practices

### Search Optimization
- Use specific, targeted queries
- Combine multiple concepts in single searches
- Leverage freshness parameter for recent data
- Batch similar searches to reduce API calls

### File Management
- Create directory structure before writing files
- Use descriptive file names
- Save data incrementally to prevent loss
- Verify file creation with list_directory

### Error Handling
- Check search result quality before processing
- Validate JSON structure before saving
- Have fallback strategies for failed searches
- Log errors to a separate file

## Common Patterns

### Research Pattern
```javascript
// For each tool
const tools = ["GitHub Copilot", "Cursor", "Claude Code", "Codeium", "Tabnine"];

for (const tool of tools) {
  // General search
  const general = await brave_web_search({
    query: `${tool} AI coding assistant features 2024`,
    count: 10
  });
  
  // Pricing search
  const pricing = await brave_web_search({
    query: `${tool} pricing plans subscription cost`,
    count: 5
  });
  
  // Process and save
  const toolData = processSearchResults(general, pricing);
  await write_file({
    path: `research/raw-data/${tool.toLowerCase().replace(' ', '-')}.json`,
    contents: JSON.stringify(toolData)
  });
}
```

### Report Generation Pattern
```javascript
// Read all tool data
const toolFiles = await list_directory({ path: "research/raw-data" });

// Generate report sections
let report = "# AI Development Tools Landscape\n\n";
report += generateExecutiveSummary(allTools);
report += generateToolAnalysis(allTools);
report += generateComparativeMatrix(allTools);

// Save final report
await write_file({
  path: "research/ai-tools-landscape.md",
  contents: report
});
```

## Troubleshooting

### Issue: Search returns too many irrelevant results
**Solution**: Refine query with more specific terms and use quotes for exact phrases

### Issue: File write permission denied
**Solution**: Ensure directory exists first using create_directory

### Issue: JSON parsing errors
**Solution**: Validate data structure before stringifying and use try-catch blocks

### Issue: Exceeding API rate limits
**Solution**: Implement delays between searches and batch similar queries