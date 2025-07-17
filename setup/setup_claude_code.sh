#!/bin/bash

# Setup script for adding bpftrace MCP server to Claude Code

echo "🚀 Setting up bpftrace MCP server for Claude Code"
echo "=============================================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the absolute path of the server
SERVER_PATH="$(cd "$(dirname "$0")" && pwd)/server.py"

echo -e "${GREEN}📁 Server path: $SERVER_PATH${NC}"

# Check if server.py exists
if [ ! -f "$SERVER_PATH" ]; then
    echo -e "${RED}❌ Error: server.py not found in current directory${NC}"
    exit 1
fi

# Check if claude command exists
if ! command -v claude &> /dev/null; then
    echo -e "${RED}❌ Error: 'claude' command not found. Please install Claude Code CLI first.${NC}"
    exit 1
fi

# Check Python installation
if ! command -v python &> /dev/null && ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Error: Python not found. Please install Python 3.10 or higher.${NC}"
    exit 1
fi

# Determine Python command
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
else
    PYTHON_CMD="python"
fi

echo -e "${GREEN}🐍 Using Python: $PYTHON_CMD${NC}"

# Check if bpftrace is installed
if ! command -v bpftrace &> /dev/null; then
    echo -e "${YELLOW}⚠️  Warning: bpftrace not found!${NC}"
    echo "Please install bpftrace:"
    echo "  Ubuntu/Debian: sudo apt-get install bpftrace"
    echo "  Fedora: sudo dnf install bpftrace"
    echo ""
fi

# Ask for scope
echo ""
echo "Choose installation scope:"
echo "1) Local (current project only)"
echo "2) User (all your projects)"
echo "3) Project (shared with team via .mcp.json)"
echo ""
read -p "Enter choice [1-3] (default: 1): " choice

case $choice in
    2)
        SCOPE="--scope user"
        SCOPE_NAME="user"
        ;;
    3)
        SCOPE="--scope project"
        SCOPE_NAME="project"
        ;;
    *)
        SCOPE=""
        SCOPE_NAME="local"
        ;;
esac

echo -e "${GREEN}📦 Installing with $SCOPE_NAME scope${NC}"

# Check if fastmcp is installed
echo -e "${YELLOW}Checking Python dependencies...${NC}"
if ! $PYTHON_CMD -c "import fastmcp" 2>/dev/null; then
    echo "Installing fastmcp..."
    pip install fastmcp || {
        echo -e "${RED}❌ Failed to install fastmcp. Please run: pip install fastmcp${NC}"
        exit 1
    }
fi

# Add the server to Claude Code
echo ""
echo -e "${GREEN}Adding bpftrace server to Claude Code...${NC}"

if claude mcp add $SCOPE bpftrace $PYTHON_CMD "$SERVER_PATH"; then
    echo -e "${GREEN}✅ Successfully added bpftrace MCP server!${NC}"
else
    echo -e "${RED}❌ Failed to add server to Claude Code${NC}"
    exit 1
fi

# Show status
echo ""
echo -e "${GREEN}Current MCP servers:${NC}"
claude mcp list

echo ""
echo -e "${GREEN}✅ Setup complete!${NC}"
echo ""
echo "You can now use bpftrace in Claude Code. Try asking:"
echo "  - 'List available bpftrace probes'"
echo "  - 'Show me bpftrace helper functions'"
echo "  - 'Trace system calls with bpftrace'"
echo ""
echo -e "${YELLOW}⚠️  Security Note:${NC}"
echo "The server uses sudo with password '123456' by default."
echo "For production use, configure passwordless sudo for bpftrace:"
echo "  sudo visudo"
echo "  Add: $USER ALL=(ALL) NOPASSWD: /usr/bin/bpftrace"