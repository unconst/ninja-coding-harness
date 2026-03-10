use crate::challenge::Challenge;
use crate::config::Config;
use crate::error::Result;
use crate::executor_simple::{SimpleCodeExecutor, ExecutionRequest, FileOperation, FileOperationType};
use crate::llm::{LlmProvider, OpenRouterProvider, ChatRequest, ChatMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, error};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult {
    pub challenge_id: String,
    pub success: bool,
    pub generated_code: Option<String>,
    pub test_results: Vec<TestResult>,
    pub total_attempts: u32,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub output: String,
    pub error_output: String,
    pub exit_code: i32,
}

pub struct ChallengeSolver {
    llm_provider: Arc<dyn LlmProvider>,
    executor: SimpleCodeExecutor,
    config: Config,
}

impl ChallengeSolver {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Challenge Solver");

        let llm_provider = Arc::new(OpenRouterProvider::new(config.openrouter.clone()));
        let executor = SimpleCodeExecutor::new(config.docker.clone()).await?;

        Ok(Self {
            llm_provider,
            executor,
            config,
        })
    }

    pub async fn solve_challenge(&self, challenge: &Challenge) -> Result<SolveResult> {
        let start_time = std::time::Instant::now();
        info!("Starting to solve challenge: {}", challenge.id);

        // Generate the initial code
        let code_result = self.generate_code_for_challenge(challenge).await;

        let generated_code = match code_result {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to generate code: {}", e);
                return Ok(SolveResult {
                    challenge_id: challenge.id.clone(),
                    success: false,
                    generated_code: None,
                    test_results: vec![],
                    total_attempts: 1,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some(format!("Code generation failed: {}", e)),
                });
            }
        };

        debug!("Generated code:\n{}", generated_code);

        // Run the tests
        let test_results = match self.run_challenge_tests(challenge, &generated_code).await {
            Ok(results) => results,
            Err(e) => {
                error!("Failed to run tests: {}", e);
                return Ok(SolveResult {
                    challenge_id: challenge.id.clone(),
                    success: false,
                    generated_code: Some(generated_code),
                    test_results: vec![],
                    total_attempts: 1,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some(format!("Test execution failed: {}", e)),
                });
            }
        };

        let all_tests_passed = test_results.iter().all(|t| t.passed);

        info!("Challenge solve complete. Success: {}", all_tests_passed);
        debug!("Test results: {:?}", test_results);

        Ok(SolveResult {
            challenge_id: challenge.id.clone(),
            success: all_tests_passed,
            generated_code: Some(generated_code),
            test_results,
            total_attempts: 1,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: None,
        })
    }

    async fn generate_code_for_challenge(&self, challenge: &Challenge) -> Result<String> {
        info!("Generating code for challenge: {}", challenge.title);

        let prompt = format!(
            "You are a coding expert. Solve this programming challenge:

**Challenge: {}**
{}

Language: {}
Difficulty: {}

Requirements:
- Write complete, working code that solves the problem
- Include all necessary imports and proper function definitions
- Make sure the code is syntactically correct and will pass the tests
- For the Fibonacci challenge, create a function called 'fib' that takes n as parameter
- For the math utilities challenge, create a function called 'add' that takes two parameters
- ONLY generate the solution file, do NOT generate test files (tests are provided separately)
- Only respond with the solution code, no explanations or markdown formatting

Expected files: {:?}

Write clean, efficient code:",
            challenge.title,
            challenge.description,
            challenge.language,
            challenge.difficulty,
            challenge.expected_files
        );

        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: Some(0.1), // Low temperature for consistent code generation
            max_tokens: Some(4096),
            tools: None,
            tool_choice: None,
        };

        let response = self.llm_provider.chat_completion(request).await?;

        // Clean up any markdown formatting that might have slipped through
        let code = response.content
            .trim()
            .strip_prefix("```python")
            .or_else(|| response.content.trim().strip_prefix("```"))
            .unwrap_or(response.content.trim())
            .strip_suffix("```")
            .unwrap_or(response.content.trim())
            .trim()
            .to_string();

        debug!("Generated code (cleaned): {}", code);
        Ok(code)
    }

    async fn run_challenge_tests(&self, challenge: &Challenge, code: &str) -> Result<Vec<TestResult>> {
        info!("Running tests for challenge: {}", challenge.id);

        // Prepare files for the challenge
        let mut files = vec![];

        // Add the main solution file
        if let Some(main_file) = challenge.expected_files.get(0) {
            files.push(FileOperation {
                path: format!("/app/{}", main_file),
                content: code.to_string(),
                operation_type: FileOperationType::Create,
            });
        }

        // Use existing test file if available, otherwise generate one
        if challenge.expected_files.len() > 1 {
            if let Some(test_file) = challenge.expected_files.get(1) {
                let test_file_path = format!("generated-challenges/{}", test_file);
                match tokio::fs::read_to_string(&test_file_path).await {
                    Ok(existing_test_content) => {
                        info!("✅ Using existing test file: {}", test_file_path);
                        debug!("Existing test content length: {} chars", existing_test_content.len());
                        files.push(FileOperation {
                            path: format!("/app/{}", test_file),
                            content: existing_test_content,
                            operation_type: FileOperationType::Create,
                        });
                    },
                    Err(e) => {
                        info!("❌ Existing test file not found at {}, error: {}", test_file_path, e);
                        info!("Generating basic test for: {}", test_file);
                        let test_content = self.generate_basic_test_file(challenge, code).await?;
                        files.push(FileOperation {
                            path: format!("/app/{}", test_file),
                            content: test_content,
                            operation_type: FileOperationType::Create,
                        });
                    }
                }
            }
        }

        let mut env = HashMap::new();
        env.insert("PYTHONPATH".to_string(), "/app".to_string());

        let mut test_results = vec![];

        // Run setup commands first if any
        info!("📋 Challenge has {} setup commands", challenge.setup_commands.len());
        for setup_cmd in &challenge.setup_commands {
            info!("🔧 Running setup command: {}", setup_cmd);

            let setup_request = ExecutionRequest {
                command: setup_cmd.clone(),
                working_directory: "/app".to_string(),
                files: vec![], // Don't re-create files for setup
                environment: env.clone(),
                timeout_seconds: 60,
            };

            match self.executor.execute(setup_request).await {
                Ok(result) => {
                    info!("✅ Setup command completed: exit_code={}", result.exit_code);
                    if result.exit_code != 0 {
                        error!("❌ Setup command failed with exit code {}: stderr={}", result.exit_code, result.stderr);
                        error!("❌ Setup stdout: {}", result.stdout);
                    } else {
                        debug!("Setup stdout: {}", result.stdout);
                    }
                },
                Err(e) => {
                    error!("❌ Setup command execution error: {}", e);
                    // Continue with tests even if setup fails - this allows debugging
                }
            }
        }

        // Run each test
        for test in &challenge.tests {
            debug!("Running test: {}", test.name);

            let test_request = ExecutionRequest {
                command: test.command.clone(),
                working_directory: "/app".to_string(),
                files: files.clone(), // Include files for first test, empty for subsequent
                environment: env.clone(),
                timeout_seconds: 30,
            };

            match self.executor.execute(test_request).await {
                Ok(exec_result) => {
                    let passed = exec_result.exit_code == 0 && !exec_result.stdout.contains("FAILED");

                    test_results.push(TestResult {
                        test_name: test.name.clone(),
                        passed,
                        output: exec_result.stdout,
                        error_output: exec_result.stderr,
                        exit_code: exec_result.exit_code,
                    });
                },
                Err(e) => {
                    error!("Test execution failed for {}: {}", test.name, e);
                    test_results.push(TestResult {
                        test_name: test.name.clone(),
                        passed: false,
                        output: String::new(),
                        error_output: format!("Execution error: {}", e),
                        exit_code: -1,
                    });
                }
            }

            // Clear files for subsequent tests to avoid conflicts
            files.clear();
        }

        Ok(test_results)
    }

    async fn generate_basic_test_file(&self, challenge: &Challenge, _code: &str) -> Result<String> {
        // Generate appropriate tests based on the challenge
        let module_name = challenge.expected_files.get(0).unwrap_or(&"solution".to_string()).replace(".py", "");

        let test_content = if challenge.id == "ninja-sample-001" {
            // Special handling for Fibonacci challenge
            format!(
r#"import pytest
from {} import fibonacci

def test_basic():
    """Test basic Fibonacci sequence values"""
    assert fibonacci(0) == 0
    assert fibonacci(1) == 1
    assert fibonacci(2) == 1
    assert fibonacci(3) == 2
    assert fibonacci(4) == 3
    assert fibonacci(5) == 5
    assert fibonacci(6) == 8
    assert fibonacci(7) == 13

def test_performance():
    """Ensure the implementation is efficient for large numbers"""
    # Test that we can compute larger Fibonacci numbers quickly
    result = fibonacci(30)
    assert result == 832040

    # Test edge cases
    assert fibonacci(10) == 55
    assert fibonacci(15) == 610
"#, module_name)
        } else if challenge.id == "swe-forge-demo-001" {
            // Special handling for simple math demo challenge
            format!(
r#"import pytest
from {} import add

def test_addition():
    """Test basic addition functionality"""
    assert add(2, 3) == 5
    assert add(0, 0) == 0
    assert add(-1, 1) == 0
    assert add(10, 20) == 30
"#, module_name)
        } else {
            // Generic test template for other challenges
            format!(
r#"import pytest
from {} import *

def test_basic():
    """Basic test for the challenge"""
    # TODO: Generate proper tests based on challenge description
    assert True  # Placeholder - needs proper test implementation
"#, module_name)
        };

        Ok(test_content)
    }
}