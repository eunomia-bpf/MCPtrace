#!/usr/bin/env python3
"""
MCP Server for bpftrace - Minimal Implementation
Provides tools for listing probes, helpers, and executing bpftrace programs
"""

import asyncio
import subprocess
import uuid
import time
from typing import Optional, Dict, List, Any
from collections import defaultdict
import shlex
import os
import signal

from fastmcp import FastMCP

# Initialize MCP server
mcp = FastMCP("bpftrace-server")

# Global storage for execution buffers
execution_buffers: Dict[str, 'ExecutionBuffer'] = {}

# Cleanup old buffers every 5 minutes
BUFFER_CLEANUP_INTERVAL = 300
BUFFER_MAX_AGE = 3600  # 1 hour

class ExecutionBuffer:
    """Stores output from a bpftrace execution"""
    def __init__(self, execution_id: str, max_lines: int = 10000):
        self.execution_id = execution_id
        self.lines: List[str] = []
        self.status = "running"  # running, completed, failed
        self.max_lines = max_lines
        self.creation_time = time.time()
        self.completion_time: Optional[float] = None
        self.error_message: Optional[str] = None
        self.process: Optional[asyncio.subprocess.Process] = None
        
    def add_line(self, line: str):
        """Add a line to the buffer"""
        if len(self.lines) < self.max_lines:
            self.lines.append(line)
        elif len(self.lines) == self.max_lines:
            self.lines.append(f"[Output truncated at {self.max_lines} lines]")
            
    def mark_completed(self):
        """Mark execution as completed"""
        self.status = "completed"
        self.completion_time = time.time()
        
    def mark_failed(self, error: str):
        """Mark execution as failed"""
        self.status = "failed"
        self.completion_time = time.time()
        self.error_message = error


async def cleanup_old_buffers():
    """Periodically clean up old execution buffers"""
    while True:
        await asyncio.sleep(BUFFER_CLEANUP_INTERVAL)
        current_time = time.time()
        to_remove = []
        
        for exec_id, buffer in execution_buffers.items():
            if current_time - buffer.creation_time > BUFFER_MAX_AGE:
                to_remove.append(exec_id)
                
        for exec_id in to_remove:
            del execution_buffers[exec_id]


async def run_bpftrace_program(execution_id: str, program: str, timeout: int):
    """Run a bpftrace program and capture output"""
    buffer = execution_buffers[execution_id]
    
    try:
        # Create the bpftrace command
        cmd = ["sudo", "bpftrace", "-e", program]
        
        # Start the process
        process = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE
        )
        
        buffer.process = process
        
        # Send password to sudo if needed
        process.stdin.write(b"123456\n")
        await process.stdin.drain()
        
        # Set up timeout
        timeout_task = asyncio.create_task(asyncio.sleep(timeout))
        
        # Read output line by line
        while True:
            # Check if timeout occurred
            if timeout_task.done():
                process.terminate()
                await asyncio.sleep(0.5)
                if process.returncode is None:
                    process.kill()
                buffer.add_line("[Execution timed out]")
                buffer.mark_failed("Timeout")
                break
                
            # Try to read a line
            try:
                line = await asyncio.wait_for(
                    process.stdout.readline(), 
                    timeout=0.1
                )
                
                if not line:
                    # Process ended
                    break
                    
                decoded_line = line.decode('utf-8').rstrip()
                buffer.add_line(decoded_line)
                
            except asyncio.TimeoutError:
                # No output available, continue
                continue
                
            # Check if process ended
            if process.returncode is not None:
                break
        
        # Read any remaining stderr
        stderr = await process.stderr.read()
        if stderr:
            stderr_text = stderr.decode('utf-8').strip()
            if stderr_text and not stderr_text.startswith("[sudo] password"):
                buffer.add_line(f"[Error] {stderr_text}")
                buffer.mark_failed(stderr_text)
                return
                
        # Cancel timeout if still running
        if not timeout_task.done():
            timeout_task.cancel()
            
        # Mark as completed if not already failed
        if buffer.status == "running":
            buffer.mark_completed()
            
    except Exception as e:
        buffer.mark_failed(str(e))
        buffer.add_line(f"[Exception] {str(e)}")


@mcp.tool()
async def list_probes(filter: Optional[str] = None) -> Dict[str, Any]:
    """
    List available bpftrace probes with optional filtering.
    
    Args:
        filter: Optional filter pattern (e.g., 'syscalls:*open*')
    
    Returns:
        Dictionary containing list of matching probes
    """
    try:
        cmd = ["sudo", "bpftrace", "-l"]
        if filter:
            cmd.append(filter)
            
        process = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            stdin=asyncio.subprocess.PIPE
        )
        
        # Send password
        process.stdin.write(b"123456\n")
        await process.stdin.drain()
        
        stdout, stderr = await process.communicate()
        
        if process.returncode != 0:
            return {
                "error": stderr.decode('utf-8').strip(),
                "probes": []
            }
            
        # Parse output
        probes = [
            line.strip() 
            for line in stdout.decode('utf-8').strip().split('\n')
            if line.strip() and not line.startswith("[sudo]")
        ]
        
        return {
            "probes": probes,
            "count": len(probes)
        }
        
    except Exception as e:
        return {
            "error": str(e),
            "probes": []
        }


