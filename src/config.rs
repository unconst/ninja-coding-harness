use crate::error::{NinjaError, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openrouter: OpenRouterConfig,
    pub docker: DockerConfig,
    pub execution: ExecutionConfig,
    pub improvement: ImprovementConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub enabled: bool,
    pub base_image: String,
    pub timeout_seconds: u64,
    pub memory_limit: String,
    pub cpu_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub max_tools_per_session: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementConfig {
    pub enabled: bool,
    pub challenge_generation_rate: u32,
    pub analysis_frequency: u32,
    pub auto_deploy: bool,
}

impl Config {
    pub async fn load_default() -> Result<Self> {
        let openrouter_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| NinjaError::Config("OPENROUTER_API_KEY environment variable not set".to_string()))?;

        Ok(Config {
            openrouter: OpenRouterConfig {
                api_key: openrouter_key,
                base_url: "https://openrouter.ai/api/v1".to_string(),
                model: "anthropic/claude-3.5-sonnet-20241022".to_string(),
                temperature: 0.1,
                max_tokens: 8192,
            },
            docker: DockerConfig {
                enabled: true,
                base_image: "python:3.12-slim".to_string(),
                timeout_seconds: 300,
                memory_limit: "1g".to_string(),
                cpu_limit: "1.0".to_string(),
            },
            execution: ExecutionConfig {
                max_retries: 3,
                timeout_seconds: 120,
                max_tools_per_session: 50,
            },
            improvement: ImprovementConfig {
                enabled: true,
                challenge_generation_rate: 10,
                analysis_frequency: 100,
                auto_deploy: false,
            },
        })
    }

    pub async fn load_from_file(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub async fn save_to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}