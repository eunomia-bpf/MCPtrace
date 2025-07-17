use anyhow::Result;
use dashmap::DashMap;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    future::Future,
    process::Stdio,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
    time::sleep,
};
use tracing::info;
use uuid::Uuid;

type McpError = rmcp::model::ErrorData;

#[derive(Debug, Clone)]
struct ExecutionBuffer {
    execution_id: String,
    lines: Arc<Mutex<Vec<String>>>,
    status: Arc<Mutex<String>>,
    max_lines: usize,
    creation_time: u64,
    completion_time: Arc<Mutex<Option<u64>>>,
    error_message: Arc<Mutex<Option<String>>>,
}

impl ExecutionBuffer {
    fn new(execution_id: String, max_lines: usize) -> Self {
        Self {
            execution_id,
            lines: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(Mutex::new("running".to_string())),
            max_lines,
            creation_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completion_time: Arc::new(Mutex::new(None)),
            error_message: Arc::new(Mutex::new(None)),
        }
    }

    async fn add_line(&self, line: String) {
        let mut lines = self.lines.lock().await;
        if lines.len() < self.max_lines {
            lines.push(line);
        } else if lines.len() == self.max_lines {
            lines.push(format!("[Output truncated at {} lines]", self.max_lines));
        }
    }

    async fn mark_completed(&self) {
        *self.status.lock().await = "completed".to_string();
        *self.completion_time.lock().await = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    async fn mark_failed(&self, error: String) {
        *self.status.lock().await = "failed".to_string();
        *self.completion_time.lock().await = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        *self.error_message.lock().await = Some(error);
    }
}

#[derive(Clone)]
struct BpftraceServer {
    tool_router: ToolRouter<Self>,
    sudo_password: Arc<String>,
    execution_buffers: Arc<DashMap<String, ExecutionBuffer>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListProbesRequest {
    #[schemars(description = "Optional filter pattern (e.g., 'syscalls:*open*')")]
    filter: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListProbesResponse {
    probes: Vec<String>,
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}


#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExecProgramRequest {
    #[schemars(description = "The bpftrace program to execute")]
    program: String,
    #[schemars(description = "Execution timeout in seconds (default: 10, max: 60)")]
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_timeout() -> u64 {
    10
}

#[derive(Debug, Serialize)]
struct ExecProgramResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_id: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetResultRequest {
    #[schemars(description = "The execution ID returned by exec_program")]
    execution_id: String,
    #[schemars(description = "Start reading from this line number (default: 0)")]
    #[serde(default)]
    offset: usize,
    #[schemars(description = "Maximum lines to return (default: 1000)")]
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    1000
}

#[derive(Debug, Serialize)]
struct GetResultResponse {
    execution_id: String,
    status: String,
    lines_total: usize,
    lines_returned: usize,
    output: Vec<String>,
    has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<u64>,
}

impl BpftraceServer {
    async fn run_bpftrace_program(
        _execution_id: String,
        program: String,
        timeout: Duration,
        sudo_password: String,
        buffer: ExecutionBuffer,
    ) {
        let mut cmd = Command::new("sudo");
        cmd.arg("-S")
            .arg("bpftrace")
            .arg("-e")
            .arg(&program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                buffer
                    .mark_failed(format!("Failed to spawn process: {}", e))
                    .await;
                return;
            }
        };

        // Send password to sudo
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(format!("{}\n", sudo_password).as_bytes()).await;
            let _ = stdin.flush().await;
        }

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let start_time = tokio::time::Instant::now();

        loop {
            tokio::select! {
                _ = sleep(Duration::from_millis(100)) => {
                    if start_time.elapsed() > timeout {
                        let _ = child.kill().await;
                        buffer.add_line("[Execution timed out]".to_string()).await;
                        buffer.mark_failed("Timeout".to_string()).await;
                        break;
                    }
                }
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            buffer.add_line(line).await;
                        }
                        Ok(None) => break,
                        Err(e) => {
                            buffer.mark_failed(format!("Read error: {}", e)).await;
                            break;
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            if !line.starts_with("[sudo] password") {
                                buffer.add_line(format!("[Error] {}", line)).await;
                            }
                        }
                        Ok(None) => {}
                        Err(_) => {}
                    }
                }
            }
        }

        let _ = child.wait().await;

        let status = buffer.status.lock().await.clone();
        if status == "running" {
            buffer.mark_completed().await;
        }
    }
}

