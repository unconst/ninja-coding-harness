// 🥷 Ninja Continuous Improvement Loop - Phase 4: Claude Code Parity Quest
//
// This binary runs the autonomous continuous improvement loop designed to
// achieve full Claude Code feature and performance parity through:
//
// 1. Continuous SWE-Forge challenge generation
// 2. Comprehensive Ninja rollout collection and analysis
// 3. Claude-powered performance evaluation and critique
// 4. Automated code improvements based on AI feedback
// 5. Regular Telegram progress reporting
//
// The loop runs indefinitely until Claude Code parity is achieved.

use ninja::{
    Config, ContinuousImprovementLoop, ContinuousImprovementConfig,
    NinjaError, Result,
};
use std::env;
use tracing::{info, error, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize comprehensive logging
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(EnvFilter::from_default_env())
        .init();

    println!("🥷 Ninja Continuous Improvement Loop - Phase 4");
    println!("🎯 Mission: Achieve Claude Code Feature & Performance Parity");
    println!("===============================================================");

    // Environment validation
    println!("\n🔍 Validating environment setup...");

    let mut missing_requirements = Vec::new();

    if env::var("OPENROUTER_API_KEY").is_err() {
        missing_requirements.push("OPENROUTER_API_KEY");
    } else {
        println!("   ✅ OPENROUTER_API_KEY found");
    }

    if env::var("GITHUB_TOKEN").is_err() {
        missing_requirements.push("GITHUB_TOKEN");
    } else {
        println!("   ✅ GITHUB_TOKEN found");
    }

    // Check for optional Telegram bot token
    match env::var("TELEGRAM_BOT_TOKEN") {
        Ok(_) => println!("   ✅ TELEGRAM_BOT_TOKEN found - reporting enabled"),
        Err(_) => {
            println!("   ⚠️  TELEGRAM_BOT_TOKEN not found - reporting will be limited");
            warn!("Telegram reporting may not work without TELEGRAM_BOT_TOKEN");
        }
    }

    if !missing_requirements.is_empty() {
        error!("❌ Missing required environment variables: {:?}", missing_requirements);
        println!("\n🛠️  Setup Instructions:");
        println!("   export OPENROUTER_API_KEY='your-openrouter-key'");
        println!("   export GITHUB_TOKEN='your-github-token'");
        println!("   export TELEGRAM_BOT_TOKEN='your-telegram-bot-token' # optional");
        return Err(NinjaError::Config("Missing required environment variables".to_string()));
    }

    println!("   ✅ All required environment variables found!");

    // Load configuration
    println!("\n⚙️  Loading configuration...");
    let ninja_config = Config::load_default().await?;

    // Create optimized configuration for continuous improvement
    let mut improvement_config = ContinuousImprovementConfig::default();

    // Customize for intensive operation
    improvement_config.generation_config.generation_interval_minutes = 30; // Generate every 30 minutes
    improvement_config.generation_config.max_concurrent_solves = 5; // Higher throughput
    improvement_config.generation_config.auto_improvement_enabled = true;
    improvement_config.telegram_reporting_interval_minutes = 120; // Report every 2 hours
    improvement_config.performance_tracking_window_hours = 24; // Track last 24 hours
    improvement_config.max_rollout_history_gb = 10.0; // Allow more storage for detailed analysis

    println!("   ✅ Configuration loaded");
    println!("      📊 Challenge generation interval: {} minutes",
             improvement_config.generation_config.generation_interval_minutes);
    println!("      💾 Max concurrent solves: {}",
             improvement_config.generation_config.max_concurrent_solves);
    println!("      📱 Telegram reporting interval: {} minutes",
             improvement_config.telegram_reporting_interval_minutes);

    // Initialize continuous improvement loop
    println!("\n🚀 Initializing Continuous Improvement Loop...");

    let mut improvement_loop = match ContinuousImprovementLoop::new(improvement_config, ninja_config).await {
        Ok(loop_instance) => {
            println!("   ✅ Continuous Improvement Loop initialized successfully!");
            loop_instance
        }
        Err(e) => {
            error!("❌ Failed to initialize Continuous Improvement Loop: {}", e);
            return Err(e);
        }
    };

    println!("🔍 DEBUG: After loop creation, before PHASE 4 message");
    println!("\n🎯 PHASE 4: CONTINUOUS OPERATION COMMENCING");
    println!("===========================================");
    println!("🔄 Loop will run indefinitely until Claude Code parity is achieved");
    println!("📊 Progress reports will be sent via Telegram every 2 hours");
    println!("🎯 Target: 100% Claude Code feature and performance compatibility");

    println!("\n🏁 Starting autonomous operation...");
    info!("Continuous Improvement Loop starting autonomous operation");

    // Run the continuous improvement loop
    match improvement_loop.run_continuous_loop().await {
        Ok(_) => {
            // This should never happen as the loop runs indefinitely
            println!("🎉 Continuous Improvement Loop completed successfully!");
            info!("Continuous Improvement Loop completed - this is unexpected");
        }
        Err(e) => {
            error!("❌ Continuous Improvement Loop failed: {}", e);
            println!("❌ Loop terminated with error: {}", e);

            // Try to send an emergency Telegram notification
            if let Err(telegram_err) = send_emergency_notification(&e).await {
                error!("Failed to send emergency notification: {}", telegram_err);
            }

            return Err(e);
        }
    }

    Ok(())
}

async fn send_emergency_notification(error: &NinjaError) -> Result<()> {
    use tokio::process::Command;

    let message = format!(
        "🚨 **NINJA LOOP EMERGENCY STOP**\n\n\
         The continuous improvement loop has terminated unexpectedly.\n\n\
         Error: {}\n\n\
         Please investigate and restart the loop.",
        error
    );

    let output = Command::new("python")
        .arg("/Arbos/tools/send_telegram.py")
        .arg(&message)
        .output()
        .await
        .map_err(|e| NinjaError::Unknown(format!("Failed to send emergency notification: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        warn!("Emergency notification failed: {}", error_msg);
    } else {
        info!("Emergency notification sent successfully");
    }

    Ok(())
}

// Additional utility functions for the continuous loop binary

fn print_startup_banner() {
    println!(r#"
    🥷 NINJA CONTINUOUS IMPROVEMENT LOOP 🔄

    ╔══════════════════════════════════════╗
    ║          PHASE 4: ACTIVE             ║
    ║    CLAUDE CODE PARITY QUEST          ║
    ╚══════════════════════════════════════╝

    Target: 100% Claude Code Compatibility
    Method: Autonomous AI-Driven Optimization
    Duration: Continuous until target achieved

    "#);
}

fn print_progress_indicators() {
    println!("📊 Progress Tracking:");
    println!("   ✅ Feature Parity Checklist");
    println!("   ⚡ Performance Benchmarks");
    println!("   🧠 AI Evaluation Scoring");
    println!("   📱 Telegram Progress Reports");
    println!("   🔧 Automatic Code Improvements");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_validation() {
        // Test environment validation logic
        assert!(true); // Placeholder
    }

    #[tokio::test]
    async fn test_configuration_loading() {
        // Test configuration loading
        let config = ContinuousImprovementConfig::default();
        assert_eq!(config.telegram_reporting_interval_minutes, 120);
    }
}