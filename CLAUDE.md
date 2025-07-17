# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a bpftrace MCP (Model Context Protocol) server that provides AI assistants with access to Linux kernel tracing capabilities. The server acts as a bridge between AI models and the bpftrace tool, enabling kernel-level system observation and debugging.

The server is implemented in Rust using the `rmcp` crate from the rust-sdk.

## Development Commands

### Setup and Dependencies
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the server
cargo build --release
```

### Running the Server
```bash
# Direct execution
./target/release/bpftrace-mcp-server

# Through cargo
cargo run --release
```

### Testing
```bash
# Run unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

## Architecture

The server is built with the rmcp crate and provides four main tools:

1. **list_probes** - Lists available bpftrace probes with optional filtering
2. **list_helpers** - Shows bpftrace helper functions
3. **exec_program** - Executes bpftrace programs asynchronously
4. **get_result** - Retrieves execution results

### Key Components

- **ExecutionBuffer** (src/main.rs): Manages async output collection from bpftrace processes
- **DashMap execution_buffers**: Thread-safe concurrent storage for active executions
- **Cleanup task**: Background task that removes old buffers after 1 hour
- **BpftraceServer**: Main server struct implementing the MCP ServerHandler trait

### Security Considerations

- Requires sudo access for bpftrace execution
- Server prompts for sudo password at startup (cached for session)
- Alternative: Configure passwordless sudo for bpftrace: `sudo visudo` then add `username ALL=(ALL) NOPASSWD: /usr/bin/bpftrace`
- No script validation - relies on AI to generate safe bpftrace programs
- Resource limits: 60s execution timeout, 10k line output buffer

## Important Implementation Details

1. **Async Execution Model**: All bpftrace programs run asynchronously with output buffered in memory
2. **Process Management**: Uses subprocess with proper signal handling (SIGTERM) for cleanup
3. **Error Handling**: Basic error handling with descriptive messages returned to the client
4. **No Persistence**: Execution results are only stored in memory and cleaned up after 1 hour

## Common Development Tasks

When modifying the server:
- Run `cargo build --release` after changes
- Test all four functions manually or with test cases
- Ensure proper error handling for subprocess failures
- Maintain async patterns for non-blocking operations
- Consider security implications of any changes to bpftrace execution
- Use `cargo clippy` for linting and `cargo fmt` for formatting

## Dependencies

- Rust (latest stable version)
- rmcp crate with server and transport-io features
- tokio async runtime
- bpftrace (system requirement, must be installed separately)