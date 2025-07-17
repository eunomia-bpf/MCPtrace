# Adding bpftrace MCP Server to Claude Code

## Quick Setup

### 1. Install the server dependencies

```bash
# Navigate to the MCPtrace directory
cd /path/to/MCPtrace

# Install Python dependencies
pip install fastmcp

# Ensure bpftrace is installed
sudo apt-get install bpftrace  # Ubuntu/Debian
# or
sudo dnf install bpftrace      # Fedora
```

### 2. Add the server to Claude Code

Run this command from your project directory:

```bash
claude mcp add bpftrace python /absolute/path/to/MCPtrace/server.py
```

Replace `/absolute/path/to/MCPtrace/server.py` with the actual path to your server.py file.

### 3. Configure server scope (optional)

By default, the server is added at "local" scope (only for current project). You can change this:

```bash
# Add for all your projects (user scope)
claude mcp add --scope user bpftrace python /absolute/path/to/MCPtrace/server.py

# Add to project configuration (shared with team)
claude mcp add --scope project bpftrace python /absolute/path/to/MCPtrace/server.py
```

## Verifying the Setup

### Check server status
```bash
claude mcp
```

This shows all configured MCP servers and their status.

### Test the server

In Claude Code, try these commands:
- "List available bpftrace probes"
- "Show me bpftrace helper functions"
- "Execute a simple bpftrace program to trace system calls"

## Managing the Server

### Remove the server
```bash
claude mcp remove bpftrace
```

### Update server configuration
```bash
# Remove and re-add with new settings
claude mcp remove bpftrace
claude mcp add bpftrace python /new/path/to/server.py
```

## Troubleshooting

### Server not starting

1. Check the server path is absolute:
   ```bash
   # Get absolute path
   cd /path/to/MCPtrace
   pwd  # Use this path in the add command
   ```

2. Test the server standalone:
   ```bash
   python /path/to/MCPtrace/test_server.py
   ```

3. Check Claude Code logs for errors

### Permission issues

The server needs sudo access for bpftrace. Currently uses password "123456" (hardcoded in server.py:169).

For production use, consider:
- Configuring passwordless sudo for bpftrace
- Running Claude Code with appropriate permissions
- Modifying the server to use a different authentication method

### Python environment issues

If you're using a virtual environment:
```bash
# Use the full path to Python in your venv
claude mcp add bpftrace /path/to/venv/bin/python /path/to/MCPtrace/server.py
```

## Advanced Configuration

### Set environment variables
```bash
claude mcp add -e MY_VAR=value bpftrace python /path/to/server.py
```

### Adjust timeout
```bash
export MCP_TIMEOUT=30  # Set 30 second timeout
claude mcp add bpftrace python /path/to/server.py
```

## Security Notes

⚠️ **Important**: The current implementation uses a hardcoded sudo password ("123456"). This is insecure for production use. Consider:

1. Using passwordless sudo for the bpftrace command:
   ```bash
   # Add to /etc/sudoers (use visudo)
   your_username ALL=(ALL) NOPASSWD: /usr/bin/bpftrace
   ```

2. Modifying server.py to remove the hardcoded password

3. Running the MCP server with appropriate system permissions