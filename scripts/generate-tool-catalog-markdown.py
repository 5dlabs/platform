#!/usr/bin/env python3
"""
Generate comprehensive tool catalog markdown documentation from the Toolman tool catalog JSON.
This markdown will be consumed by the docs agent to understand available tools and generate
task-specific client configurations.
"""

import json
import sys
from datetime import datetime
from typing import Dict, List, Any


def generate_tool_catalog_markdown(catalog_data: Dict[str, Any]) -> str:
    """Generate comprehensive markdown documentation from tool catalog data."""

    markdown = []

    # Header
    markdown.append("# MCP Tool Catalog")
    markdown.append("")
    markdown.append("This document provides a comprehensive overview of all available MCP tools,")
    markdown.append("both local and remote, for task-specific configuration generation.")
    markdown.append("")

    # Last updated info
    last_updated = catalog_data.get("last_updated", "Unknown")
    markdown.append(f"**Last Updated**: {last_updated}")
    markdown.append("")

    # Table of Contents
    markdown.append("## Table of Contents")
    markdown.append("")
    markdown.append("- [Local Servers](#local-servers)")

    # Add TOC entries for each local server
    if "local" in catalog_data:
        for server_name in catalog_data["local"].keys():
            markdown.append(f"  - [{server_name.title()}](#{server_name.lower()})")

    markdown.append("- [Remote Tools](#remote-tools)")

    # Add TOC entries for each remote server
    if "remote" in catalog_data:
        for server_name in catalog_data["remote"].keys():
            markdown.append(f"  - [{server_name.title()}](#{server_name.lower()})")

    markdown.append("- [Client Configuration Guide](#client-configuration-guide)")
    markdown.append("")

    # Local Servers Section
    markdown.append("## Local Servers")
    markdown.append("")
    markdown.append("Local servers run as separate processes and communicate via stdio transport.")
    markdown.append("They require complete command configuration in the client config.")
    markdown.append("")

    if "local" in catalog_data:
        for server_name, server_info in catalog_data["local"].items():
            generate_local_server_section(markdown, server_name, server_info)

    # Remote Tools Section
    markdown.append("## Remote Tools")
    markdown.append("")
    markdown.append("Remote tools are accessible through the Toolman proxy server.")
    markdown.append("Only tool names are needed in the client config.")
    markdown.append("")

    if "remote" in catalog_data:
        for server_name, server_info in catalog_data["remote"].items():
            generate_remote_server_section(markdown, server_name, server_info)

    # Configuration Guide
    generate_configuration_guide(markdown)

    return "\n".join(markdown)


def generate_local_server_section(markdown: List[str], server_name: str, server_info: Dict[str, Any]):
    """Generate documentation section for a local server."""

    markdown.append(f"### {server_name.title()}")
    markdown.append("")

    # Description
    description = server_info.get("description", "No description available")
    markdown.append(f"**Description**: {description}")
    markdown.append("")

    # Server Configuration (if available)
    if "command" in server_info:
        markdown.append("**Server Configuration**:")
        markdown.append("```json")
        markdown.append("{")
        markdown.append(f'  "command": "{server_info.get("command", "")}"')

        args = server_info.get("args", [])
        if args:
            args_str = ", ".join([f'"{arg}"' for arg in args])
            markdown.append(f'  "args": [{args_str}]')

        working_dir = server_info.get("working_directory", "project_root")
        markdown.append(f'  "workingDirectory": "{working_dir}"')
        markdown.append("}")
        markdown.append("```")
        markdown.append("")

    # Tools
    tools = server_info.get("tools", [])
    if tools:
        markdown.append(f"**Available Tools** ({len(tools)} tools):")
        markdown.append("")

        for tool in tools:
            generate_tool_documentation(markdown, tool, indent="")


