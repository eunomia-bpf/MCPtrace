# GPTtrace Features Integration Plan for MCPtrace

## Overview

This document outlines specific features from GPTtrace that should be integrated into MCPtrace to enhance its capabilities while maintaining MCP protocol compliance.

## Priority 1: Essential Features (Immediate Implementation)

### 1. Example Database with Semantic Search

**GPTtrace Implementation**:
```python
# From gpttrace/examples.py
loader = JSONLoader(file_path='./tools/examples.json', ...)
embeddings = OpenAIEmbeddings()
db = FAISS.from_documents(documents, embeddings)
results = db.search(query, search_type='similarity')
```

**MCPtrace Implementation Plan**:
```rust
// New tool: search_examples
#[tool(description = "Search for relevant bpftrace examples using semantic similarity")]
async fn search_examples(
    &self,
    Parameters(SearchRequest { query, limit }): Parameters<SearchRequest>,
) -> Result<CallToolResult> {
    // Load examples from JSON (same format as GPTtrace)
    // Return top N similar examples with their programs
    // Include metadata: use case, complexity, kernel version
}
```

**Benefits**:
- AI can find relevant examples without exact keyword matching
- Reduces need for AI to generate programs from scratch
- Improves accuracy by providing tested examples

### 2. Program Validation and Safety Checks

**GPTtrace Feature**:
```python
# Interactive confirmation before execution
user_input = input("Enter 'y' to proceed: ")
if user_input.lower() != 'y':
    print("Aborting...")
    exit()
```

**MCPtrace Implementation**:
```rust
#[tool(description = "Validate a bpftrace program for syntax and safety")]
async fn validate_program(
    &self,
    Parameters(ValidateRequest { program, safety_level }): Parameters<ValidateRequest>,
) -> Result<CallToolResult> {
    // Check syntax with bpftrace -d
    // Verify probe availability
    // Check for unsafe operations
    // Return validation report with warnings
}
```

### 3. Execution with Retry Logic

**GPTtrace Feature**:
```python
def execute(user_input, verbose=False, retry=5, previous_prompt=None, output=None):
    if retry == 0:
        print("Retry times exceeded...")
    # Retry with error feedback
```

**MCPtrace Enhancement**:
```rust
#[tool(description = "Execute program with automatic retry on failure")]
async fn exec_with_retry(
    &self,
    Parameters(RetryRequest { 
        program, 
        max_retries, 
        timeout,
        fix_errors 
    }): Parameters<RetryRequest>,
) -> Result<CallToolResult> {
    // Execute program
    // On syntax error, return error details for AI to fix
    // Track retry attempts and modifications
}
```

## Priority 2: Enhanced Functionality (Week 2-3)

### 4. Result Analysis and Summarization

**GPTtrace Feature**:
```python
def construct_prompt_for_explain(text: str, output: str) -> str:
    return f"""
    please explain the output of the previous bpftrace result:
    {output}
    The original user request is: {text}
    """
```

**MCPtrace Tool**:
```rust
#[tool(description = "Analyze trace output and provide structured summary")]
async fn analyze_output(
    &self,
    Parameters(AnalyzeRequest { 
        execution_id,
        output_type,  // histogram, counter, trace, etc.
        focus_areas   // specific metrics to highlight
    }): Parameters<AnalyzeRequest>,
) -> Result<CallToolResult> {
    // Parse output based on type
    // Extract key metrics
    // Identify anomalies or patterns
    // Return structured analysis
}
```

### 5. Template-Based Program Generation

**New Feature** (inspired by GPTtrace's examples):
```rust
#[tool(description = "Generate bpftrace program from template")]
async fn generate_from_template(
    &self,
    Parameters(TemplateRequest { 
        template_name,  // e.g., "syscall_counter", "latency_histogram"
        parameters      // template-specific params
    }): Parameters<TemplateRequest>,
) -> Result<CallToolResult> {
    // Load template
    // Substitute parameters
    // Validate generated program
    // Return ready-to-run program
}
```

### 6. Probe Discovery Assistant

**Enhancement** beyond GPTtrace:
```rust
#[tool(description = "Discover relevant probes for a tracing goal")]
async fn discover_probes(
    &self,
    Parameters(DiscoverRequest { 
        goal,        // what user wants to trace
        probe_types  // kprobe, uprobe, tracepoint, etc.
    }): Parameters<DiscoverRequest>,
) -> Result<CallToolResult> {
    // List relevant probes
    // Group by subsystem
    // Include probe descriptions
    // Suggest probe combinations
}
```

## Priority 3: Advanced Features (Month 2)

### 7. Multi-Tool Execution Support

**GPTtrace Feature**:
```python
# Support for bcc tools
def cmd(cmd_name: str, query: str, verbose: bool):
    # Execute pre-built bcc tools
```

**MCPtrace Enhancement**:
```rust
#[tool(description = "Execute bcc tool for common tracing tasks")]
async fn exec_bcc_tool(
    &self,
    Parameters(BccRequest { 
        tool_name,   // execsnoop, opensnoop, etc.
        arguments,
        timeout
    }): Parameters<BccRequest>,
) -> Result<CallToolResult> {
    // Map to appropriate bcc tool
    // Execute with proper arguments
    // Return structured output
}
```

### 8. Execution History and Learning

**New Feature**:
```rust
#[tool(description = "Search execution history for similar traces")]
async fn search_history(
    &self,
    Parameters(HistoryRequest { 
        query,
        include_failed,
        time_range
    }): Parameters<HistoryRequest>,
) -> Result<CallToolResult> {
    // Search past executions
    // Include programs, results, and corrections
    // Learn from successful patterns
}
```

## Implementation Strategy

### Phase 1: Foundation (Week 1)
1. Add examples.json from GPTtrace
2. Implement search_examples tool
3. Add validate_program tool
4. Update exec_program with basic retry

### Phase 2: Intelligence (Week 2)
1. Implement analyze_output tool
2. Add template system
3. Create probe discovery tool
4. Enhance error messages with suggestions

### Phase 3: Integration (Week 3)
1. Add bcc tool support
2. Implement execution history
3. Create combined workflows
4. Add performance profiling

### Phase 4: Polish (Week 4)
1. Optimize example search
2. Add caching for common queries
3. Improve error recovery
4. Create comprehensive tests

## Migration Guide for GPTtrace Users

### For Users
1. Install MCPtrace server
2. Configure Claude Desktop/Code
3. Use natural language as before
4. Get enhanced features through MCP

### For Developers
1. Examples format remains compatible
2. Add new examples to shared database
3. Use MCP tools instead of direct API calls
4. Contribute templates and patterns

## Success Metrics

1. **Feature Parity**: All GPTtrace capabilities available
2. **Performance**: Faster execution with async model
3. **Reliability**: 90%+ success rate on first attempt
4. **Usability**: Reduced prompt engineering needed
5. **Safety**: No unsafe executions without explicit override

## Conclusion

By integrating these GPTtrace features, MCPtrace will combine the best of both worlds:
- GPTtrace's AI-native design and user experience
- MCPtrace's robust architecture and standard protocol

This creates a powerful, safe, and user-friendly kernel tracing solution that works seamlessly with modern AI assistants.