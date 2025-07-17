# Setting up bpftrace MCP Server with Claude Desktop

## Quick Setup (Automatic)

Run the setup script:
```bash
./setup_claude.sh
```

This script will:
- Detect your OS (macOS or Linux)
- Find the correct Claude Desktop config location
- Add the bpftrace MCP server configuration
- Create a backup of your existing config

## Manual Setup

If you prefer to configure manually or the script doesn't work:

### 1. Find your Claude Desktop config file

**macOS:**
```bash
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Linux:**
```bash
~/.config/claude/claude_desktop_config.json
```

### 2. Add the server configuration

Open the config file and add this to the `mcpServers` section:

```json
{
  "mcpServers": {
    "bpftrace": {
      "command": "python",
      "args": ["/absolute/path/to/MCPtrace/server.py"],
      "env": {}
    }
  }
}
```

Replace `/absolute/path/to/MCPtrace/server.py` with the actual path to your server.py file.

### 3. Install dependencies

```bash
# Install Python package
pip install fastmcp

# Install bpftrace (Ubuntu/Debian)
sudo apt-get install bpftrace

# Install bpftrace (Fedora)
sudo dnf install bpftrace
```

### 4. Restart Claude Desktop

After making changes, completely quit and restart Claude Desktop.

## Verifying the Setup

Once Claude Desktop restarts, you can verify the server is working by asking Claude:

- "Can you list available bpftrace probes?"
- "Show me bpftrace helper functions"
- "Run a simple bpftrace program to trace file opens"

## Troubleshooting

### Server not appearing in Claude

1. Check the config file syntax is valid JSON
2. Ensure the path to server.py is absolute, not relative
3. Check Claude Desktop logs for errors

### Permission errors

The server needs sudo access to run bpftrace. Currently uses password "123456" (see server.py:169).

### Python not found

Make sure Python is in your PATH, or use the full path to Python in the config:
```json
"command": "/usr/bin/python3"
```

## Security Note

⚠️ The current implementation uses a hardcoded sudo password. For production use, consider:
- Using passwordless sudo for bpftrace
- Implementing proper authentication
- Running the server with appropriate permissions