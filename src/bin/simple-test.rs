use ninja::{Config, ContinuousImprovementConfig, ContinuousImprovementLoop, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔍 Simple test - loading config");
    let ninja_config = Config::load_default().await?;

    println!("🔍 Simple test - creating improvement config");
    let improvement_config = ContinuousImprovementConfig::default();

    println!("🔍 Simple test - calling new()");
    let _improvement_loop = ContinuousImprovementLoop::new(improvement_config, ninja_config).await?;

    println!("✅ Simple test - new() completed successfully");

    // Test accessing the struct without calling run_continuous_loop
    println!("✅ Simple test - object created and moved successfully");

    Ok(())
}