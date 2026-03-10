use crate::config::OpenRouterConfig;
use crate::error::{NinjaError, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, trace};

/// LLM provider trait for abstraction
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn function_call(&self, request: FunctionCallRequest) -> Result<FunctionCallResponse>;
}

/// OpenRouter LLM provider implementation
pub struct OpenRouterProvider {
    client: Client,
    config: OpenRouterConfig,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallRequest {
    pub prompt: String,
    pub functions: Vec<ToolDefinition>,
    pub required_function: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallResponse {
    pub function_name: String,
    pub arguments: Value,
    pub raw_response: String,
}

impl OpenRouterProvider {
    pub fn new(config: OpenRouterConfig) -> Self {
        let client = Client::new();
        let semaphore = Arc::new(Semaphore::new(5)); // Rate limit to 5 concurrent requests

        Self {
            client,
            config,
            semaphore,
        }
    }

    async fn make_request(&self, payload: Value) -> Result<Response> {
        let _permit = self.semaphore.acquire().await.unwrap();

        debug!("Making OpenRouter API request");
        trace!("Request payload: {}", serde_json::to_string_pretty(&payload).unwrap());

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(NinjaError::http_client(format!("OpenRouter API error: {}", error_text)));
        }

        Ok(response)
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse> {
        let payload = json!({
            "model": self.config.model,
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(self.config.temperature),
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "tools": request.tools,
            "tool_choice": request.tool_choice
        });

        let response = self.make_request(payload).await?;
        let response_json: Value = response.json().await?;

        trace!("OpenRouter response: {}", serde_json::to_string_pretty(&response_json).unwrap());

        // Parse response
        let choice = response_json["choices"][0].clone();
        let message = choice["message"].clone();

        let content = message["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = TokenUsage {
            prompt_tokens: response_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: response_json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: response_json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        // Parse tool calls if present
        let tool_calls = if let Some(tool_calls_json) = message.get("tool_calls") {
            if tool_calls_json.is_array() {
                let mut calls = Vec::new();
                for call in tool_calls_json.as_array().unwrap() {
                    if let (Some(id), Some(name), Some(args)) = (
                        call["id"].as_str(),
                        call["function"]["name"].as_str(),
                        call["function"].get("arguments"),
                    ) {
                        calls.push(ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments: args.clone(),
                        });
                    }
                }
                Some(calls)
            } else {
                None
            }
        } else {
            None
        };

        Ok(ChatResponse {
            content,
            usage,
            tool_calls,
        })
    }

    async fn function_call(&self, request: FunctionCallRequest) -> Result<FunctionCallResponse> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: request.prompt,
        }];

        let chat_request = ChatRequest {
            messages,
            temperature: Some(0.1),
            max_tokens: None,
            tools: Some(request.functions),
            tool_choice: request.required_function,
        };

        let response = self.chat_completion(chat_request).await?;

        if let Some(tool_calls) = response.tool_calls {
            if let Some(first_call) = tool_calls.first() {
                return Ok(FunctionCallResponse {
                    function_name: first_call.name.clone(),
                    arguments: first_call.arguments.clone(),
                    raw_response: response.content,
                });
            }
        }

        // Fallback if no tool calls were made
        Err(NinjaError::http_client("No function call found in response".to_string()))
    }
}

// Utility functions for creating common tool definitions
pub fn create_coding_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: "execute_command".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Command to execute"
                    },
                    "working_directory": {
                        "type": "string",
                        "description": "Working directory for the command"
                    }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List files in a directory".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to list files recursively"
                    }
                },
                "required": ["path"]
            }),
        },
    ]
}