# bpftrace MCP Server: generate eBPF to trace linux kernel

A minimal MCP (Model Context Protocol) server that provides AI assistants with access to bpftrace kernel tracing capabilities.

**Now implemented in Rust** using the `rmcp` crate for better performance and type safety. The Python implementation is still available in the git history.

## Features

- **list_probes**: List available bpftrace probes with optional filtering
- **list_helpers**: Get information about bpftrace helper functions
- **exec_program**: Execute bpftrace programs with buffered output
- **get_result**: Retrieve execution results asynchronously

## Installation

### Prerequisites

1. Install Rust (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Ensure bpftrace is installed:
```bash
sudo apt-get install bpftrace  # Ubuntu/Debian
# or
sudo dnf install bpftrace      # Fedora
```

3. Build the server:
```bash
cargo build --release
```

### Quick Setup

Use our automated setup scripts:

- **Claude Desktop**: `./setup/setup_claude.sh`
- **Claude Code**: `./setup/setup_claude_code.sh`

For detailed setup instructions and manual configuration, see [setup/SETUP.md](./setup/SETUP.md).

## Running the Server

### Direct Execution
```bash
./target/release/bpftrace-mcp-server
```

### Through Cargo
```bash
cargo run --release
```

### Manual Configuration

For manual setup instructions for Claude Desktop or Claude Code, see [setup/SETUP.md](./setup/SETUP.md).

## Usage Examples

### List System Call Probes
```python
await list_probes(filter="syscalls:*read*")
```

### Get BPF System Information
```python
info = await bpf_info()
# Returns system info, kernel helpers, features, map types, and probe types
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
- **Password Handling**: Create a `.env` file with your sudo password:
  ```bash
  echo "BPFTRACE_PASSWD=your_sudo_password" > .env
  ```
- **Alternative**: Configure passwordless sudo for bpftrace:
  ```bash
  sudo visudo
  # Add: your_username ALL=(ALL) NOPASSWD: /usr/bin/bpftrace
  ```
- No script validation - trust the AI client to generate safe scripts
- Resource limits: 60s max execution, 10k lines buffer
- See [SECURITY.md](./SECURITY.md) for detailed security configuration

## Architecture

The Rust server uses:
- Tokio async runtime for concurrent operations
- Subprocess management for bpftrace execution
- DashMap for thread-safe in-memory buffering
- Automatic cleanup of old buffers
- rmcp crate for MCP protocol implementation

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