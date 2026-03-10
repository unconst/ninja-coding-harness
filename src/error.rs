use thiserror::Error;

/// Main error type for the Ninja harness
#[derive(Error, Debug)]
pub enum NinjaError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("LLM API error: {0}")]
    Llm(#[from] reqwest::Error),

    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Challenge parsing error: {0}")]
    ChallengeParse(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl NinjaError {
    pub fn http_client(message: String) -> Self {
        NinjaError::Unknown(message)
    }
}

pub type Result<T> = std::result::Result<T, NinjaError>;