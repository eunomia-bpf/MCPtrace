#!/bin/bash

# Setup script for adding bpftrace MCP server to Claude Desktop

echo "🚀 Setting up bpftrace MCP server for Claude Desktop"
echo "=================================================="

# Get the absolute path of the current directory
SERVER_PATH="$(pwd)/../server.py"

# Detect OS and set config path
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_PATH="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CONFIG_PATH="$HOME/.config/claude/claude_desktop_config.json"
else
    echo "❌ Unsupported OS. Please configure manually."
    exit 1
fi

echo "📁 Server path: $SERVER_PATH"
echo "📁 Config path: $CONFIG_PATH"

# Check if server.py exists
if [ ! -f "$SERVER_PATH" ]; then
    echo "❌ Error: server.py not found in current directory"
    exit 1
fi

# Create config directory if it doesn't exist
CONFIG_DIR=$(dirname "$CONFIG_PATH")
mkdir -p "$CONFIG_DIR"

# Check if config file exists
if [ -f "$CONFIG_PATH" ]; then
    echo "📄 Found existing Claude Desktop config"
    
    # Backup existing config
    cp "$CONFIG_PATH" "$CONFIG_PATH.backup"
    echo "✅ Created backup at: $CONFIG_PATH.backup"
    
    # Check if bpftrace server is already configured
    if grep -q '"bpftrace"' "$CONFIG_PATH"; then
        echo "⚠️  Warning: bpftrace server already configured in Claude Desktop"
        echo "   Please check your config file manually"
        exit 0
    fi
else
    echo "📄 Creating new Claude Desktop config"
    echo '{"mcpServers": {}}' > "$CONFIG_PATH"
fi

# Create temporary Python script to update JSON
cat > /tmp/update_claude_config.py << EOF
import json
import sys

config_path = sys.argv[1]
server_path = sys.argv[2]

with open(config_path, 'r') as f:
    config = json.load(f)

if 'mcpServers' not in config:
    config['mcpServers'] = {}

config['mcpServers']['bpftrace'] = {
    "command": "python",
    "args": [server_path],
    "env": {}
}

with open(config_path, 'w') as f:
    json.dump(config, f, indent=2)
EOF

# Update the config
python /tmp/update_claude_config.py "$CONFIG_PATH" "$SERVER_PATH"
rm /tmp/update_claude_config.py

echo "✅ Successfully added bpftrace MCP server to Claude Desktop"
echo ""
echo "📋 Next steps:"
echo "1. Make sure you have bpftrace installed:"
echo "   Ubuntu/Debian: sudo apt-get install bpftrace"
echo "   Fedora: sudo dnf install bpftrace"
echo ""
echo "2. Install Python dependencies:"
echo "   pip install fastmcp"
echo ""
echo "3. Restart Claude Desktop"
echo ""
echo "4. The bpftrace tools will be available in Claude!"
echo ""
echo "⚠️  Note: The server uses sudo with password '123456' by default."
echo "   Update server.py line 169 for production use."