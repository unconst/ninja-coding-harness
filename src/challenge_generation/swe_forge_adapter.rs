// 🥷 SWE-Forge Integration Adapter
//
// This module adapts SWE-Forge functionality for use with the Ninja harness.
// It provides a simplified interface to generate challenges from GitHub PRs.

use crate::challenge::{Challenge, TestCase, TestType, TestResult, DifficultyLevel};
use crate::error::{NinjaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tracing::{debug, info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweForgeConfig {
    pub swe_forge_binary: String,
    pub github_token: String,
    pub openrouter_api_key: String,
    pub output_dir: String,
    pub max_tasks: usize,
    pub difficulty: Option<String>,
    pub min_stars: Option<u32>,
    pub languages: Option<Vec<String>>,
}

impl Default for SweForgeConfig {
    fn default() -> Self {
        Self {
            swe_forge_binary: "swe-forge".to_string(),
            github_token: std::env::var("GITHUB_TOKEN").unwrap_or_default(),
            openrouter_api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_default(),
            output_dir: "./generated-challenges".to_string(),
            max_tasks: 5,
            difficulty: Some("medium".to_string()),
            min_stars: Some(20),
            languages: Some(vec!["python".to_string(), "rust".to_string(), "javascript".to_string()]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedChallenge {
    pub task_id: String,
    pub repository: String,
    pub difficulty: String,
    pub workspace_path: PathBuf,
    pub challenge: Challenge,
}

pub struct SweForgeAdapter {
    config: SweForgeConfig,
}

impl SweForgeAdapter {
    pub fn new(config: SweForgeConfig) -> Self {
        Self { config }
    }

    pub async fn generate_challenges(&self) -> Result<Vec<GeneratedChallenge>> {
        info!("Starting SWE-Forge challenge generation");

        // Ensure output directory exists
        fs::create_dir_all(&self.config.output_dir).await?;

        // Build SWE-Forge command
        let mut cmd = Command::new(&self.config.swe_forge_binary);
        cmd.arg("swe")
            .arg("mine")
            .arg("--output")
            .arg(&self.config.output_dir)
            .arg("--max-tasks")
            .arg(self.config.max_tasks.to_string())
            .arg("--once"); // Run once, don't loop continuously

        // Add optional parameters
        if let Some(difficulty) = &self.config.difficulty {
            cmd.arg("--difficulty").arg(difficulty);
        }

        if let Some(min_stars) = self.config.min_stars {
            cmd.arg("--min-stars").arg(min_stars.to_string());
        }

        if let Some(languages) = &self.config.languages {
            let lang_string = languages.join(",");
            cmd.arg("--languages").arg(lang_string);
        }

        // Set environment variables
        cmd.env("GITHUB_TOKEN", &self.config.github_token)
           .env("OPENROUTER_API_KEY", &self.config.openrouter_api_key);

        info!("Executing SWE-Forge: {:?}", cmd);

        // Execute the command
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("SWE-Forge execution failed: {}", stderr);
            return Err(NinjaError::Execution(format!("SWE-Forge failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("SWE-Forge output: {}", stdout);

        // Parse the generated challenges
        self.parse_generated_challenges().await
    }

    async fn parse_generated_challenges(&self) -> Result<Vec<GeneratedChallenge>> {
        let output_path = Path::new(&self.config.output_dir);

        if !output_path.exists() {
            warn!("Output directory does not exist: {}", self.config.output_dir);
            return Ok(vec![]);
        }

        let mut challenges = Vec::new();
        let mut entries = fs::read_dir(output_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                debug!("Processing workspace directory: {:?}", path);

                if let Ok(challenge) = self.parse_workspace_yaml(&path).await {
                    challenges.push(challenge);
                }
            }
        }

        info!("Successfully parsed {} challenges", challenges.len());
        Ok(challenges)
    }

    async fn parse_workspace_yaml(&self, workspace_path: &Path) -> Result<GeneratedChallenge> {
        let yaml_path = workspace_path.join("workspace.yaml");

        if !yaml_path.exists() {
            return Err(NinjaError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "workspace.yaml not found"
            )));
        }

        let yaml_content = fs::read_to_string(&yaml_path).await?;
        let workspace_data: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;

        // Extract key information from workspace.yaml
        let task_id = workspace_data["instance_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let repository = workspace_data["repo"]
            .as_str()
            .unwrap_or("unknown/unknown")
            .to_string();

        let problem_statement = workspace_data["problem_statement"]
            .as_str()
            .unwrap_or("No description provided")
            .to_string();

        // Convert to Ninja Challenge format
        let difficulty = match self.config.difficulty.as_ref().map(|s| s.as_str()).unwrap_or("medium") {
            "easy" => DifficultyLevel::Easy,
            "hard" => DifficultyLevel::Hard,
            _ => DifficultyLevel::Medium,
        };

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "swe-forge".to_string());
        metadata.insert("repository".to_string(), repository.clone());

        let ninja_challenge = Challenge {
            id: task_id.clone(),
            title: format!("SWE Challenge: {}", task_id),
            description: problem_statement,
            repository: Some(repository.clone()),
            language: self.detect_language(workspace_path).await.unwrap_or("python".to_string()),
            difficulty,
            expected_files: self.extract_expected_files(&workspace_data).await,
            setup_commands: self.extract_setup_commands(&workspace_data).await,
            validation_commands: vec![], // Add validation commands if needed
            tests: self.extract_tests(&workspace_data).await,
            metadata,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(GeneratedChallenge {
            task_id,
            repository,
            difficulty: self.config.difficulty.clone().unwrap_or("medium".to_string()),
            workspace_path: workspace_path.to_path_buf(),
            challenge: ninja_challenge,
        })
    }

    async fn detect_language(&self, _workspace_path: &Path) -> Option<String> {
        // TODO: Implement language detection based on file extensions
        // For now, default to Python
        Some("python".to_string())
    }

    async fn extract_expected_files(&self, workspace_data: &serde_yaml::Value) -> Vec<String> {
        // Extract file paths from the workspace data
        // This is a simplified implementation - real SWE-bench format is more complex
        vec!["solution.py".to_string()]
    }

    async fn extract_setup_commands(&self, _workspace_data: &serde_yaml::Value) -> Vec<String> {
        // Extract setup commands from workspace data
        vec!["pip install -e .".to_string()]
    }

    async fn extract_tests(&self, workspace_data: &serde_yaml::Value) -> Vec<crate::challenge::TestCase> {
        let mut tests = Vec::new();

        // Extract fail_to_pass tests
        if let Some(fail_to_pass) = workspace_data["fail_to_pass"].as_sequence() {
            for (i, test_cmd) in fail_to_pass.iter().enumerate() {
                if let Some(cmd) = test_cmd.as_str() {
                    tests.push(TestCase {
                        name: format!("fail_to_pass_{}", i),
                        description: "Fail-to-pass test from SWE-Forge".to_string(),
                        test_type: TestType::FailToPass,
                        command: cmd.to_string(),
                        expected_result: TestResult::Pass,
                    });
                }
            }
        }

        // Extract pass_to_pass tests
        if let Some(pass_to_pass) = workspace_data["pass_to_pass"].as_sequence() {
            for (i, test_cmd) in pass_to_pass.iter().enumerate() {
                if let Some(cmd) = test_cmd.as_str() {
                    tests.push(TestCase {
                        name: format!("pass_to_pass_{}", i),
                        description: "Pass-to-pass test from SWE-Forge".to_string(),
                        test_type: TestType::PassToPass,
                        command: cmd.to_string(),
                        expected_result: TestResult::Pass,
                    });
                }
            }
        }

        // If no tests found, create a basic test
        if tests.is_empty() {
            tests.push(TestCase {
                name: "basic_test".to_string(),
                description: "Basic test fallback".to_string(),
                test_type: TestType::Validation,
                command: "python -m pytest -xvs".to_string(),
                expected_result: TestResult::Pass,
            });
        }

        tests
    }

    pub async fn install_swe_forge(&self) -> Result<()> {
        info!("Installing or updating SWE-Forge");

        // Check if SWE-Forge is already installed
        let output = Command::new("which")
            .arg("swe-forge")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                info!("SWE-Forge already installed");
                return Ok(());
            }
            _ => {
                info!("SWE-Forge not found, building from source");
            }
        }

        // Build SWE-Forge from the cloned repository
        let build_output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir("../swe-forge")
            .output()?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            error!("Failed to build SWE-Forge: {}", stderr);
            return Err(NinjaError::Execution(format!("SWE-Forge build failed: {}", stderr)));
        }

        info!("SWE-Forge built successfully");
        Ok(())
    }
}

// std::io::Error conversion is already handled by the main error module

impl From<serde_yaml::Error> for NinjaError {
    fn from(err: serde_yaml::Error) -> Self {
        NinjaError::Serialization(err.to_string())
    }
}