@mcp.tool()
async def list_helpers() -> Dict[str, Any]:
    """
    List available bpftrace helper functions.
    
    Returns:
        Dictionary containing list of helper functions with descriptions
    """
    # bpftrace doesn't have a command to list helpers, so we provide a curated list
    helpers = [
        {"name": "printf", "description": "Print formatted output"},
        {"name": "time", "description": "Current timestamp (nanoseconds since boot)"},
        {"name": "str", "description": "Convert to string (for char arrays)"},
        {"name": "comm", "description": "Current process name"},
        {"name": "pid", "description": "Process ID"},
        {"name": "tid", "description": "Thread ID"},
        {"name": "uid", "description": "User ID"},
        {"name": "gid", "description": "Group ID"},
        {"name": "nsecs", "description": "Nanoseconds since boot"},
        {"name": "kstack", "description": "Kernel stack trace"},
        {"name": "ustack", "description": "User stack trace"},
        {"name": "arg0...argN", "description": "Function arguments"},
        {"name": "retval", "description": "Return value (in return probes)"},
        {"name": "cpu", "description": "Current CPU"},
        {"name": "curtask", "description": "Current task struct"},
        {"name": "rand", "description": "Random number"},
        {"name": "cgroup", "description": "Cgroup ID"},
        {"name": "kaddr", "description": "Kernel address for symbol"},
        {"name": "uaddr", "description": "User address for symbol"},
        {"name": "ntop", "description": "Convert IP address to string"},
        {"name": "reg", "description": "CPU register value"},
        {"name": "signal", "description": "Send signal to process"},
        {"name": "exit", "description": "Exit bpftrace"},
        {"name": "system", "description": "Execute shell command"},
        {"name": "cat", "description": "Print file contents"},
        {"name": "join", "description": "Join array elements"},
        {"name": "ksym", "description": "Resolve kernel address to symbol"},
        {"name": "usym", "description": "Resolve user address to symbol"},
        {"name": "kptr", "description": "Annotate kernel pointer"},
        {"name": "uptr", "description": "Annotate user pointer"},
        {"name": "sizeof", "description": "Size of type or expression"},
        {"name": "print", "description": "Print non-formatted output"},
        {"name": "clear", "description": "Clear a map"},
        {"name": "zero", "description": "Zero a map"},
        {"name": "hist", "description": "Print histogram"},
        {"name": "lhist", "description": "Print linear histogram"},
        {"name": "count", "description": "Count occurrences"},
        {"name": "sum", "description": "Sum values"},
        {"name": "min", "description": "Track minimum value"},
        {"name": "max", "description": "Track maximum value"},
        {"name": "avg", "description": "Calculate average"},
        {"name": "stats", "description": "Calculate statistics"},
    ]
    
    return {
        "helpers": helpers,
        "count": len(helpers)
    }


@mcp.tool()
async def exec_program(program: str, timeout: int = 10) -> Dict[str, Any]:
    """
    Execute a bpftrace program with buffered output.
    
    Args:
        program: The bpftrace program to execute
        timeout: Execution timeout in seconds (default: 10, max: 60)
    
    Returns:
        Dictionary with execution status and ID
    """
    # Validate timeout
    if timeout < 1:
        timeout = 1
    elif timeout > 60:
        timeout = 60
        
    # Generate execution ID
    execution_id = f"exec_{uuid.uuid4().hex[:8]}"
    
    # Create buffer
    buffer = ExecutionBuffer(execution_id)
    execution_buffers[execution_id] = buffer
    
    # Start execution in background
    asyncio.create_task(run_bpftrace_program(execution_id, program, timeout))
    
    # Give it a moment to check for syntax errors
    await asyncio.sleep(0.5)
    
    # Check if it failed immediately (syntax error)
    if buffer.status == "failed":
        return {
            "status": "error",
            "message": buffer.error_message or "Failed to start program"
        }
    
    return {
        "status": "success",
        "execution_id": execution_id,
        "message": "Program started successfully"
    }


@mcp.tool()
async def get_result(
    execution_id: str, 
    offset: int = 0, 
    limit: int = 1000
) -> Dict[str, Any]:
    """
    Get buffered output from a bpftrace execution.
    
    Args:
        execution_id: The execution ID returned by exec_program
        offset: Start reading from this line number (default: 0)
        limit: Maximum lines to return (default: 1000)
    
    Returns:
        Dictionary with execution status and output
    """
    if execution_id not in execution_buffers:
        return {
            "error": "Execution ID not found",
            "execution_id": execution_id
        }
        
    buffer = execution_buffers[execution_id]
    
    # Get the requested lines
    total_lines = len(buffer.lines)
    end_index = min(offset + limit, total_lines)
    output_lines = buffer.lines[offset:end_index]
    
    result = {
        "execution_id": execution_id,
        "status": buffer.status,
        "lines_total": total_lines,
        "lines_returned": len(output_lines),
        "output": output_lines,
        "has_more": end_index < total_lines
    }
    
    if buffer.error_message:
        result["error_message"] = buffer.error_message
        
    if buffer.completion_time:
        result["duration"] = buffer.completion_time - buffer.creation_time
        
    return result


# Start cleanup task when server starts
@mcp.server.on_initialize
async def on_initialize():
    """Initialize server and start background tasks"""
    asyncio.create_task(cleanup_old_buffers())


if __name__ == "__main__":
    # Run the server
    import sys
    mcp.run(transport="stdio")