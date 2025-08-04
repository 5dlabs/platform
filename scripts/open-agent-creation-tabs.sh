#!/bin/bash
# Open GitHub App creation tabs for all agents
# Makes it easy to create multiple agents in parallel

set -euo pipefail

# Configuration
ORG="5dlabs"
REPO="platform"
HOMEPAGE_URL="https://github.com/${ORG}/${REPO}"
GITHUB_APP_URL="https://github.com/organizations/${ORG}/settings/apps/new"

# Agents to create (name:description format)
AGENTS=(
    "rex:Senior Backend Architecture Agent"
    "blaze:Performance Optimization Agent" 
    "cipher:Security & Code Analysis Agent"
)

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 Opening GitHub App creation tabs for all agents...${NC}"
echo "================================================================="
echo

# Function to URL encode text
url_encode() {
    echo "$1" | sed 's/ /%20/g'
}

# Open avatar files in Finder for easy access
echo -e "${BLUE}📁 Opening avatar files in Finder...${NC}"
open -R rex-avatar.png &
open -R blaze-avatar.png &
open -R cipher-avatar.png &

echo -e "${GREEN}✅ Avatar files opened in Finder${NC}"
echo

# Open GitHub App creation pages
for agent_entry in "${AGENTS[@]}"; do
    # Split name:description
    agent="${agent_entry%%:*}"
    description="${agent_entry#*:}"
    
    # Capitalize first letter properly
    app_name="5DLabs-$(echo "${agent:0:1}" | tr '[:lower:]' '[:upper:]')$(echo "${agent:1}")"
    
    echo -e "${BLUE}🌐 Opening creation page for: ${app_name}${NC}"
    echo "   Description: ${description}"
    
    # Pre-fill the form with URL parameters
    encoded_name=$(url_encode "$app_name")
    encoded_description=$(url_encode "$description")
    encoded_homepage=$(url_encode "$HOMEPAGE_URL")
    
    # GitHub App creation URL with pre-filled fields
    url="${GITHUB_APP_URL}?name=${encoded_name}&description=${encoded_description}&url=${encoded_homepage}"
    
    # Open in new tab
    open "$url" &
    
    # Small delay to prevent overwhelming the browser
    sleep 0.5
done

echo
echo -e "${GREEN}🎉 All tabs opened!${NC}"
echo
echo -e "${YELLOW}📋 Instructions for each GitHub App:${NC}"
echo "1. Fill in the pre-populated fields (should already be there)"
echo "2. Set Webhook to INACTIVE (uncheck 'Active')"
echo "3. Set Repository permissions:"
echo "   - Contents: Write"
echo "   - Issues: Write"
echo "   - Pull requests: Write"
echo "   - Metadata: Read"
echo "4. Choose 'Only on this account'"
echo "5. Create the app"
echo "6. Upload the avatar from Finder"
echo "7. Install to 5dlabs organization"
echo "8. Generate and download private key"
echo
echo -e "${YELLOW}📸 Avatar files opened in Finder:${NC}"
echo "- rex-avatar.png"
echo "- blaze-avatar.png" 
echo "- cipher-avatar.png"
echo
echo -e "${BLUE}💡 Tip: Work through the tabs in order, then come back here${NC}"
echo "    I'll help you store all the credentials automatically!"

# Wait for user
echo
read -p "Press Enter when you've created all the GitHub Apps..."

echo
echo -e "${GREEN}🎯 Ready to store credentials!${NC}"
echo "Place all downloaded private key files in this directory:"
echo "- rex-private-key.pem"
echo "- blaze-private-key.pem"
echo "- cipher-private-key.pem"
echo
echo "Then run: ./scripts/store-all-agent-credentials.sh"