# bpftrace MCP Server: generate eBPF to trace linux kernel

A minimal MCP (Model Context Protocol) server that provides AI assistants with access to bpftrace kernel tracing capabilities.

## Features

- **list_probes**: List available bpftrace probes with optional filtering
- **list_helpers**: Get information about bpftrace helper functions
- **exec_program**: Execute bpftrace programs with buffered output
- **get_result**: Retrieve execution results asynchronously

## Installation

### Prerequisites

1. Ensure bpftrace is installed:
```bash
sudo apt-get install bpftrace  # Ubuntu/Debian
# or
sudo dnf install bpftrace      # Fedora
```

2. Install Python dependencies:
```bash
pip install fastmcp
# or
pip install -r requirements.txt
```

### Quick Setup

Use our automated setup scripts:

- **Claude Desktop**: `./setup/setup_claude.sh`
- **Claude Code**: `./setup/setup_claude_code.sh`

For detailed setup instructions and manual configuration, see [setup/SETUP.md](./setup/SETUP.md).

## Running the Server

### Standalone Mode (for testing)
```bash
python test_server.py
```

### FastMCP Development Tools (recommended for development)
```bash
fastmcp dev server.py
```

### Manual Configuration

For manual setup instructions for Claude Desktop or Claude Code, see [setup/SETUP.md](./setup/SETUP.md).

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
- **Password Handling**: The server prompts for sudo password at startup and caches it for the session
- **Alternative**: Configure passwordless sudo for bpftrace:
  ```bash
  sudo visudo
  # Add: your_username ALL=(ALL) NOPASSWD: /usr/bin/bpftrace
  ```
- No script validation - trust the AI client to generate safe scripts
- Resource limits: 60s max execution, 10k lines buffer
- See [SECURITY.md](./SECURITY.md) for detailed security configuration

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

## Documentation

- [Setup Guide](./setup/SETUP.md) - Detailed installation and configuration
- [Claude Code Setup](./setup/CLAUDE_CODE_SETUP.md) - Claude Code specific instructions
- [CLAUDE.md](./CLAUDE.md) - Development guidance for AI assistants
- [Design Document](./doc/mcp-bpftrace-design.md) - Architecture and design details

## Future Enhancements

- Add SSE transport for real-time streaming
- Implement proper authentication
- Add script validation and sandboxing
- Support for saving/loading trace sessions
- Integration with eBPF programs