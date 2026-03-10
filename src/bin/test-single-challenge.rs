use ninja::challenge::Challenge;
use ninja::challenge_solver::{ChallengeSolver};
use ninja::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading challenge...");
    let challenge = Challenge::from_file("generated-challenges/fibonacci-sample.json").await?;
    println!("Challenge loaded: {}", challenge.title);
    println!("Setup commands: {:?}", challenge.setup_commands);
    println!("Expected files: {:?}", challenge.expected_files);

    println!("Initializing solver...");
    let config = Config::load_default().await?;
    let solver = ChallengeSolver::new(config).await?;

    println!("Solving challenge...");
    let result = solver.solve_challenge(&challenge).await?;

    println!("=== SOLVE RESULT ===");
    println!("Success: {}", result.success);
    println!("Test Results: {} tests", result.test_results.len());

    for (i, test) in result.test_results.iter().enumerate() {
        println!("  Test {}: {} ({})", i+1, test.test_name, if test.passed { "PASS" } else { "FAIL" });
        if !test.passed {
            println!("    Exit Code: {}", test.exit_code);
            println!("    Output: {}", test.output);
            println!("    Error: {}", test.error_output);
        }
    }

    if let Some(ref code) = result.generated_code {
        println!("\n=== GENERATED CODE ===");
        println!("{}", code);
    }

    if let Some(ref error) = result.error_message {
        println!("\n=== ERROR ===");
        println!("{}", error);
    }

    Ok(())
}