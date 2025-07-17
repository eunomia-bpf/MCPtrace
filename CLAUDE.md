# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a bpftrace MCP (Model Context Protocol) server that provides AI assistants with access to Linux kernel tracing capabilities. The server acts as a bridge between AI models and the bpftrace tool, enabling kernel-level system observation and debugging.

## Development Commands

### Setup and Dependencies
```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Linux/Mac

# Install dependencies
pip install -r requirements.txt
# or
pip install fastmcp
```

### Running the Server
```bash
# Run test script (includes example usage)
python test_server.py

# Run with MCP dev tools (recommended for development)
uv run mcp dev server.py

# Direct execution
python server.py
```

### Testing
There is no formal test framework. Use `test_server.py` for manual testing of all four server functions.

## Architecture

The server is built with FastMCP and provides four main tools:

1. **list_probes** - Lists available bpftrace probes (server.py:53-79)
2. **list_helpers** - Shows bpftrace helper functions (server.py:81-104)
3. **exec_program** - Executes bpftrace programs asynchronously (server.py:149-199)
4. **get_result** - Retrieves execution results (server.py:201-217)

### Key Components

- **ExecutionBuffer** (server.py:106-147): Manages async output collection from bpftrace processes
- **Global execution_buffers**: In-memory storage for active executions
- **Cleanup task**: Removes old buffers after 1 hour (server.py:27-40)

### Security Considerations

- Requires sudo access for bpftrace execution
- Currently uses hardcoded password "123456" in server.py:169 (needs improvement for production)
- No script validation - relies on AI to generate safe bpftrace programs
- Resource limits: 60s execution timeout, 10k line output buffer

## Important Implementation Details

1. **Async Execution Model**: All bpftrace programs run asynchronously with output buffered in memory
2. **Process Management**: Uses subprocess with proper signal handling (SIGTERM) for cleanup
3. **Error Handling**: Basic error handling with descriptive messages returned to the client
4. **No Persistence**: Execution results are only stored in memory and cleaned up after 1 hour

## Common Development Tasks

When modifying the server:
- Test all four functions using `test_server.py` after changes
- Ensure proper error handling for subprocess failures
- Maintain async patterns for non-blocking operations
- Consider security implications of any changes to bpftrace execution

## Dependencies

- Python >=3.10
- fastmcp (only external dependency)
- bpftrace (system requirement, must be installed separately)