def generate_remote_server_section(markdown: List[str], server_name: str, server_info: Dict[str, Any]):
    """Generate documentation section for a remote server."""

    markdown.append(f"### {server_name.title()}")
    markdown.append("")

    # Description
    description = server_info.get("description", "No description available")
    markdown.append(f"**Description**: {description}")
    markdown.append("")

    # Endpoint info (if available)
    if "endpoint" in server_info:
        markdown.append(f"**Endpoint**: `{server_info['endpoint']}`")
        markdown.append("")

    # Tools
    tools = server_info.get("tools", [])
    if tools:
        markdown.append(f"**Available Tools** ({len(tools)} tools):")
        markdown.append("")

        for tool in tools:
            generate_tool_documentation(markdown, tool, indent="")


def generate_tool_documentation(markdown: List[str], tool: Dict[str, Any], indent: str):
    """Generate documentation for a single tool."""

    tool_name = tool.get("name", "unknown")
    description = tool.get("description", "No description available")
    category = tool.get("category", "general")
    use_cases = tool.get("use_cases", [])

    markdown.append(f"{indent}#### `{tool_name}`")
    markdown.append(f"{indent}")
    markdown.append(f"{indent}**Description**: {description}")
    markdown.append(f"{indent}")
    markdown.append(f"{indent}**Category**: {category}")

    if use_cases:
        use_cases_str = ", ".join(use_cases)
        markdown.append(f"{indent}**Use Cases**: {use_cases_str}")

    # Input Schema (simplified)
    input_schema = tool.get("input_schema")
    if input_schema and isinstance(input_schema, dict):
        properties = input_schema.get("properties", {})
        required = input_schema.get("required", [])

        if properties:
            markdown.append(f"{indent}")
            markdown.append(f"{indent}**Parameters**:")
            for param_name, param_info in properties.items():
                param_type = param_info.get("type", "unknown")
                param_desc = param_info.get("description", "No description")
                required_marker = " *(required)*" if param_name in required else ""
                markdown.append(f"{indent}- `{param_name}` ({param_type}){required_marker}: {param_desc}")

    markdown.append(f"{indent}")


def generate_configuration_guide(markdown: List[str]):
    """Generate the client configuration guide section."""

    markdown.append("## Client Configuration Guide")
    markdown.append("")
    markdown.append("When generating task-specific client configurations, use this structure:")
    markdown.append("")

    markdown.append("### Expected client-config.json Structure")
    markdown.append("")
    markdown.append("```json")
    markdown.append("{")
    markdown.append('  "remoteTools": [')
    markdown.append('    "kubernetes_listResources",')
    markdown.append('    "memory_create_entities"')
    markdown.append('  ],')
    markdown.append('  "localServers": {')
    markdown.append('    "filesystem": {')
    markdown.append('      "command": "npx",')
    markdown.append('      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],')
    markdown.append('      "tools": ["read_file", "write_file", "list_directory"],')
    markdown.append('      "workingDirectory": "project_root"')
    markdown.append('    }')
    markdown.append('  }')
    markdown.append('}')
    markdown.append("```")
    markdown.append("")

    markdown.append("### Key Points")
    markdown.append("")
    markdown.append("- **Remote Tools**: Array of specific tool names (not server names)")
    markdown.append("- **Local Servers**: Object with complete server configurations")
    markdown.append("- **Tools Selection**: Only include tools actually needed for the specific task")
    markdown.append("- **Working Directory**: Use `project_root` as the standard working directory")
    markdown.append("")

    markdown.append("### Tool Selection Guidelines")
    markdown.append("")
    markdown.append("1. **Read Task Requirements**: Understand what the task needs to accomplish")
    markdown.append("2. **Match Tools to Actions**: Select tools that directly support the required actions")
    markdown.append("3. **Minimize Tool Set**: Only include tools that are actually needed")
    markdown.append("4. **Consider Dependencies**: Include tools needed for prerequisite actions")
    markdown.append("")


def main():
    """Main function to read JSON and generate markdown."""

    if len(sys.argv) != 2:
        print("Usage: python3 generate-tool-catalog-markdown.py <tool-catalog.json>")
        sys.exit(1)

    catalog_file = sys.argv[1]

    try:
        with open(catalog_file, 'r') as f:
            catalog_data = json.load(f)

        markdown = generate_tool_catalog_markdown(catalog_data)
        print(markdown)

    except FileNotFoundError:
        print(f"Error: File '{catalog_file}' not found")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in '{catalog_file}': {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()