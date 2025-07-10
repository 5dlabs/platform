#!/bin/bash
# Export Claude session to markdown when session stops

# Read hook input
HOOK_INPUT=$(cat)
TRANSCRIPT_PATH=$(echo "$HOOK_INPUT" | jq -r '.transcript_path')
SESSION_ID=$(echo "$HOOK_INPUT" | jq -r '.session_id')

# Extract task ID and attempt from environment
TASK_ID="999999"
ATTEMPT="1"

# Output directory - use attempt-specific subdirectory
OUTPUT_DIR="/workspace/.task/${TASK_ID}/attempt-${ATTEMPT}"
mkdir -p "$OUTPUT_DIR"

# Convert JSONL to readable markdown
OUTPUT_FILE="$OUTPUT_DIR/claude-session.md"

echo "# Claude Session Export" > "$OUTPUT_FILE"
echo "**Session ID:** ${SESSION_ID}" >> "$OUTPUT_FILE"
echo "**Task ID:** ${TASK_ID}" >> "$OUTPUT_FILE"
echo "**Export Time:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "## Conversation History" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Parse JSONL and format as markdown
jq -s -r '.[] | 
  if .type == "user" then
    "### User\n" + (.message.content // .message | tostring) + "\n"
  elif .type == "assistant" then
    if .message.content then
      "### Assistant\n" + (
        .message.content | 
        if type == "array" then 
          map(
            if .type == "text" then .text
            elif .type == "tool_use" then "**Tool Use:** " + .name + "\n```json\n" + (.input | tojson) + "\n```"
            else tostring
            end
          ) | join("\n")
        else tostring
        end
      ) + "\n"
    else empty
    end
  else empty
  end
' "$TRANSCRIPT_PATH" >> "$OUTPUT_FILE" 2>/dev/null || echo "Error parsing transcript" >> "$OUTPUT_FILE"

echo "" >> "$OUTPUT_FILE"
echo "---" >> "$OUTPUT_FILE"
echo "*Exported from Claude session ${SESSION_ID}*" >> "$OUTPUT_FILE"

# Also create a compressed copy of the raw JSONL
cp "$TRANSCRIPT_PATH" "$OUTPUT_DIR/claude-session-raw.jsonl" 2>/dev/null || true

echo "Session exported to $OUTPUT_FILE"

# Generate XML export
XML_FILE="$OUTPUT_DIR/claude-session.xml"

echo '<?xml version="1.0" encoding="UTF-8"?>' > "$XML_FILE"
echo '<session>' >> "$XML_FILE"
echo "  <metadata>" >> "$XML_FILE"
echo "    <session_id>${SESSION_ID}</session_id>" >> "$XML_FILE"
echo "    <task_id>${TASK_ID}</task_id>" >> "$XML_FILE"
echo "    <export_time>$(date -u +"%Y-%m-%dT%H:%M:%SZ")</export_time>" >> "$XML_FILE"
echo "  </metadata>" >> "$XML_FILE"

# Extract file modifications
echo "  <files_modified>" >> "$XML_FILE"
jq -r '.[] | select(.type == "assistant" and .message.content) | .message.content[] | select(.type == "tool_use" and (.name == "Write" or .name == "Edit" or .name == "MultiEdit")) | .input' "$TRANSCRIPT_PATH" 2>/dev/null | 
while read -r tool_input; do
  FILE_PATH=$(echo "$tool_input" | jq -r '.file_path // .path // "unknown"' 2>/dev/null)
  if [ "$FILE_PATH" != "unknown" ] && [ "$FILE_PATH" != "null" ]; then
    echo "    <file path=\"$FILE_PATH\" />" >> "$XML_FILE"
  fi
done
echo "  </files_modified>" >> "$XML_FILE"

# Extract tool usage summary
echo "  <tool_usage_summary>" >> "$XML_FILE"
jq -r '.[] | select(.type == "assistant" and .message.content) | .message.content[] | select(.type == "tool_use") | .name' "$TRANSCRIPT_PATH" 2>/dev/null | 
sort | uniq -c | while read count tool; do
  echo "    <tool name=\"$tool\" count=\"$count\" />" >> "$XML_FILE"
done
echo "  </tool_usage_summary>" >> "$XML_FILE"

# Convert conversation to XML
echo "  <conversation>" >> "$XML_FILE"
jq -s -r '.[] | 
  if .type == "user" then
    "    <message role=\"user\" timestamp=\"" + .timestamp + "\">\n" +
    "      <content><![CDATA[" + (.message.content // .message | tostring) + "]]></content>\n" +
    "    </message>"
  elif .type == "assistant" then
    if .message.content then
      "    <message role=\"assistant\" timestamp=\"" + .timestamp + "\" model=\"" + .message.model + "\">\n" +
      (.message.content | 
        if type == "array" then 
          map(
            if .type == "text" then
              "      <text><![CDATA[" + .text + "]]></text>"
            elif .type == "tool_use" then
              "      <tool_use name=\"" + .name + "\" id=\"" + .id + "\">\n" +
              "        <input><![CDATA[" + (.input | tojson) + "]]></input>\n" +
              "      </tool_use>"
            else
              "      <unknown>" + tostring + "</unknown>"
            end
          ) | join("\n")
        else 
          "      <text><![CDATA[" + tostring + "]]></text>"
        end
      ) + "\n    </message>"
    else empty
    end
  elif .type == "user" and .toolUseResult then
    "    <tool_result tool_use_id=\"" + .message.content[0].tool_use_id + "\">\n" +
    "      <content><![CDATA[" + .message.content[0].content + "]]></content>\n" +
    "    </tool_result>"
  else empty
  end
' "$TRANSCRIPT_PATH" >> "$XML_FILE" 2>/dev/null || echo "    <error>Failed to parse conversation</error>" >> "$XML_FILE"
echo "  </conversation>" >> "$XML_FILE"

# Close XML
echo "</session>" >> "$XML_FILE"

echo "XML export saved to $XML_FILE"