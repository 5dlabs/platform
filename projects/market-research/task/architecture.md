# Market Research Tool - Architecture

## System Overview
A lightweight research automation tool that combines web research capabilities with local file processing to generate market intelligence reports.

## Architecture Components

### 1. Research Engine
- **Web Research Module**: Uses Brave Search API via remote MCP server
- **Data Collection**: Gathers information about competitor tools and market trends
- **Search Strategy**: Focused queries to minimize API usage while maximizing relevance

### 2. Local Processing
- **File System Operations**: Uses local MCP server for file read/write operations
- **Report Generator**: Creates structured markdown reports from research data
- **Data Analysis**: Basic processing of collected information

### 3. Output System
- **Markdown Reports**: Structured findings in readable format
- **Summary Generation**: Condensed insights and recommendations
- **File Organization**: Logical structure for research artifacts

## Technology Stack
- **Remote MCP**: Brave Search integration for web research
- **Local MCP**: File system operations and local tool access
- **Claude**: AI-powered analysis and report generation
- **Task Master**: Workflow orchestration and task management

## Data Flow
1. **Task Initiation**: Task Master triggers research workflow
2. **Web Research**: Remote MCP server performs Brave Search queries
3. **Data Processing**: Claude analyzes and structures findings
4. **Local Storage**: Local MCP server writes reports to file system
5. **Report Generation**: Final markdown reports with insights

## Integration Points
- Task Master workflow orchestration
- Remote MCP server for web research
- Local MCP server for file operations
- Claude for intelligent processing and analysis

## Security & Privacy
- No sensitive data storage
- Public web research only
- Local file operations in sandboxed environment
- Minimal API key usage

## Testing Strategy
- Validate remote MCP server connectivity
- Test local MCP server file operations
- Verify end-to-end workflow execution
- Confirm report generation and format