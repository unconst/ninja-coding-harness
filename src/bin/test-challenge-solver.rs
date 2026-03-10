use ninja::{Challenge, ChallengeSolver, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Challenge Solver directly");

    // Load the challenge
    let challenge_path = "./generated-challenges/fibonacci-sample.json";
    println!("📁 Loading challenge from: {}", challenge_path);

    let challenge = Challenge::from_file(challenge_path).await?;
    println!("✅ Challenge loaded: {}", challenge.title);

    // Initialize config and solver
    println!("⚙️  Initializing Challenge Solver...");
    let config = Config::load_default().await?;
    let solver = ChallengeSolver::new(config).await?;
    println!("✅ Challenge Solver initialized");

    // Solve the challenge
    println!("🚀 Starting challenge solve...");
    println!("   Challenge: {} ({})", challenge.title, challenge.language);
    println!("   Tests: {} test cases", challenge.tests.len());

    match solver.solve_challenge(&challenge).await {
        Ok(result) => {
            println!("✅ Challenge solved successfully!");
            println!("   Duration: {}ms", result.duration_ms);
            println!("   Success: {}", result.success);
            println!("   Attempts: {}", result.total_attempts);
            println!("   Test results: {} tests", result.test_results.len());

            if let Some(code) = &result.generated_code {
                println!("   Generated code preview:");
                println!("   {}", code.lines().take(3).collect::<Vec<_>>().join("\n   "));
            }

            if let Some(error) = &result.error_message {
                println!("   Error: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Challenge solving failed: {}", e);
            return Err(e.into());
        }
    }

    println!("🎉 Challenge solver test completed successfully!");
    Ok(())
}