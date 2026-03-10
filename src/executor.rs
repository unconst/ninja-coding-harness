use crate::config::DockerConfig;
use crate::error::{NinjaError, Result};
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::Docker;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Docker-based code execution sandbox
pub struct CodeExecutor {
    docker: Docker,
    config: DockerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub working_directory: String,
    pub command: String,
    pub environment: HashMap<String, String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i64,
    pub success: bool,
    pub execution_time_ms: u64,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub path: String,
    pub content: String,
    pub operation_type: FileOperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperationType {
    Read,
    Write,
    Create,
    Delete,
}

impl CodeExecutor {
    pub async fn new(config: DockerConfig) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;

        // Test connection
        let version = docker.version().await?;
        info!("Connected to Docker Engine version: {}", version.version.unwrap_or_default());

        Ok(Self { docker, config })
    }

    /// Create a new execution container with the workspace mounted
    pub async fn create_container(&self, workspace_path: &str) -> Result<String> {
        let container_name = format!("ninja-{}", Uuid::new_v4());

        let host_config = bollard::container::HostConfig {
            memory: Some(self.parse_memory_limit(&self.config.memory_limit)?),
            cpu_period: Some(100000),  // 100ms period
            cpu_quota: Some((self.parse_cpu_limit(&self.config.cpu_limit)? * 100000.0) as i64),
            binds: Some(vec![format!("{}:/workspace", workspace_path)]),
            network_mode: Some("none".to_string()), // No network access for security
            pids_limit: Some(256), // Limit number of processes
            ..Default::default()
        };

        let config = Config {
            image: Some(self.config.base_image.clone()),
            working_dir: Some("/workspace".to_string()),
            cmd: Some(vec!["/bin/bash".to_string()]),
            tty: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        info!("Creating container: {}", container_name);
        self.docker
            .create_container(Some(options), config)
            .await?;

        // Start the container
        self.docker
            .start_container(&container_name, None::<StartContainerOptions<String>>)
            .await?;

        debug!("Container {} created and started", container_name);
        Ok(container_name)
    }

    /// Execute a command in the specified container
    pub async fn execute_in_container(
        &self,
        container_id: &str,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();

        debug!("Executing command in container {}: {}", container_id, request.command);

        let exec_options = CreateExecOptions {
            cmd: Some(vec!["/bin/bash", "-c", &request.command]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            working_dir: Some(request.working_directory),
            env: if request.environment.is_empty() {
                None
            } else {
                Some(
                    request
                        .environment
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect(),
                )
            },
            ..Default::default()
        };

        let exec = self.docker.create_exec(container_id, exec_options).await?;

        let start_options = StartExecOptions {
            detach: false,
            tty: false,
            ..Default::default()
        };

        let mut stream = self.docker.start_exec(&exec.id, Some(start_options));

        let mut stdout = String::new();
        let mut stderr = String::new();
        let timeout_duration = std::time::Duration::from_secs(
            request.timeout_seconds.unwrap_or(self.config.timeout_seconds)
        );

        let mut timed_out = false;

        // Collect output with timeout
        tokio::select! {
            _ = async {
                while let Some(msg) = stream.next().await {
                    match msg {
                        Ok(output) => {
                            match output {
                                bollard::container::LogOutput::StdOut { message } => {
                                    let text = String::from_utf8_lossy(&message);
                                    stdout.push_str(&text);
                                }
                                bollard::container::LogOutput::StdErr { message } => {
                                    let text = String::from_utf8_lossy(&message);
                                    stderr.push_str(&text);
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            warn!("Error reading container output: {}", e);
                            break;
                        }
                    }
                }
            } => {},
            _ = tokio::time::sleep(timeout_duration) => {
                timed_out = true;
                warn!("Command timed out after {}s", timeout_duration.as_secs());
            }
        }

        // Get exit code
        let exec_inspect = self.docker.inspect_exec(&exec.id).await?;
        let exit_code = exec_inspect.exit_code.unwrap_or(-1);

        let execution_time = start_time.elapsed();
        let success = exit_code == 0 && !timed_out;

        Ok(ExecutionResult {
            stdout,
            stderr,
            exit_code,
            success,
            execution_time_ms: execution_time.as_millis() as u64,
            timed_out,
        })
    }

    /// Read a file from the container
    pub async fn read_file(&self, container_id: &str, file_path: &str) -> Result<String> {
        let request = ExecutionRequest {
            working_directory: "/workspace".to_string(),
            command: format!("cat '{}'", file_path),
            environment: HashMap::new(),
            timeout_seconds: Some(30),
        };

        let result = self.execute_in_container(container_id, request).await?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(NinjaError::Execution(format!(
                "Failed to read file {}: {}",
                file_path, result.stderr
            )))
        }
    }

    /// Write a file to the container
    pub async fn write_file(
        &self,
        container_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<()> {
        // Escape content for shell
        let escaped_content = content.replace('\'', "'\"'\"'");

        let request = ExecutionRequest {
            working_directory: "/workspace".to_string(),
            command: format!("mkdir -p $(dirname '{}') && printf '%s' '{}' > '{}'",
                           file_path, escaped_content, file_path),
            environment: HashMap::new(),
            timeout_seconds: Some(30),
        };

        let result = self.execute_in_container(container_id, request).await?;

        if result.success {
            Ok(())
        } else {
            Err(NinjaError::Execution(format!(
                "Failed to write file {}: {}",
                file_path, result.stderr
            )))
        }
    }

    /// List files in a directory
    pub async fn list_files(
        &self,
        container_id: &str,
        directory: &str,
        recursive: bool,
    ) -> Result<Vec<String>> {
        let command = if recursive {
            format!("find '{}' -type f 2>/dev/null || echo", directory)
        } else {
            format!("ls -1 '{}' 2>/dev/null || echo", directory)
        };

        let request = ExecutionRequest {
            working_directory: "/workspace".to_string(),
            command,
            environment: HashMap::new(),
            timeout_seconds: Some(30),
        };

        let result = self.execute_in_container(container_id, request).await?;

        if result.success {
            let files: Vec<String> = result
                .stdout
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| line.to_string())
                .collect();
            Ok(files)
        } else {
            // If ls fails, return empty list (directory might not exist)
            Ok(Vec::new())
        }
    }

    /// Remove a container
    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        debug!("Removing container: {}", container_id);

        // Stop container first
        if let Err(e) = self.docker.stop_container(container_id, None).await {
            warn!("Failed to stop container {}: {}", container_id, e);
        }

        // Remove container
        self.docker
            .remove_container(
                container_id,
                Some(bollard::container::RemoveContainerOptions {
                    force: true,
                    v: true,
                    ..Default::default()
                }),
            )
            .await?;

        debug!("Container {} removed", container_id);
        Ok(())
    }

    /// Create a temporary workspace directory
    pub async fn create_workspace(&self) -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;
        info!("Created temporary workspace: {}", temp_dir.path().display());
        Ok(temp_dir)
    }

    /// Setup workspace with initial files
    pub async fn setup_workspace(
        &self,
        workspace_path: &Path,
        setup_files: &[FileOperation],
    ) -> Result<()> {
        for file_op in setup_files {
            let full_path = workspace_path.join(&file_op.path);

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            match file_op.operation_type {
                FileOperationType::Write | FileOperationType::Create => {
                    tokio::fs::write(&full_path, &file_op.content).await?;
                }
                FileOperationType::Delete => {
                    if full_path.exists() {
                        tokio::fs::remove_file(&full_path).await?;
                    }
                }
                FileOperationType::Read => {
                    // Read operation doesn't modify workspace
                }
            }
        }

        Ok(())
    }

    fn parse_memory_limit(&self, limit: &str) -> Result<i64> {
        // Parse memory limits like "1g", "512m", etc.
        let limit = limit.to_lowercase();
        if let Some(stripped) = limit.strip_suffix('g') {
            Ok(stripped.parse::<i64>().unwrap_or(1) * 1024 * 1024 * 1024)
        } else if let Some(stripped) = limit.strip_suffix('m') {
            Ok(stripped.parse::<i64>().unwrap_or(512) * 1024 * 1024)
        } else if let Some(stripped) = limit.strip_suffix('k') {
            Ok(stripped.parse::<i64>().unwrap_or(512) * 1024)
        } else {
            Ok(limit.parse::<i64>().unwrap_or(1024 * 1024 * 1024))
        }
    }

    fn parse_cpu_limit(&self, limit: &str) -> Result<f64> {
        // Parse CPU limits like "1.0", "0.5", etc.
        Ok(limit.parse::<f64>().unwrap_or(1.0))
    }
}