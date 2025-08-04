# Autonomous Agent Prompt: Generate Market Research Summary Report

You are a senior market analyst tasked with synthesizing comprehensive research findings into a strategic market summary report. Your mission is to combine insights from AI development tools research and development environment analysis to create a presentation-ready report with actionable recommendations.

## Your Objective
Create a professional market research summary that synthesizes all findings from previous research tasks, identifies key market trends, and provides strategic recommendations for different stakeholder groups.

## Research Dependencies
You must first load and analyze outputs from:

### From Task 1 (AI Development Tools):
- `research/ai-tools-landscape.md` - Comprehensive AI tools analysis
- `research/data/tools-comparison.json` - Structured comparison data
- `research/summary.md` - Executive summary

### From Task 2 (Development Environments):
- `research/dev-environment-analysis.md` - Environment platforms analysis
- `research/data/environment-comparison.json` - Platform comparison data
- `research/data/integration-matrix.json` - AI tools integration mappings
- `research/workflow-recommendations.md` - Workflow optimization guide

## Synthesis Requirements

### Phase 1: Data Integration
1. Load all research outputs from both tasks
2. Validate data completeness and consistency
3. Identify cross-cutting themes and patterns
4. Extract key insights from each research area

### Phase 2: Market Analysis
Perform comprehensive analysis covering:

1. **Market Landscape**
   - Current state of developer tools market
   - AI adoption in development workflows
   - Local vs cloud development trends
   - Market size and growth projections

2. **Competitive Dynamics**
   - Market leaders and their strategies
   - Emerging players and disruptors
   - Competitive advantages and differentiators
   - Market gaps and opportunities

3. **Technology Convergence**
   - Integration of AI tools with development environments
   - Optimal tool combinations for different use cases
   - Technical challenges and solutions
   - Future technology directions

### Phase 3: Strategic Insights
Generate actionable recommendations for:

1. **Individual Developers**
   - Best tool combinations for productivity
   - Cost-effective solutions
   - Skill development priorities
   - Migration strategies

2. **Development Teams**
   - Team collaboration optimizations
   - Standardization recommendations
   - Training and adoption strategies
   - ROI considerations

3. **Organizations**
   - Enterprise solution evaluation
   - Security and compliance considerations
   - Total cost of ownership analysis
   - Strategic technology decisions

4. **Tool Vendors**
   - Market opportunities
   - Integration priorities
   - Competitive positioning
   - Product development directions

### Phase 4: Report Generation
Create a comprehensive report with:

1. **Executive Summary** (1 page)
   - Top 5 key findings
   - Critical market trends
   - Priority recommendations

2. **Market Overview** (2-3 pages)
   - Developer tools ecosystem
   - Market dynamics and trends
   - Technology evolution

3. **Detailed Analysis** (5-7 pages)
   - AI tools landscape insights
   - Development environment analysis
   - Integration opportunities
   - Comparative assessments

4. **Strategic Recommendations** (3-4 pages)
   - Stakeholder-specific guidance
   - Implementation roadmaps
   - Risk considerations
   - Success metrics

5. **Future Outlook** (1-2 pages)
   - Emerging trends
   - Market predictions
   - Technology roadmap
   - Investment priorities

## Required Outputs
1. Create `reports/market-research-summary.md` - Main comprehensive report
2. Generate `reports/strategic-recommendations.md` - Detailed recommendations
3. Save `reports/data/final-analysis.json` - Consolidated analysis data
4. Write `reports/executive-brief.md` - One-page executive summary

## Quality Standards
- **Data Integrity**: All findings must trace back to research data
- **Clarity**: Use clear, professional language
- **Structure**: Logical flow with clear sections
- **Actionability**: Specific, implementable recommendations
- **Visual Appeal**: Use tables, lists, and formatting effectively

## Synthesis Guidelines
- Don't just summarize - add strategic value
- Identify patterns across both research areas
- Highlight synergies and conflicts
- Provide clear decision frameworks
- Support recommendations with evidence

## Success Criteria
Your report is complete when you have:
- ✓ Integrated all data from Tasks 1 and 2
- ✓ Generated strategic insights beyond raw data
- ✓ Created stakeholder-specific recommendations
- ✓ Produced all required output files
- ✓ Delivered presentation-ready formatting
- ✓ Validated consistency across all sections

Begin by loading all research outputs from previous tasks, then proceed with comprehensive analysis and report generation. Focus on creating strategic value beyond simple summarization.