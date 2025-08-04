# Toolman Guide: Generate Market Research Summary Report

## Overview
This guide explains how to use the selected tools for synthesizing research findings and generating the final market research summary report. This task focuses entirely on local file operations - reading previous research outputs and creating comprehensive summary documents.

## Core Tools

### 1. read_multiple_files (Local Server Tool)
**Purpose**: Efficiently load all research outputs from Tasks 1 and 2

**When to Use**:
- Initial task setup to load all dependencies
- Batch loading of research data
- Ensuring all inputs are available

**How to Use**:
```json
{
  "paths": [
    "research/ai-tools-landscape.md",
    "research/data/tools-comparison.json",
    "research/dev-environment-analysis.md",
    "research/data/environment-comparison.json",
    "research/data/integration-matrix.json",
    "research/workflow-recommendations.md"
  ]
}
```

**Critical Files to Load**:
From Task 1:
- `research/ai-tools-landscape.md`
- `research/data/tools-comparison.json`
- `research/summary.md`

From Task 2:
- `research/dev-environment-analysis.md`
- `research/data/environment-comparison.json`
- `research/data/integration-matrix.json`
- `research/workflow-recommendations.md`

### 2. read_text_file (Local Server Tool)
**Purpose**: Read individual files when needed for specific analysis

**When to Use**:
- Re-reading specific sections
- Validating data points
- Cross-referencing information
- Loading supplementary files

**How to Use**:
```json
{
  "path": "research/data/tools-comparison.json"
}
```

### 3. write_file (Local Server Tool)
**Purpose**: Create all summary reports and output documents

**When to Use**:
- Generating the main summary report
- Creating strategic recommendations
- Saving consolidated analysis data
- Writing executive brief

**Output Files to Create**:
- `reports/market-research-summary.md` - Comprehensive report
- `reports/strategic-recommendations.md` - Detailed recommendations
- `reports/data/final-analysis.json` - Consolidated data
- `reports/executive-brief.md` - One-page summary

## Supporting Tools

### 4. create_directory (Local Server Tool)
**Purpose**: Set up the reports directory structure

**When to Use**:
- Before generating any reports
- Creating subdirectories for data
- Organizing output structure

**Directory Structure**:
```
reports/
├── market-research-summary.md
├── strategic-recommendations.md
├── executive-brief.md
└── data/
    └── final-analysis.json
```

### 5. list_directory (Local Server Tool)
**Purpose**: Verify all required inputs exist and outputs are created

**When to Use**:
- Checking Task 1 and 2 outputs
- Validating file creation
- Final quality check

### 6. directory_tree (Local Server Tool)
**Purpose**: Get comprehensive view of research file structure

**When to Use**:
- Understanding overall file organization
- Documenting research artifacts
- Troubleshooting missing files

## Implementation Flow

### Phase 1: Environment Setup
1. Use `create_directory` to establish reports structure:
   ```json
   {
     "path": "reports/data"
   }
   ```

2. Use `list_directory` to verify all inputs exist:
   ```json
   {
     "path": "research"
   }
   ```

### Phase 2: Data Loading
1. Use `read_multiple_files` to load all research:
   ```json
   {
     "paths": [
       "research/ai-tools-landscape.md",
       "research/data/tools-comparison.json",
       "research/dev-environment-analysis.md",
       "research/data/environment-comparison.json",
       "research/data/integration-matrix.json"
     ]
   }
   ```

2. Parse and validate all loaded data
3. Extract key findings from each source

### Phase 3: Analysis and Synthesis
1. Identify patterns across both research areas
2. Generate strategic insights
3. Develop stakeholder-specific recommendations
4. Create market projections

### Phase 4: Report Generation
1. Structure the main summary report
2. Write comprehensive sections:
   - Executive summary
   - Market overview
   - Competitive analysis
   - Technology insights
   - Strategic recommendations
   - Future outlook

3. Create supporting documents:
   - Detailed recommendations
   - Consolidated data
   - Executive brief

## Best Practices

### Data Synthesis
- Cross-reference findings between tasks
- Identify contradictions and resolve them
- Look for emerging patterns
- Generate insights beyond raw data

### Report Structure
- Start with high-level summary
- Progress to detailed analysis
- End with actionable recommendations
- Maintain logical flow throughout

### Quality Assurance
- Validate all data references
- Ensure consistency across documents
- Check for completeness
- Verify professional formatting

## Common Patterns

### Synthesis Pattern
1. Load data from both tasks
2. Identify common themes
3. Find unique insights in overlap
4. Generate strategic conclusions
5. Craft specific recommendations

### Report Section Pattern
1. Open with key finding
2. Provide supporting evidence
3. Analyze implications
4. Offer strategic insight
5. Close with recommendation

### Recommendation Pattern
1. State the recommendation clearly
2. Provide rationale from research
3. Identify target stakeholder
4. Suggest implementation approach
5. Define success metrics

## Troubleshooting

### Missing Dependencies
- Use `list_directory` to check file presence
- Verify file paths are correct
- Check for task completion markers
- Report specific missing files

### Data Inconsistencies
- Cross-check between sources
- Identify authoritative data
- Document discrepancies
- Make reasoned decisions

### Synthesis Challenges
- Return to source data
- Look for additional patterns
- Consider multiple perspectives
- Focus on strategic value

## Success Indicators
- All source files successfully loaded
- Strategic insights clearly articulated
- Recommendations specific and actionable
- Reports professionally formatted
- All deliverables created
- Synthesis adds clear value

## Final Checklist
Before completion, ensure:
- ✓ All Task 1 & 2 data incorporated
- ✓ Strategic insights generated
- ✓ All stakeholder groups addressed
- ✓ Professional formatting applied
- ✓ All output files created
- ✓ Executive brief compelling
- ✓ Data accuracy verified