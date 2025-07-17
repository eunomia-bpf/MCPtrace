#!/usr/bin/env python3
"""
Test script for the bpftrace MCP server
"""

import asyncio
import json
from server import mcp, list_probes, list_helpers, exec_program, get_result

async def test_server():
    print("Testing bpftrace MCP server...\n")
    
    # Test 1: List probes
    print("1. Testing list_probes...")
    result = await list_probes(filter="syscalls:*open*")
    print(f"Found {result.get('count', 0)} probes matching 'syscalls:*open*'")
    if result.get('probes'):
        print(f"First few probes: {result['probes'][:5]}")
    print()
    
    # Test 2: List helpers
    print("2. Testing list_helpers...")
    result = await list_helpers()
    print(f"Found {result['count']} helper functions")
    print(f"First few helpers: {[h['name'] for h in result['helpers'][:5]]}")
    print()
    
    # Test 3: Execute a simple program
    print("3. Testing exec_program...")
    program = 'BEGIN { printf("Hello from bpftrace!\\n"); exit(); }'
    result = await exec_program(program, timeout=5)
    print(f"Execution result: {result}")
    
    if result['status'] == 'success':
        exec_id = result['execution_id']
        print(f"Execution ID: {exec_id}")
        
        # Wait a bit for execution to complete
        await asyncio.sleep(2)
        
        # Test 4: Get results
        print("\n4. Testing get_result...")
        result = await get_result(exec_id)
        print(f"Status: {result['status']}")
        print(f"Output lines: {result['lines_total']}")
        if result['output']:
            print("Output:")
            for line in result['output']:
                print(f"  {line}")
    print()
    
    # Test 5: Execute a syscall trace (short duration)
    print("5. Testing syscall trace...")
    program = '''
    tracepoint:syscalls:sys_enter_openat
    {
        printf("%s opened %s\\n", comm, str(args->filename));
    }
    '''
    result = await exec_program(program, timeout=3)
    print(f"Execution result: {result}")
    
    if result['status'] == 'success':
        exec_id = result['execution_id']
        print(f"Execution ID: {exec_id}")
        
        # Wait for trace to run
        print("Tracing for 3 seconds...")
        await asyncio.sleep(4)
        
        # Get results
        result = await get_result(exec_id)
        print(f"Status: {result['status']}")
        print(f"Captured {result['lines_total']} lines")
        if result['output'][:10]:  # Show first 10 lines
            print("First few lines:")
            for line in result['output'][:10]:
                print(f"  {line}")

if __name__ == "__main__":
    asyncio.run(test_server())