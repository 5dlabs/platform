# Toolman Guide: Research AI Development Tools Landscape

## Overview
This guide explains how to use the selected tools for conducting comprehensive market research on AI development tools. The task requires web search capabilities for gathering market intelligence and local file operations for processing and storing research data.

## Core Tools

### 1. brave_web_search (Remote Tool)
**Purpose**: Primary tool for conducting market research via web searches

**When to Use**:
- Searching for AI development tools and their features
- Finding pricing information and comparisons
- Gathering user reviews and market analyses
- Researching company backgrounds and funding information

**How to Use**:
```json
{
  "query": "AI code generation tools 2025 comparison",
  "count": 20,
  "offset": 0
}
```

**Best Practices**:
- Use specific, targeted queries for better results
- Leverage pagination with offset for comprehensive coverage
- Combine multiple search queries for different perspectives
- Filter results by recency for current information

**Example Queries**:
- "GitHub Copilot vs Cursor vs Claude Code features pricing"
- "AI development tools market analysis 2025"
- "Developer productivity AI tools comparison"
- "Codeium Tabnine features enterprise pricing"

### 2. write_file (Local Server Tool)
**Purpose**: Save research findings and generate reports

**When to Use**:
- Creating the main research report (ai-tools-landscape.md)
- Saving structured comparison data (JSON format)
- Storing raw search results for reference
- Generating executive summaries

**How to Use**:
```json
{
  "path": "research/ai-tools-landscape.md",
  "content": "# AI Development Tools Landscape\n\n## Executive Summary..."
}
```

**File Structure**:
- `research/ai-tools-landscape.md` - Main comprehensive report
- `research/data/tools-comparison.json` - Structured comparison data
- `research/data/raw-search-results.json` - Raw research data
- `research/summary.md` - Executive summary

### 3. create_directory (Local Server Tool)
**Purpose**: Set up directory structure for research artifacts

**When to Use**:
- Before starting research to create necessary folders
- Organizing different types of research data

**How to Use**:
```json
{
  "path": "research/data"
}
```

**Directory Structure**:
```
research/
├── ai-tools-landscape.md
├── summary.md
└── data/
    ├── tools-comparison.json
    └── raw-search-results.json
```

## Supporting Tools

### 4. read_text_file (Local Server Tool)
**Purpose**: Read existing files for data aggregation and validation

**When to Use**:
- Reviewing saved search results
- Aggregating data from multiple sources
- Validating report content
- Cross-referencing information

### 5. list_directory (Local Server Tool)
**Purpose**: Verify file creation and organization

**When to Use**:
- Confirming all required files are created
- Checking directory structure
- Validating output completeness

## Implementation Flow

### Phase 1: Environment Setup
1. Use `create_directory` to establish folder structure
2. Prepare templates for data collection

### Phase 2: Research Execution
1. Execute multiple `brave_web_search` queries:
   - General market overview searches
   - Specific tool-focused searches
   - Pricing and comparison searches
   - User feedback and review searches

2. For each search:
   - Parse results for relevant information
   - Extract key data points
   - Store raw results using `write_file`

### Phase 3: Data Processing
1. Aggregate search results
2. Structure data into comparison format
3. Use `write_file` to save:
   - Raw search results (JSON)
   - Structured comparisons (JSON)
   - Preliminary findings (Markdown)

### Phase 4: Report Generation
1. Compile all findings into comprehensive report
2. Use `write_file` to create final deliverables:
   - Main research report
   - Executive summary
   - Supporting data files

## Best Practices

### Search Strategy
- Start broad, then narrow down to specific tools
- Use multiple query variations for comprehensive coverage
- Cross-reference information from multiple sources
- Prioritize recent information (2024-2025)

### Data Management
- Save raw data immediately after collection
- Structure data incrementally as you research
- Maintain clear file naming conventions
- Validate JSON structure before saving

### Report Quality
- Follow consistent markdown formatting
- Include specific examples and data points
- Cite sources where applicable
- Ensure logical flow and readability

## Troubleshooting

### Search Issues
- If searches return limited results, try alternative queries
- Use different keyword combinations
- Check for API rate limits and adjust accordingly

### File Operations
- Ensure directories exist before writing files
- Handle file operation errors gracefully
- Validate JSON data before saving
- Use proper encoding (UTF-8) for all files

## Common Patterns

### Tool Research Pattern
1. Search for "[Tool Name] features pricing 2025"
2. Search for "[Tool Name] vs competitors"
3. Search for "[Tool Name] user reviews"
4. Aggregate findings into tool profile

### Comparison Pattern
1. Collect data for all tools
2. Identify common comparison criteria
3. Structure into comparison matrix
4. Generate insights from patterns

## Success Indicators
- Comprehensive coverage of 5-7 major tools
- Accurate pricing and feature information
- Well-structured, readable reports
- Complete set of output files
- Integration-ready data for subsequent tasks