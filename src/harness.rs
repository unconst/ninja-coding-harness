use crate::challenge::{Challenge, ChallengeResult, SolutionStatus, SolutionMetrics, TestExecution, TestResult, TestType};
use crate::config::Config;
use crate::error::{NinjaError, Result};
use crate::executor::{CodeExecutor, ExecutionRequest, FileOperation, FileOperationType};
use crate::llm::{ChatMessage, ChatRequest, LlmProvider, OpenRouterProvider, create_coding_tools, FunctionCallRequest};
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tracing::{debug, info, warn, error};

/// Main Ninja coding harness for solving programming challenges
pub struct CodingHarness {
    config: Config,
    llm_provider: Arc<dyn LlmProvider>,
    executor: CodeExecutor,
}

impl CodingHarness {
    /// Create a new coding harness instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Ninja coding harness");

        let llm_provider = Arc::new(OpenRouterProvider::new(config.openrouter.clone()));
        let executor = CodeExecutor::new(config.docker.clone()).await?;

        Ok(Self {
            config,
            llm_provider,
            executor,
        })
    }

    /// Validate that the harness is properly set up
    pub async fn validate(&self) -> Result<()> {
        info!("Validating harness setup");

        // Test LLM connection
        let test_request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello, are you working?".to_string(),
            }],
            temperature: Some(0.1),
            max_tokens: Some(50),
            tools: None,
            tool_choice: None,
        };

        let response = self.llm_provider.chat_completion(test_request).await?;
        debug!("LLM test response: {}", response.content);

        // Test Docker connection by creating a simple container
        if self.config.docker.enabled {
            let workspace = self.executor.create_workspace().await?;
            let container_id = self.executor.create_container(workspace.path().to_str().unwrap()).await?;

            let test_exec = ExecutionRequest {
                working_directory: "/workspace".to_string(),
                command: "echo 'Hello from container'".to_string(),
                environment: HashMap::new(),
                timeout_seconds: Some(10),
            };

            let result = self.executor.execute_in_container(&container_id, test_exec).await?;
            if !result.success {
                return Err(NinjaError::Validation(
                    format!("Docker test execution failed: {}", result.stderr)
                ));
            }

            self.executor.remove_container(&container_id).await?;
            debug!("Docker test execution successful");
        }

        info!("✅ Harness validation completed successfully");
        Ok(())
    }

    /// Solve a single coding challenge
    pub async fn solve_challenge(&self, challenge_path: &str, output_dir: &str) -> Result<ChallengeResult> {
        info!("Starting to solve challenge: {}", challenge_path);

        // Load the challenge
        let challenge = Challenge::from_file(challenge_path).await?;
        challenge.validate()?;

        let start_time = std::time::Instant::now();
        let mut result = ChallengeResult::new(challenge.id.clone());

        // Create workspace and container
        let workspace = self.executor.create_workspace().await?;
        let workspace_path = workspace.path().to_str().unwrap();

        // Setup initial workspace files
        let setup_files: Vec<FileOperation> = challenge
            .expected_files
            .iter()
            .map(|file_path| FileOperation {
                path: file_path.clone(),
                content: "".to_string(), // Create empty files initially
                operation_type: FileOperationType::Create,
            })
            .collect();

        self.executor.setup_workspace(workspace.path(), &setup_files).await?;
        let container_id = self.executor.create_container(workspace_path).await?;

        // Execute setup commands
        for setup_cmd in &challenge.setup_commands {
            info!("Running setup command: {}", setup_cmd);
            let request = ExecutionRequest {
                working_directory: "/workspace".to_string(),
                command: setup_cmd.clone(),
                environment: HashMap::new(),
                timeout_seconds: Some(60),
            };

            let exec_result = self.executor.execute_in_container(&container_id, request).await?;
            if !exec_result.success {
                warn!("Setup command failed: {} - {}", setup_cmd, exec_result.stderr);
            }
        }

        // Run the agent loop to solve the challenge
        match self.run_agent_loop(&challenge, &container_id, &mut result).await {
            Ok(_) => {
                info!("Agent loop completed successfully");
            }
            Err(e) => {
                error!("Agent loop failed: {}", e);
                result.status = SolutionStatus::Error;
                result.error_message = Some(format!("Agent loop error: {}", e));
            }
        }

        // Run validation tests
        result.test_results = self.run_tests(&challenge, &container_id).await?;

        // Determine final status based on test results
        let passed_tests = result.test_results.iter()
            .filter(|t| matches!(t.status, TestResult::Pass))
            .count();
        let total_tests = result.test_results.len();

        result.status = if passed_tests == total_tests {
            SolutionStatus::Solved
        } else if passed_tests > 0 {
            SolutionStatus::Partial
        } else {
            SolutionStatus::Failed
        };

        // Collect solution files
        result.solution_files = self.collect_solution_files(&container_id).await?;

        // Update metrics
        result.metrics.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        result.completed_at = Utc::now();

        // Cleanup
        self.executor.remove_container(&container_id).await?;

        // Save results
        tokio::fs::create_dir_all(output_dir).await?;
        let result_path = format!("{}/result_{}.json", output_dir, challenge.id);
        result.save_to_file(&result_path).await?;

        info!("Challenge solution completed with status: {:?}", result.status);
        Ok(result)
    }

    /// Run the agent loop to solve the challenge
    async fn run_agent_loop(
        &self,
        challenge: &Challenge,
        container_id: &str,
        result: &mut ChallengeResult,
    ) -> Result<()> {
        let max_iterations = 20;
        let mut iteration = 0;
        let tools = create_coding_tools();

        // Initial system prompt
        let system_prompt = format!(
            "You are a programming assistant that solves coding challenges. Your task is to solve this challenge:\n\n\
            Title: {}\n\
            Description: {}\n\
            Language: {}\n\
            Difficulty: {:?}\n\n\
            Expected files to create/modify: {:?}\n\n\
            You have access to tools to read files, write files, execute commands, and list files. \
            Work systematically to understand the problem and implement a solution.\n\n\
            After implementing your solution, run any tests or validation to ensure it works correctly.",
            challenge.title,
            challenge.description,
            challenge.language,
            challenge.difficulty,
            challenge.expected_files
        );

        let mut conversation = vec![ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }];

        while iteration < max_iterations {
            debug!("Agent iteration {}/{}", iteration + 1, max_iterations);

            // Make LLM request with tools
            let request = ChatRequest {
                messages: conversation.clone(),
                temperature: Some(0.1),
                max_tokens: Some(4096),
                tools: Some(tools.clone()),
                tool_choice: Some("auto".to_string()),
            };

            let response = self.llm_provider.chat_completion(request).await?;

            // Update metrics
            result.metrics.llm_calls_made += 1;
            result.metrics.tokens_used += response.usage.total_tokens;

            // Add assistant response to conversation
            if !response.content.is_empty() {
                conversation.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                });
            }

            // Execute tool calls if present
            if let Some(tool_calls) = response.tool_calls {
                let mut tool_results = Vec::new();

                for tool_call in tool_calls {
                    result.metrics.tool_calls_made += 1;

                    let tool_result = self.execute_tool_call(&tool_call.name, &tool_call.arguments, container_id, result).await?;
                    tool_results.push(format!("Tool: {}\nResult: {}", tool_call.name, tool_result));
                }

                // Add tool results to conversation
                if !tool_results.is_empty() {
                    conversation.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!("Tool execution results:\n{}", tool_results.join("\n\n")),
                    });
                }

                // Check if we should continue
                if self.should_stop_agent_loop(&response.content, &tool_calls).await {
                    info!("Agent indicated completion");
                    break;
                }
            } else {
                // No tool calls, agent might be finished
                info!("No tool calls in response, agent may be finished");
                break;
            }

            iteration += 1;
        }

        if iteration >= max_iterations {
            warn!("Agent loop reached maximum iterations");
        }

        Ok(())
    }

    /// Execute a tool call
    async fn execute_tool_call(
        &self,
        tool_name: &str,
        arguments: &Value,
        container_id: &str,
        result: &mut SolutionMetrics,
    ) -> Result<String> {
        debug!("Executing tool: {}", tool_name);

        match tool_name {
            "read_file" => {
                let path = arguments["path"].as_str()
                    .ok_or_else(|| NinjaError::Execution("Missing path parameter".to_string()))?;

                match self.executor.read_file(container_id, path).await {
                    Ok(content) => Ok(content),
                    Err(e) => Ok(format!("Error reading file: {}", e)),
                }
            }
            "write_file" => {
                let path = arguments["path"].as_str()
                    .ok_or_else(|| NinjaError::Execution("Missing path parameter".to_string()))?;
                let content = arguments["content"].as_str()
                    .ok_or_else(|| NinjaError::Execution("Missing content parameter".to_string()))?;

                match self.executor.write_file(container_id, path, content).await {
                    Ok(_) => {
                        result.files_modified += 1;
                        Ok(format!("Successfully wrote to file: {}", path))
                    }
                    Err(e) => Ok(format!("Error writing file: {}", e)),
                }
            }
            "execute_command" => {
                let command = arguments["command"].as_str()
                    .ok_or_else(|| NinjaError::Execution("Missing command parameter".to_string()))?;
                let working_directory = arguments["working_directory"].as_str()
                    .unwrap_or("/workspace");

                let request = ExecutionRequest {
                    working_directory: working_directory.to_string(),
                    command: command.to_string(),
                    environment: HashMap::new(),
                    timeout_seconds: Some(60),
                };

                let exec_result = self.executor.execute_in_container(container_id, request).await?;

                Ok(format!(
                    "Exit code: {}\nStdout:\n{}\nStderr:\n{}",
                    exec_result.exit_code,
                    exec_result.stdout,
                    exec_result.stderr
                ))
            }
            "list_files" => {
                let path = arguments["path"].as_str()
                    .ok_or_else(|| NinjaError::Execution("Missing path parameter".to_string()))?;
                let recursive = arguments["recursive"].as_bool().unwrap_or(false);

                match self.executor.list_files(container_id, path, recursive).await {
                    Ok(files) => Ok(files.join("\n")),
                    Err(e) => Ok(format!("Error listing files: {}", e)),
                }
            }
            _ => {
                warn!("Unknown tool: {}", tool_name);
                Ok(format!("Unknown tool: {}", tool_name))
            }
        }
    }

    /// Check if we should stop the agent loop
    async fn should_stop_agent_loop(&self, content: &str, tool_calls: &[crate::llm::ToolCall]) -> bool {
        // Stop if agent explicitly says they're done
        let done_keywords = ["done", "complete", "finished", "solution is ready"];
        let content_lower = content.to_lowercase();

        if done_keywords.iter().any(|keyword| content_lower.contains(keyword)) {
            return true;
        }

        // Stop if no more tool calls and the response seems conclusive
        if tool_calls.is_empty() && content.len() > 50 {
            return true;
        }

        false
    }

    /// Run tests for the challenge
    async fn run_tests(&self, challenge: &Challenge, container_id: &str) -> Result<Vec<TestExecution>> {
        let mut test_results = Vec::new();

        for test_case in &challenge.tests {
            info!("Running test: {}", test_case.name);

            let start_time = std::time::Instant::now();

            let request = ExecutionRequest {
                working_directory: "/workspace".to_string(),
                command: test_case.command.clone(),
                environment: HashMap::new(),
                timeout_seconds: Some(60),
            };

            let result = self.executor.execute_in_container(container_id, request).await?;
            let execution_time = start_time.elapsed();

            // Determine test status based on test type and result
            let test_status = match &test_case.test_type {
                TestType::FailToPass => {
                    if result.success {
                        TestResult::Pass
                    } else {
                        TestResult::Fail
                    }
                }
                TestType::PassToPass => {
                    if result.success {
                        TestResult::Pass
                    } else {
                        TestResult::Fail
                    }
                }
                TestType::Validation => {
                    if result.success {
                        TestResult::Pass
                    } else {
                        TestResult::Fail
                    }
                }
            };

            test_results.push(TestExecution {
                test_name: test_case.name.clone(),
                status: test_status,
                output: format!("stdout:\n{}\nstderr:\n{}", result.stdout, result.stderr),
                execution_time_ms: execution_time.as_millis() as u64,
            });
        }

        // Also run validation commands
        for (i, validation_cmd) in challenge.validation_commands.iter().enumerate() {
            info!("Running validation command {}: {}", i + 1, validation_cmd);

            let start_time = std::time::Instant::now();

            let request = ExecutionRequest {
                working_directory: "/workspace".to_string(),
                command: validation_cmd.clone(),
                environment: HashMap::new(),
                timeout_seconds: Some(60),
            };

            let result = self.executor.execute_in_container(container_id, request).await?;
            let execution_time = start_time.elapsed();

            test_results.push(TestExecution {
                test_name: format!("validation_{}", i + 1),
                status: if result.success { TestResult::Pass } else { TestResult::Fail },
                output: format!("stdout:\n{}\nstderr:\n{}", result.stdout, result.stderr),
                execution_time_ms: execution_time.as_millis() as u64,
            });
        }

        Ok(test_results)
    }

    /// Collect solution files from the container
    async fn collect_solution_files(&self, container_id: &str) -> Result<Vec<String>> {
        let files = self.executor.list_files(container_id, "/workspace", true).await?;

        // Filter out common non-solution files
        let solution_files: Vec<String> = files
            .into_iter()
            .filter(|f| {
                !f.contains("__pycache__")
                && !f.ends_with(".pyc")
                && !f.contains(".git/")
                && !f.contains("node_modules/")
            })
            .collect();

        Ok(solution_files)
    }

    /// Start the continuous improvement loop
    pub async fn run_improvement_loop(&self) -> Result<()> {
        info!("Starting continuous improvement loop - NOT YET IMPLEMENTED");

        // TODO: Implement the full improvement loop with:
        // 1. Challenge generation from SWE-Forge
        // 2. Running challenges against the harness
        // 3. Analyzing performance and identifying improvements
        // 4. Auto-optimization and deployment

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        Ok(())
    }
}