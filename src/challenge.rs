use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a coding challenge similar to SWE-bench format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub repository: Option<String>,
    pub language: String,
    pub difficulty: DifficultyLevel,
    pub tests: Vec<TestCase>,
    pub setup_commands: Vec<String>,
    pub validation_commands: Vec<String>,
    pub expected_files: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: String,  // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub test_type: TestType,
    pub command: String,
    pub expected_result: TestResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    FailToPass,  // Must fail before fix, pass after fix
    PassToPass,  // Must pass both before and after fix (regression test)
    Validation,  // General validation test
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestResult {
    Pass,
    Fail,
    Any,
}

/// Result of solving a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub challenge_id: String,
    pub status: SolutionStatus,
    pub solution_files: Vec<String>,
    pub execution_log: String,
    pub test_results: Vec<TestExecution>,
    pub metrics: SolutionMetrics,
    pub error_message: Option<String>,
    pub completed_at: String,  // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolutionStatus {
    Solved,       // All tests pass
    Partial,      // Some tests pass
    Failed,       // No tests pass or execution failed
    Timeout,      // Exceeded time limit
    Error,        // System error during execution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    pub test_name: String,
    pub status: TestResult,
    pub output: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionMetrics {
    pub total_execution_time_ms: u64,
    pub llm_calls_made: u32,
    pub tokens_used: u32,
    pub tool_calls_made: u32,
    pub files_created: u32,
    pub files_modified: u32,
}

impl Challenge {
    /// Create a sample challenge for demonstration
    pub fn sample() -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Ninja Harness".to_string());
        metadata.insert("category".to_string(), "algorithms".to_string());

        Self {
            id: "ninja-sample-001".to_string(),
            title: "Fibonacci Sequence Implementation".to_string(),
            description: "Implement a function that calculates the nth Fibonacci number efficiently using dynamic programming.".to_string(),
            repository: Some("https://github.com/ninja-harness/challenges".to_string()),
            language: "python".to_string(),
            difficulty: DifficultyLevel::Easy,
            tests: vec![
                TestCase {
                    name: "test_fibonacci_basic".to_string(),
                    description: "Test basic Fibonacci sequence values".to_string(),
                    test_type: TestType::FailToPass,
                    command: "python -m pytest test_fibonacci.py::test_basic -v".to_string(),
                    expected_result: TestResult::Pass,
                },
                TestCase {
                    name: "test_fibonacci_performance".to_string(),
                    description: "Ensure the implementation is efficient for large numbers".to_string(),
                    test_type: TestType::Validation,
                    command: "python -m pytest test_fibonacci.py::test_performance -v".to_string(),
                    expected_result: TestResult::Pass,
                }
            ],
            setup_commands: vec![
                "pip install pytest".to_string(),
                "pip install numpy".to_string(),
            ],
            validation_commands: vec![
                "python -c \"import fibonacci; print('Module imported successfully')\"".to_string(),
            ],
            expected_files: vec![
                "fibonacci.py".to_string(),
                "test_fibonacci.py".to_string(),
            ],
            metadata,
            created_at: "2026-03-10T09:00:00Z".to_string(),
        }
    }

    /// Validate the challenge structure
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Challenge ID cannot be empty".to_string());
        }

        if self.tests.is_empty() {
            return Err("Challenge must have at least one test".to_string());
        }

        // Validate that we have at least one fail-to-pass test
        let has_fail_to_pass = self.tests.iter().any(|t| matches!(t.test_type, TestType::FailToPass));
        if !has_fail_to_pass {
            return Err("Challenge must have at least one fail-to-pass test".to_string());
        }

        Ok(())
    }
}

impl ChallengeResult {
    pub fn new(challenge_id: String) -> Self {
        Self {
            challenge_id,
            status: SolutionStatus::Failed,
            solution_files: Vec::new(),
            execution_log: String::new(),
            test_results: Vec::new(),
            metrics: SolutionMetrics::default(),
            error_message: None,
            completed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        }
    }
}

impl Default for SolutionMetrics {
    fn default() -> Self {
        Self {
            total_execution_time_ms: 0,
            llm_calls_made: 0,
            tokens_used: 0,
            tool_calls_made: 0,
            files_created: 0,
            files_modified: 0,
        }
    }
}