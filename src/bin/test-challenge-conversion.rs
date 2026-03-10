use ninja::challenge_generation::{ChallengeGenerator, ChallengeGenerationConfig, SweForgeConfig};
use ninja::{Config, ChallengeSolver};
use ninja::challenge_generation::PerformanceTracker;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Challenge conversion and pipeline setup");

    // Create configuration
    let config = Config::default();
    let swe_config = SweForgeConfig {
        binary_path: "/Arbos/swe-forge/target/release/swe-forge".to_string(),
        max_challenges: 3,
        repositories: vec!["python".to_string(), "rust".to_string()],
        output_directory: "./generated-challenges".to_string(),
        difficulty: "easy".to_string(),
        min_stars: Some(5),
    };

    let challenge_gen_config = ChallengeGenerationConfig {
        swe_forge: swe_config,
        max_concurrent_solves: 3,
        batch_interval_minutes: 30,
    };

    // Initialize components
    let challenge_solver = Arc::new(ChallengeSolver::new(config.clone()).await?);
    let performance_tracker = Arc::new(Mutex::new(PerformanceTracker::new()));

    // Create generator
    let generator = ChallengeGenerator::new(
        challenge_gen_config,
        challenge_solver,
        performance_tracker,
    ).await?;

    println!("✅ Challenge generator initialized successfully");

    // Test loading existing challenges
    println!("📁 Testing load_existing_challenges method...");

    // This calls load_existing_challenges internally which was hanging
    println!("🎯 About to call generate_and_solve_batch (this was hanging before)");
    match generator.generate_and_solve_batch().await {
        Ok(result) => {
            println!("✅ Successfully completed batch generation:");
            println!("   Generated: {} challenges", result.generated_count);
            println!("   Solved: {} challenges", result.solved_count);
            println!("   Failed: {} challenges", result.failed_count);
            println!("   Generation time: {}ms", result.generation_time_ms);
            println!("   Total solve time: {}ms", result.total_solve_time_ms);
        }
        Err(e) => {
            println!("❌ Batch generation failed: {}", e);
            return Err(e.into());
        }
    }

    println!("🎉 Challenge conversion test completed successfully!");
    Ok(())
}