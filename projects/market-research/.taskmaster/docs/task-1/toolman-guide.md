# Toolman Guide: Research AI Development Tools Landscape

## Overview
This guide explains how to use the selected MCP tools for implementing the AI development tools research task. The task requires web research capabilities and local file operations, which are provided through carefully selected tools.

## Core Tools

### brave_web_search (Remote Tool)
**Server**: brave-search  
**Purpose**: Primary tool for conducting market research on AI development tools

**When to Use**:
- Searching for information about AI development tools and code generation platforms
- Finding current market trends and tool comparisons
- Gathering pricing information and feature lists
- Researching user reviews and adoption metrics

**How to Use**:
```javascript
// Search for AI development tools
const searchResults = await brave_web_search({
  query: "AI code generation tools 2024 comparison",
  count: 20,  // Maximum results per request
  offset: 0,  // For pagination
  freshness: "year"  // Focus on recent content
});

// Search for specific tool information
const toolInfo = await brave_web_search({
  query: "GitHub Copilot pricing features enterprise",
  count: 10
});

// Search for market analysis
const marketData = await brave_web_search({
  query: "developer productivity AI tools market share 2024",
  count: 15
});
```

**Parameters**:
- `query` (required): The search query string
- `count`: Number of results to return (max 20)
- `offset`: Starting position for pagination
- `freshness`: Filter by recency (day, week, month, year)
- `safesearch`: Content filtering level

**Best Practices**:
- Use specific, targeted queries to get relevant results
- Combine multiple searches to gather comprehensive data
- Use freshness parameter to ensure current information
- Process results to extract structured data

## Supporting Tools

### Local File System Tools

#### create_directory
**Purpose**: Create directory structure for organizing research outputs

**When to Use**:
- Setting up the research output directory structure
- Creating subdirectories for different data types

**How to Use**:
```javascript
// Create main research directory
await create_directory({
  path: "/tmp/research"
});

// Create subdirectories for organization
await create_directory({
  path: "/tmp/research/tool-profiles"
});
```

#### write_file
**Purpose**: Save research findings and generated reports

**When to Use**:
- Saving the main markdown report
- Storing raw research data in JSON format
- Creating individual tool profile files

**How to Use**:
```javascript
// Save the main research report
await write_file({
  path: "/tmp/research/ai-tools-landscape.md",
  content: markdownReport
});

// Save structured data
await write_file({
  path: "/tmp/research/raw-data.json",
  content: JSON.stringify(researchData, null, 2)
});
```

#### read_text_file
**Purpose**: Read existing files for reference or continuation

**When to Use**:
- Reading configuration files
- Checking existing research data
- Loading templates or previous reports

**How to Use**:
```javascript
// Read existing data
const existingData = await read_text_file({
  path: "/tmp/research/raw-data.json"
});

// Read with line limits
const summary = await read_text_file({
  path: "/tmp/research/ai-tools-landscape.md",
  head: 50  // Read first 50 lines only
});
```

#### list_directory
**Purpose**: Check existing files and directory structure

**When to Use**:
- Verifying output directory structure
- Checking for existing files before overwriting
- Listing generated artifacts

**How to Use**:
```javascript
// List research directory contents
const files = await list_directory({
  path: "/tmp/research"
});
```

#### get_file_info
**Purpose**: Get metadata about files

**When to Use**:
- Checking file sizes before processing
- Verifying file creation timestamps
- Confirming file permissions

**How to Use**:
```javascript
// Get report file information
const fileInfo = await get_file_info({
  path: "/tmp/research/ai-tools-landscape.md"
});
```

## Implementation Flow

### 1. Initialize Directory Structure
```javascript
// Create base directories
await create_directory({ path: "/tmp/research" });
await create_directory({ path: "/tmp/research/tool-profiles" });
```

### 2. Conduct Web Research
```javascript
// Search for AI development tools
const toolsSearch = await brave_web_search({
  query: "best AI code generation tools 2024",
  count: 20,
  freshness: "year"
});

// Extract tool names and search for details
for (const tool of extractedTools) {
  const detailSearch = await brave_web_search({
    query: `${tool.name} pricing features reviews`,
    count: 10
  });
  // Process and structure the data
}
```

### 3. Process and Structure Data
```javascript
// Structure the collected data
const structuredData = {
  researchDate: new Date().toISOString(),
  tools: processedTools,
  marketTrends: identifiedTrends,
  sources: searchSources
};
```

### 4. Generate Reports
```javascript
// Generate markdown report
const report = generateMarkdownReport(structuredData);

// Save all outputs
await write_file({
  path: "/tmp/research/ai-tools-landscape.md",
  content: report
});

await write_file({
  path: "/tmp/research/raw-data.json",
  content: JSON.stringify(structuredData, null, 2)
});
```

## Best Practices

### Research Strategy
1. Start with broad searches to identify tools
2. Follow up with specific searches for each tool
3. Cross-reference information from multiple sources
4. Focus on recent content (2023-2024)

### Data Organization
1. Maintain consistent data structures
2. Save raw data for future processing
3. Use clear file naming conventions
4. Create backups of important findings

### Error Handling
1. Implement retry logic for failed searches
2. Handle missing data gracefully
3. Validate data before saving
4. Log all operations for debugging

## Common Patterns

### Pagination for Comprehensive Results
```javascript
let allResults = [];
for (let offset = 0; offset < 60; offset += 20) {
  const results = await brave_web_search({
    query: "AI developer tools",
    count: 20,
    offset: offset
  });
  allResults = allResults.concat(results);
}
```

### Structured Data Extraction
```javascript
function extractToolInfo(searchResults) {
  return {
    name: extractName(searchResults),
    features: extractFeatures(searchResults),
    pricing: extractPricing(searchResults),
    company: extractCompany(searchResults)
  };
}
```

### Report Generation Template
```javascript
function generateToolSection(tool) {
  return `
### ${tool.name}
- **Company**: ${tool.company}
- **Core Features**: ${tool.features.join(', ')}
- **Pricing**: ${tool.pricing}
- **Market Position**: ${tool.marketPosition}
`;
}
```

## Troubleshooting

### Search API Issues
- **Rate Limiting**: Implement exponential backoff
- **No Results**: Try alternative query formulations
- **Timeout**: Reduce result count and use pagination

### File System Issues
- **Permission Denied**: Ensure `/tmp` directory access
- **Disk Full**: Check available space before writing
- **Path Not Found**: Create parent directories first

### Data Quality Issues
- **Outdated Information**: Use freshness filters
- **Incomplete Data**: Combine multiple searches
- **Conflicting Information**: Note discrepancies in report

## Task-Specific Tips

1. **Tool Categories**: Ensure coverage across IDE extensions, standalone IDEs, and AI agents
2. **Pricing Research**: Look for official pricing pages and recent announcements
3. **Feature Comparison**: Create a standardized feature list for fair comparison
4. **Market Trends**: Look for industry reports and analysis articles
5. **User Feedback**: Search for reviews and community discussions

Remember: The goal is to produce actionable market intelligence that helps readers understand the AI development tools landscape and make informed decisions.