#[tool_router]
impl BpftraceServer {
    fn new(sudo_password: String) -> Self {
        let server = Self {
            tool_router: Self::tool_router(),
            sudo_password: Arc::new(sudo_password),
            execution_buffers: Arc::new(DashMap::new()),
        };

        // Start cleanup task
        let buffers = server.execution_buffers.clone();
        tokio::spawn(async move {
            let cleanup_interval = Duration::from_secs(300); // 5 minutes
            let max_age = 3600; // 1 hour

            loop {
                sleep(cleanup_interval).await;
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let mut to_remove = Vec::new();
                for entry in buffers.iter() {
                    if current_time - entry.value().creation_time > max_age {
                        to_remove.push(entry.key().clone());
                    }
                }

                for key in to_remove {
                    buffers.remove(&key);
                }
            }
        });

        server
    }

    #[tool(description = "List available bpftrace probes with optional filtering")]
    async fn list_probes(
        &self,
        Parameters(ListProbesRequest { filter }): Parameters<ListProbesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("sudo");
        cmd.arg("-S").arg("bpftrace").arg("-l");
        
        if let Some(filter) = filter {
            cmd.arg(filter);
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return Err(McpError::internal_error(
                    "Failed to spawn bpftrace process",
                    Some(json!({"error": e.to_string()})),
                ));
            }
        };

