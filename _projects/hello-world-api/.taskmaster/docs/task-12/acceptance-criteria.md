# Task 12: Research Express.js Best Practices - Acceptance Criteria

## Definition of Done
The research task is successfully completed when all the following criteria are met:

## Required Deliverables

### 1. Documentation File Creation
- [ ] Directory `docs` exists in project root
- [ ] File `docs/best-practices.md` is created
- [ ] File contains valid markdown syntax
- [ ] File is properly formatted and readable

### 2. Content Structure
- [ ] Table of Contents is present
- [ ] Introduction section exists
- [ ] Error Handling Patterns section exists
- [ ] Middleware Organization section exists
- [ ] Request Logging section exists
- [ ] Security Best Practices section exists
- [ ] References section exists

### 3. Error Handling Content
- [ ] At least 3 error handling patterns documented
- [ ] Global error handler example provided
- [ ] Async error handling covered
- [ ] Error response formatting discussed
- [ ] Code examples are syntactically correct

### 4. Middleware Content
- [ ] Middleware order best practices explained
- [ ] Application vs router middleware covered
- [ ] Custom middleware patterns shown
- [ ] At least 2 code examples provided
- [ ] Practical implementation guidance included

### 5. Logging Content
- [ ] Morgan configuration examples provided
- [ ] At least one alternative logging solution mentioned
- [ ] Environment-specific configurations shown
- [ ] Log security considerations addressed
- [ ] Code examples for implementation

### 6. Security Content
- [ ] Helmet.js configuration documented
- [ ] CORS setup explained
- [ ] Rate limiting strategies provided
- [ ] At least 3 security headers mentioned
- [ ] Implementation examples included

### 7. References and Quality
- [ ] At least 5 credible sources cited
- [ ] Sources are from 2023 or later
- [ ] Links to official documentation included
- [ ] No broken or invalid links

## Verification Tests

### Test 1: File Structure Verification
```bash
# Check file exists
if [ -f "hello-world-api/docs/best-practices.md" ]; then
  echo "✓ Documentation file exists"
else
  echo "✗ Documentation file missing"
  exit 1
fi

# Check file size (should be substantial)
SIZE=$(wc -c < "hello-world-api/docs/best-practices.md")
if [ $SIZE -gt 3000 ]; then
  echo "✓ Documentation has substantial content"
else
  echo "✗ Documentation too short"
fi
```

### Test 2: Content Completeness
```bash
# Check for required sections
FILE="hello-world-api/docs/best-practices.md"

SECTIONS=("Table of Contents" "Error Handling" "Middleware" "Request Logging" "Security" "References")

for section in "${SECTIONS[@]}"; do
  if grep -qi "$section" "$FILE"; then
    echo "✓ Section found: $section"
  else
    echo "✗ Missing section: $section"
  fi
done
```

### Test 3: Code Examples Verification
```bash
# Count code blocks
CODE_BLOCKS=$(grep -c "^\`\`\`" "hello-world-api/docs/best-practices.md")

if [ $CODE_BLOCKS -ge 10 ]; then
  echo "✓ Sufficient code examples (${CODE_BLOCKS} blocks)"
else
  echo "✗ Insufficient code examples (${CODE_BLOCKS} blocks, need at least 10)"
fi
```

### Test 4: References Check
```bash
# Check for HTTP/HTTPS links
LINKS=$(grep -o 'https\?://[^)]*' "hello-world-api/docs/best-practices.md" | wc -l)

if [ $LINKS -ge 5 ]; then
  echo "✓ Sufficient references (${LINKS} links)"
else
  echo "✗ Insufficient references (${LINKS} links, need at least 5)"
fi
```

### Test 5: Markdown Quality
```bash
# Check markdown structure
FILE="hello-world-api/docs/best-practices.md"

# Check for proper headers
H1=$(grep -c "^# " "$FILE")
H2=$(grep -c "^## " "$FILE")
H3=$(grep -c "^### " "$FILE")

if [ $H1 -ge 1 ] && [ $H2 -ge 5 ] && [ $H3 -ge 3 ]; then
  echo "✓ Good markdown structure"
else
  echo "✗ Poor markdown structure (H1: $H1, H2: $H2, H3: $H3)"
fi
```

## Content Quality Criteria

### Code Example Requirements
1. **Syntax Correctness**
   - All JavaScript code must be valid
   - ES6+ syntax should be used
   - No syntax errors when parsed

2. **Relevance**
   - Examples directly applicable to Express.js
   - Code can be used in the project
   - Patterns are current (not deprecated)

3. **Completeness**
   - Full working examples, not fragments
   - Include necessary imports
   - Show error handling

### Documentation Standards
1. **Clarity**
   - Clear explanations for each practice
   - Reasoning behind recommendations
   - When to use each pattern

2. **Organization**
   - Logical flow between sections
   - Consistent formatting
   - Easy navigation

3. **Practicality**
   - Actionable recommendations
   - Implementation steps
   - Common pitfalls noted

## Common Failure Modes

1. **Outdated Information**
   - Using deprecated Express patterns
   - References to old versions
   - Obsolete security practices

2. **Incomplete Research**
   - Missing key topics
   - Shallow coverage
   - No code examples

3. **Poor Documentation**
   - No structure or organization
   - Missing table of contents
   - Broken markdown formatting

4. **Irrelevant Content**
   - Practices for wrong use case
   - Overly complex examples
   - Off-topic information

## Final Validation Script
```bash
#!/bin/bash
echo "Validating Express.js Best Practices Documentation..."

FILE="hello-world-api/docs/best-practices.md"
ERRORS=0

# Check file exists
if [ ! -f "$FILE" ]; then
  echo "✗ File not found: $FILE"
  exit 1
fi

# Check file size
SIZE=$(wc -c < "$FILE")
if [ $SIZE -lt 3000 ]; then
  echo "✗ File too small: ${SIZE} bytes"
  ((ERRORS++))
fi

# Check required sections
REQUIRED=("Error Handling" "Middleware" "Logging" "Security" "References")
for section in "${REQUIRED[@]}"; do
  if ! grep -qi "$section" "$FILE"; then
    echo "✗ Missing section: $section"
    ((ERRORS++))
  fi
done

# Check code examples
CODE_BLOCKS=$(grep -c "^\`\`\`" "$FILE")
if [ $CODE_BLOCKS -lt 8 ]; then
  echo "✗ Too few code examples: $CODE_BLOCKS"
  ((ERRORS++))
fi

# Check references
REFS=$(grep -c "http" "$FILE")
if [ $REFS -lt 5 ]; then
  echo "✗ Too few references: $REFS"
  ((ERRORS++))
fi

# Check for Express.js specific content
if ! grep -qi "express" "$FILE"; then
  echo "✗ No Express.js specific content found"
  ((ERRORS++))
fi

# Check for security practices
if ! grep -qi "helmet" "$FILE"; then
  echo "✗ No Helmet.js security practices found"
  ((ERRORS++))
fi

if [ $ERRORS -eq 0 ]; then
  echo "✅ All validation checks passed!"
  exit 0
else
  echo "❌ Found $ERRORS validation errors"
  exit 1
fi
```