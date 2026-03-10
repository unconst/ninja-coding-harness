// 🥷 Ninja Coding Harness - A Self-Improving Coding Challenge Solver
//
// This is the initial prototype implementation of the Ninja coding harness,
// designed to solve programming challenges and continuously improve through
// synthetic challenge generation and feedback loops.
//
// Architecture inspired by SWE-Forge research and designed for:
// - Challenge-solving with LLM agents
// - Docker-based sandbox execution
// - Continuous self-improvement loops
// - OpenRouter API integration
//
// Current status: Prototype implementation with core structure in place

use std::env;

mod challenge;
mod config;
mod error;
mod llm;
// mod harness;
// mod executor;
mod executor_simple;

use config::Config;
use llm::{LlmProvider, OpenRouterProvider, ChatRequest, ChatMessage};
use executor_simple::{SimpleCodeExecutor, ExecutionRequest, FileOperation, FileOperationType};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🥷 Ninja Coding Harness v0.1.0");
    println!("A self-improving coding challenge solver");
    println!("=====================================");

    // Check environment setup
    println!("\n🔍 Checking environment...");

    if env::var("OPENROUTER_API_KEY").is_ok() {
        println!("✅ OPENROUTER_API_KEY found");
    } else {
        println!("⚠️  OPENROUTER_API_KEY not set (will be needed for LLM features)");
    }

    // Display architecture overview
    println!("\n🏗️  Architecture Overview:");
    println!("   📁 Challenge Management - JSON-based challenge definitions");
    println!("   🧠 LLM Integration - OpenRouter API for code generation");
    println!("   🐳 Docker Sandbox - Isolated execution environment");
    println!("   🔄 Improvement Loop - Synthetic challenge generation");
    println!("   📊 Metrics & Analysis - Performance tracking");

    // Display core modules status
    println!("\n📦 Core Modules:");
    println!("   ✅ challenge.rs - Challenge definition structures");
    println!("   🚧 config.rs - Configuration management");
    println!("   🚧 llm.rs - Language model integration");
    println!("   🚧 executor.rs - Docker-based code execution");
    println!("   🚧 harness.rs - Main challenge-solving orchestrator");

    // Show next development steps
    println!("\n🎯 Development Roadmap:");
    println!("   Phase 1: Core harness implementation");
    println!("   Phase 2: SWE-Forge integration for synthetic challenges");
    println!("   Phase 3: Self-improvement and optimization loops");

    // Sample challenge format
    println!("\n📄 Challenge Format Example:");
    let sample = challenge::Challenge::sample();
    let json = serde_json::to_string_pretty(&sample)?;
    println!("{}", json);

    println!("\n🚀 Testing LLM Integration...");

    // Test LLM integration
    if env::var("OPENROUTER_API_KEY").is_ok() {
        match test_llm_integration().await {
            Ok(response) => {
                println!("✅ LLM Integration Test Successful!");
                println!("   Response: {}", response.trim());
            },
            Err(e) => {
                println!("❌ LLM Integration Test Failed: {}", e);
            }
        }
    } else {
        println!("⚠️  LLM test skipped - API key not configured");
    }

    println!("\n🐳 Testing Docker Integration...");

    match test_docker_integration().await {
        Ok(result) => {
            println!("✅ Docker Integration Test Successful!");
            println!("   Exit Code: {}", result.0);
            println!("   Output: {}", result.1.trim());
        },
        Err(e) => {
            println!("❌ Docker Integration Test Failed: {}", e);
        }
    }

    println!("\n🚀 Ninja harness foundation is ready!");
    println!("   Phase 2: LLM integration and Docker execution");

    Ok(())
}

async fn test_llm_integration() -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::load_default().await?;
    let provider = OpenRouterProvider::new(config.openrouter);

    let request = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Say 'Hello, World!' in exactly 2 words".to_string(),
        }],
        temperature: Some(0.1),
        max_tokens: Some(50),
        tools: None,
        tool_choice: None,
    };

    let response = provider.chat_completion(request).await?;
    Ok(response.content)
}

async fn test_docker_integration() -> Result<(i32, String), Box<dyn std::error::Error>> {
    let config = Config::load_default().await?;
    let executor = SimpleCodeExecutor::new(config.docker).await?;

    // Create a simple Python test script
    let files = vec![
        FileOperation {
            path: "/app/test.py".to_string(),
            content: "print('Hello from Docker!')".to_string(),
            operation_type: FileOperationType::Create,
        }
    ];

    let mut env = HashMap::new();
    env.insert("PYTHONPATH".to_string(), "/app".to_string());

    let request = ExecutionRequest {
        command: "python /app/test.py".to_string(),
        working_directory: "/app".to_string(),
        files,
        environment: env,
        timeout_seconds: 30,
    };

    let result = executor.execute(request).await?;
    Ok((result.exit_code, result.stdout))
}
