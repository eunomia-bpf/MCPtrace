# MCP Server Design for bpftrace - Minimal Implementation

## Overview

This document outlines a minimal MCP (Model Context Protocol) server implementation for bpftrace, focusing on essential functionality: listing probes, listing helpers, executing programs, and retrieving buffered results.

## Core Design Principles

1. **Minimal API Surface**: Start with only essential operations
2. **Buffered Output**: Store bpftrace output for later retrieval
3. **Simple Error Handling**: Clear success/error responses
4. **Stateless Operations**: Each operation is independent

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   AI Client     │────▶│   MCP Server     │────▶│    bpftrace     │
│                 │◀────│                  │◀────│                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                              │
                              ▼
                        ┌──────────────┐
                        │Output Buffer │
                        │   Storage    │
                        └──────────────┘
```

## MCP Tools Definition

### 1. list_probes
Lists available bpftrace probes with optional filtering.

```json
{
  "name": "list_probes",
  "description": "List available bpftrace probes",
  "inputSchema": {
    "type": "object",
    "properties": {
      "filter": {
        "type": "string",
        "description": "Optional filter pattern (e.g., 'syscalls:*open*')"
      }
    }
  }
}
```

**Example Response:**
```json
{
  "probes": [
    "tracepoint:syscalls:sys_enter_open",
    "tracepoint:syscalls:sys_exit_open",
    "tracepoint:syscalls:sys_enter_openat",
    "kprobe:vfs_open"
  ]
}
```

### 2. list_helpers
Lists available bpftrace built-in functions and helpers.

```json
{
  "name": "list_helpers",
  "description": "List available bpftrace helper functions",
  "inputSchema": {
    "type": "object",
    "properties": {}
  }
}
```

**Example Response:**
```json
{
  "helpers": [
    {
      "name": "printf",
      "description": "Print formatted output"
    },
    {
      "name": "time",
      "description": "Current timestamp"
    },
    {
      "name": "str",
      "description": "Convert to string"
    },
    {
      "name": "comm",
      "description": "Current process name"
    }
  ]
}
```

### 3. exec_program
Execute a bpftrace program with buffered output.

```json
{
  "name": "exec_program",
  "description": "Execute a bpftrace program",
  "inputSchema": {
    "type": "object",
    "properties": {
      "program": {
        "type": "string",
        "description": "The bpftrace program to execute"
      },
      "timeout": {
        "type": "integer",
        "description": "Execution timeout in seconds (default: 10, max: 60)"
      }
    },
    "required": ["program"]
  }
}
```

**Success Response:**
```json
{
  "status": "success",
  "execution_id": "exec_123456",
  "message": "Program started successfully"
}
```

**Error Response:**
```json
{
  "status": "error",
  "message": "Syntax error at line 2: unexpected token '}'"
}
```

### 4. get_result
Retrieve buffered output from a previous execution.

```json
{
  "name": "get_result",
  "description": "Get buffered output from a bpftrace execution",
  "inputSchema": {
    "type": "object",
    "properties": {
      "execution_id": {
        "type": "string",
        "description": "The execution ID returned by exec_program"
      },
      "offset": {
        "type": "integer",
        "description": "Start reading from this line number (default: 0)"
      },
      "limit": {
        "type": "integer",
        "description": "Maximum lines to return (default: 1000)"
      }
    },
    "required": ["execution_id"]
  }
}
```

**Response:**
```json
{
  "execution_id": "exec_123456",
  "status": "running|completed|failed",
  "lines_total": 150,
  "lines_returned": 100,
  "output": [
    "Attaching 1 probe...",
    "nginx opened /var/log/nginx/access.log",
    "nginx opened /etc/nginx/nginx.conf"
  ],
  "has_more": true
}
```

## Implementation Details

### Output Buffer Management

```python
class ExecutionBuffer:
    def __init__(self, execution_id: str, max_lines: int = 10000):
        self.execution_id = execution_id
        self.lines = []
        self.status = "running"
        self.max_lines = max_lines
        self.creation_time = time.time()
        self.completion_time = None
```

### Process Management

- Use subprocess to run bpftrace
- Capture stdout/stderr in real-time
- Enforce timeout limits
- Clean up zombie processes

### Error Handling

1. **Syntax Errors**: Return immediately with error message
2. **Permission Errors**: Clear message about required privileges
3. **Timeout**: Mark execution as failed, keep partial output
4. **Resource Limits**: Fail gracefully when buffer is full

## Security Considerations

### Minimal Security Model

1. **No Script Validation**: Trust the AI client to generate safe scripts
2. **Process Isolation**: Run bpftrace in separate process
3. **Resource Limits**:
   - Max execution time: 60 seconds
   - Max output buffer: 10,000 lines per execution
   - Max concurrent executions: 5

## Implementation Framework

### Python with FastMCP

```python
from fastmcp import FastMCP, Tool
import subprocess
import asyncio
import uuid

mcp = FastMCP("bpftrace-server")

# Global buffer storage
execution_buffers = {}

@mcp.tool()
async def list_probes(filter: str = None) -> dict:
    cmd = ["bpftrace", "-l"]
    if filter:
        cmd.append(filter)
    
    result = await run_command(cmd)
    return {"probes": result.stdout.strip().split('\n')}

@mcp.tool()
async def exec_program(program: str, timeout: int = 10) -> dict:
    execution_id = f"exec_{uuid.uuid4().hex[:8]}"
    
    # Start execution in background
    asyncio.create_task(
        run_bpftrace_program(execution_id, program, timeout)
    )
    
    return {
        "status": "success",
        "execution_id": execution_id,
        "message": "Program started successfully"
    }
```

## Usage Examples

### Example 1: List System Call Probes
```python
# Client request
list_probes(filter="syscalls:*read*")

# Returns available read-related syscalls
```

### Example 2: Simple Trace Program
```python
# Client request
exec_program(
    program='tracepoint:syscalls:sys_enter_open { printf("%s opened %s\\n", comm, str(args->filename)); }',
    timeout=5
)

# Returns execution_id for later retrieval
```

### Example 3: Get Results
```python
# Client request
get_result(execution_id="exec_123456", offset=0, limit=100)

# Returns buffered output lines
```

## Future Enhancements (Not in V1)

1. **Streaming Output**: Real-time output via SSE
2. **Probe Documentation**: Include descriptions for each probe
3. **Script Templates**: Pre-defined safe script patterns
4. **Output Parsing**: Structured output formats (JSON, CSV)
5. **Persistent Storage**: Save executions to disk

## Conclusion

This minimal MCP server design provides essential bpftrace functionality through a simple, focused API. The design prioritizes:

- Clear separation of execution and result retrieval
- Simple error handling
- Buffered output management
- Minimal security overhead

The implementation can be extended incrementally as needs grow, while maintaining backward compatibility with the core tools.