# bpftrace MCP Server: generate eBPF to trace linux kernel

A minimal MCP (Model Context Protocol) server that provides AI assistants with access to bpftrace kernel tracing capabilities.

## Features

- **list_probes**: List available bpftrace probes with optional filtering
- **list_helpers**: Get information about bpftrace helper functions
- **exec_program**: Execute bpftrace programs with buffered output
- **get_result**: Retrieve execution results asynchronously

## Installation

1. Install dependencies:
```bash
pip install mcp
# or with uv:
uv pip install mcp
```

2. Ensure bpftrace is installed:
```bash
sudo apt-get install bpftrace  # Ubuntu/Debian
# or
sudo dnf install bpftrace      # Fedora
```

## Running the Server

### Standalone Mode (for testing)
```bash
python test_server.py
```

### MCP Inspector (recommended for development)
```bash
uv run mcp dev server.py
```

### With Claude Desktop

1. Edit your Claude Desktop configuration:
```bash
# On macOS:
nano ~/Library/Application\ Support/Claude/claude_desktop_config.json

# On Linux:
nano ~/.config/claude/claude_desktop_config.json
```

2. Add the server configuration:
```json
{
  "mcpServers": {
    "bpftrace": {
      "command": "python",
      "args": ["/path/to/MCPtrace/server.py"],
      "env": {}
    }
  }
}
```

3. Restart Claude Desktop

## Usage Examples

### List System Call Probes
```python
await list_probes(filter="syscalls:*read*")
```

### Execute a Simple Trace
```python
result = await exec_program(
    'tracepoint:syscalls:sys_enter_open { printf("%s\\n", comm); }',
    timeout=10
)
exec_id = result["execution_id"]
```

### Get Results
```python
output = await get_result(exec_id)
print(output["output"])
```

## Security Notes

- The server requires sudo access for bpftrace
- Currently uses hardcoded password (123456) - modify for production
- No script validation - trust the AI client to generate safe scripts
- Resource limits: 60s max execution, 10k lines buffer

## Architecture

The server uses:
- AsyncIO for concurrent operations
- Subprocess management for bpftrace execution
- In-memory buffering for output storage
- Automatic cleanup of old buffers

## Limitations

- No real-time streaming (use get_result to poll)
- Simple password handling (improve for production)
- No persistent storage of executions
- Basic error handling

## Future Enhancements

- Add SSE transport for real-time streaming
- Implement proper authentication
- Add script validation and sandboxing
- Support for saving/loading trace sessions
- Integration with eBPF programs