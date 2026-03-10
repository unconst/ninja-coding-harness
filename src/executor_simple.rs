use crate::config::DockerConfig;
use crate::error::{NinjaError, Result};
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::models::{HostConfig};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub command: String,
    pub working_directory: String,
    pub files: Vec<FileOperation>,
    pub environment: HashMap<String, String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub path: String,
    pub content: String,
    pub operation_type: FileOperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperationType {
    Create,
    Write,
    Append,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub container_id: Option<String>,
}

pub struct SimpleCodeExecutor {
    docker: Docker,
    config: DockerConfig,
}

impl SimpleCodeExecutor {
    pub async fn new(config: DockerConfig) -> Result<Self> {
        info!("Initializing simple Docker executor");

        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| NinjaError::Docker(e))?;

        // Test Docker connectivity
        let _info = docker.info().await.map_err(|e| NinjaError::Docker(e))?;
        info!("✅ Docker connection established");

        Ok(Self { docker, config })
    }

    pub async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let container_name = format!("ninja-{}", Uuid::new_v4());

        info!("Creating container: {}", container_name);
        debug!("Execution request: {:?}", request);

        // Create container config with sleep command to keep it running
        let container_config = Config {
            image: Some(self.config.base_image.clone()),
            working_dir: Some(request.working_directory.clone()),
            cmd: Some(vec!["sleep".to_string(), "300".to_string()]), // Keep container alive for 5 minutes
            env: Some(
                request
                    .environment
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<String>>(),
            ),
            host_config: Some(HostConfig {
                memory: Some(1073741824), // 1GB in bytes
                nano_cpus: Some(1000000000), // 1 CPU
                auto_remove: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        // Create the container
        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: container_name.as_str(),
                    platform: None,
                }),
                container_config,
            )
            .await
            .map_err(|e| NinjaError::Docker(e))?;

        let container_id = container.id;
        info!("Created container: {}", container_id);

        // Start the container
        self.docker
            .start_container::<String>(&container_id, None)
            .await
            .map_err(|e| NinjaError::Docker(e))?;

        info!("Started container: {}", container_id);

        // Write files to container
        for file_op in &request.files {
            self.write_file_to_container(&container_id, file_op).await?;
        }

        // Execute the command
        let execution_result = self
            .execute_command_in_container(&container_id, &request.command, request.timeout_seconds)
            .await?;

        // Stop and remove container (auto_remove should handle cleanup)
        let _ = self.docker.stop_container(&container_id, None).await;

        let duration = start_time.elapsed();

        Ok(ExecutionResult {
            exit_code: execution_result.0,
            stdout: execution_result.1,
            stderr: execution_result.2,
            duration_ms: duration.as_millis() as u64,
            container_id: Some(container_id),
        })
    }

    async fn write_file_to_container(&self, container_id: &str, file_op: &FileOperation) -> Result<()> {
        debug!("Writing file to container: {}", file_op.path);

        // For simplicity, we'll use exec to create files
        let create_dir_cmd = format!("mkdir -p $(dirname '{}')", file_op.path);
        let _ = self.exec_simple_command(container_id, &create_dir_cmd).await;

        let write_file_cmd = format!("cat > '{}' << 'EOF'\n{}\nEOF", file_op.path, file_op.content);
        let (exit_code, _stdout, stderr) = self.exec_simple_command(container_id, &write_file_cmd).await?;

        if exit_code != 0 {
            warn!("Failed to write file {}: {}", file_op.path, stderr);
            return Err(NinjaError::Execution(format!("Failed to write file {}: {}", file_op.path, stderr)));
        }

        debug!("Successfully wrote file: {}", file_op.path);
        Ok(())
    }

    async fn execute_command_in_container(
        &self,
        container_id: &str,
        command: &str,
        timeout_seconds: u64,
    ) -> Result<(i32, String, String)> {
        info!("Executing command in container: {}", command);

        let execution_future = self.exec_simple_command(container_id, command);

        match timeout(Duration::from_secs(timeout_seconds), execution_future).await {
            Ok(result) => result,
            Err(_) => {
                error!("Command execution timed out after {} seconds", timeout_seconds);
                Err(NinjaError::Execution(format!("Command timed out after {} seconds", timeout_seconds)))
            }
        }
    }

    async fn exec_simple_command(&self, container_id: &str, command: &str) -> Result<(i32, String, String)> {
        let exec_config = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", command]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec_result = self
            .docker
            .create_exec(container_id, exec_config)
            .await
            .map_err(|e| NinjaError::Docker(e))?;

        let start_result = self
            .docker
            .start_exec(&exec_result.id, None)
            .await
            .map_err(|e| NinjaError::Docker(e))?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        match start_result {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(msg) = output.next().await {
                    match msg.map_err(|e| NinjaError::Docker(e))? {
                        bollard::container::LogOutput::StdOut { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    }
                }
            }
            _ => return Err(NinjaError::Execution("Failed to attach to exec".to_string())),
        }

        // Get the exit code
        let inspect_result = self
            .docker
            .inspect_exec(&exec_result.id)
            .await
            .map_err(|e| NinjaError::Docker(e))?;

        let exit_code = inspect_result.exit_code.unwrap_or(-1) as i32;

        debug!("Command exit code: {}", exit_code);
        debug!("Command stdout: {}", stdout.trim());
        debug!("Command stderr: {}", stderr.trim());

        Ok((exit_code, stdout, stderr))
    }
}