use ninja::Challenge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Challenge::from_file() method");

    let challenge_path = "./generated-challenges/fibonacci-sample.json";
    println!("📁 Loading challenge from: {}", challenge_path);

    match Challenge::from_file(challenge_path).await {
        Ok(challenge) => {
            println!("✅ Successfully loaded challenge:");
            println!("   ID: {}", challenge.id);
            println!("   Title: {}", challenge.title);
            println!("   Language: {}", challenge.language);
            println!("   Difficulty: {}", challenge.difficulty);
            println!("   Tests: {} test cases", challenge.tests.len());
            println!("   Setup commands: {} commands", challenge.setup_commands.len());
            println!("   Expected files: {}", challenge.expected_files.join(", "));
        }
        Err(e) => {
            println!("❌ Failed to load challenge: {}", e);
            return Err(e.into());
        }
    }

    println!("🎉 Challenge loading test completed successfully!");
    Ok(())
}