        // Send password to sudo
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin
                .write_all(format!("{}\n", self.sudo_password).as_bytes())
                .await;
            let _ = stdin.flush().await;
        }

        let output = match child.wait_with_output().await {
            Ok(output) => output,
            Err(e) => {
                return Err(McpError::internal_error(
                    "Failed to execute bpftrace",
                    Some(json!({"error": e.to_string()})),
                ));
            }
        };

        if !output.status.success() {
            return Err(McpError::internal_error(
                "Bpftrace command failed",
                Some(json!({"stderr": String::from_utf8_lossy(&output.stderr).to_string()})),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let probes: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("[sudo]"))
            .map(|s| s.to_string())
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            json!({
                "probes": probes,
                "count": probes.len()
            }).to_string()
        )]))
    }

    #[tool(description = "Get bpftrace system information and capabilities")]
    async fn bpf_info(&self) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("sudo");
        cmd.arg("-S")
            .arg("bpftrace")
            .arg("--info")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return Err(McpError::internal_error(
                    "Failed to spawn bpftrace process",
                    Some(json!({"error": e.to_string()})),
                ));
            }
        };

        // Send password to sudo
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin
                .write_all(format!("{}\n", self.sudo_password).as_bytes())
                .await;
            let _ = stdin.flush().await;
        }

        let output = match child.wait_with_output().await {
            Ok(output) => output,
            Err(e) => {
                return Err(McpError::internal_error(
                    "Failed to execute bpftrace",
                    Some(json!({"error": e.to_string()})),
                ));
            }
        };

        if !output.status.success() {
            return Err(McpError::internal_error(
                "Bpftrace command failed",
                Some(json!({"stderr": String::from_utf8_lossy(&output.stderr).to_string()})),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        Ok(CallToolResult::success(vec![Content::text(stdout.to_string())]))
    }

    #[tool(description = "Execute a bpftrace program with buffered output")]
    async fn exec_program(
        &self,
        Parameters(ExecProgramRequest { program, timeout }): Parameters<ExecProgramRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate timeout
        let timeout = timeout.clamp(1, 60);

        // Generate execution ID
        let execution_id = format!("exec_{}", Uuid::new_v4().to_string()[..8].to_string());

        // Create buffer
        let buffer = ExecutionBuffer::new(execution_id.clone(), 10000);
        self.execution_buffers
            .insert(execution_id.clone(), buffer.clone());

        // Start execution in background
        let password = self.sudo_password.to_string();
        let exec_id = execution_id.clone();
        tokio::spawn(async move {
            BpftraceServer::run_bpftrace_program(
                exec_id,
                program,
                Duration::from_secs(timeout),
                password,
                buffer,
            )
            .await;
        });

        // Give it a moment to check for syntax errors
        sleep(Duration::from_millis(500)).await;

        // Check if it failed immediately (syntax error)
        if let Some(buffer) = self.execution_buffers.get(&execution_id) {
            let status = buffer.status.lock().await.clone();
            if status == "failed" {
                let error_msg = buffer
                    .error_message
                    .lock()
                    .await
                    .clone()
                    .unwrap_or_else(|| "Failed to start program".to_string());
                return Err(McpError::internal_error(
                    "Failed to start bpftrace program", 
                    Some(json!({"error": error_msg})),
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            json!({
                "execution_id": execution_id,
                "status": "started",
                "message": format!("Program execution started with timeout of {}s", timeout)
            }).to_string()
        )]))
    }

    #[tool(description = "Get buffered output from a bpftrace execution")]
    async fn get_result(
        &self,
        Parameters(GetResultRequest {
            execution_id,
            offset,
            limit,
        }): Parameters<GetResultRequest>,
    ) -> Result<CallToolResult, McpError> {
        if let Some(buffer) = self.execution_buffers.get(&execution_id) {
            let lines = buffer.lines.lock().await;
            let total_lines = lines.len();
            let end_index = (offset + limit).min(total_lines);
            let output_lines: Vec<String> = lines[offset..end_index].to_vec();

            let status = buffer.status.lock().await.clone();
            let error_message = buffer.error_message.lock().await.clone();
            
            let duration = if let Some(completion_time) = *buffer.completion_time.lock().await {
                Some(completion_time - buffer.creation_time)
            } else {
                None
            };

            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "execution_id": execution_id,
                    "status": status,
                    "lines_total": total_lines,
                    "lines_returned": output_lines.len(),
                    "output": output_lines,
                    "has_more": end_index < total_lines,
                    "error_message": error_message,
                    "duration": duration
                }).to_string()
            )]))
        } else {
            Err(McpError::invalid_params(
                "Execution ID not found",
                None,
            ))
        }
    }
}

#[tool_handler]
impl ServerHandler for BpftraceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("MCP server for bpftrace - provides Linux kernel tracing capabilities".to_string()),
        }
    }
}

fn verify_password(password: &str) -> Result<()> {
    // Test the password with a simple sudo command
    let output = std::process::Command::new("sudo")
        .arg("-S")
        .arg("true")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(format!("{}\n", password).as_bytes())?;
                stdin.flush()?;
            }
            child.wait_with_output()
        })?;
    
    if !output.status.success() {
        // Don't use eprintln since it interferes with stdio MCP communication
        std::process::exit(1);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("bpftrace_mcp_server=info".parse()?)
                .add_directive("rmcp=info".parse()?),
        )
        .init();

    // Get password from environment variable
    let sudo_password = match std::env::var("BPFTRACE_PASSWD") {
        Ok(password) => {
            verify_password(&password)?;
            password
        },
        Err(_) => {
            // Exit without printing to stdio/stderr to avoid interfering with MCP
            std::process::exit(1);
        }
    };
    
    let server = BpftraceServer::new(sudo_password);
    
    info!("Starting bpftrace MCP server on stdio");
    
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    
    Ok(())
}