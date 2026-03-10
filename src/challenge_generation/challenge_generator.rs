// 🥷 Challenge Generator - Core orchestrator for synthetic challenge generation
//
// This module provides the main interface for generating coding challenges using
// SWE-Forge methodology and feeding them to the Ninja harness.

use crate::challenge::Challenge;
use crate::challenge_solver::{ChallengeSolver, SolveResult};
use crate::config::Config;
use crate::error::{NinjaError, Result};
use crate::challenge_generation::{SweForgeAdapter, SweForgeConfig, GeneratedChallenge, PerformanceTracker};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeGenerationConfig {
    pub swe_forge: SweForgeConfig,
    pub generation_interval_minutes: u64,
    pub max_concurrent_solves: usize,
    pub performance_analysis_threshold: usize,
    pub auto_improvement_enabled: bool,
}

impl Default for ChallengeGenerationConfig {
    fn default() -> Self {
        Self {
            swe_forge: SweForgeConfig::default(),
            generation_interval_minutes: 60, // Generate new challenges every hour
            max_concurrent_solves: 3,
            performance_analysis_threshold: 10, // Analyze after every 10 challenges
            auto_improvement_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeGenerationResult {
    pub generated_count: usize,
    pub solved_count: usize,
    pub failed_count: usize,
    pub generation_time_ms: u64,
    pub total_solve_time_ms: u64,
    pub challenges: Vec<ChallengeAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeAttempt {
    pub challenge: GeneratedChallenge,
    pub solve_result: SolveResult,
    pub performance_metrics: ChallengePerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengePerformanceMetrics {
    pub challenge_complexity_score: f64,
    pub solve_time_ms: u64,
    pub token_usage_estimated: u32,
    pub docker_execution_time_ms: u64,
    pub success_rate: f64,
    pub error_types: Vec<String>,
}

pub struct ChallengeGenerator {
    config: ChallengeGenerationConfig,
    swe_forge: SweForgeAdapter,
    challenge_solver: Arc<ChallengeSolver>,
    performance_tracker: Arc<Mutex<PerformanceTracker>>,
}

impl ChallengeGenerator {
    pub async fn new(config: ChallengeGenerationConfig, ninja_config: Config) -> Result<Self> {
        info!("Initializing Challenge Generator");

        let swe_forge = SweForgeAdapter::new(config.swe_forge.clone());

        // Ensure SWE-Forge is available
        swe_forge.install_swe_forge().await?;

        let challenge_solver = Arc::new(ChallengeSolver::new(ninja_config).await?);
        let performance_tracker = Arc::new(Mutex::new(PerformanceTracker::new()));

        Ok(Self {
            config,
            swe_forge,
            challenge_solver,
            performance_tracker,
        })
    }

    pub async fn run_continuous_generation_loop(&self) -> Result<()> {
        info!("Starting continuous challenge generation loop");

        loop {
            match self.generate_and_solve_batch().await {
                Ok(result) => {
                    info!("Generation batch completed: {} generated, {} solved",
                          result.generated_count, result.solved_count);

                    // Update performance tracking
                    {
                        let mut tracker = self.performance_tracker.lock().await;
                        for attempt in &result.challenges {
                            tracker.record_attempt(&attempt.challenge.challenge, &attempt.solve_result).await;
                        }
                    }

                    // Check if we should run performance analysis
                    if result.challenges.len() >= self.config.performance_analysis_threshold {
                        if let Err(e) = self.run_performance_analysis().await {
                            warn!("Performance analysis failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Generation batch failed: {}", e);
                }
            }

            // Wait before next generation cycle
            let sleep_duration = Duration::from_secs(self.config.generation_interval_minutes * 60);
            info!("Sleeping for {} minutes until next generation cycle",
                  self.config.generation_interval_minutes);
            sleep(sleep_duration).await;
        }
    }

    pub async fn generate_and_solve_batch(&self) -> Result<ChallengeGenerationResult> {
        let start_time = Instant::now();
        info!("Starting challenge generation batch");

        // Generate challenges using SWE-Forge
        let generated_challenges = self.swe_forge.generate_challenges().await?;
        let generated_count = generated_challenges.len();

        info!("Generated {} challenges from SWE-Forge", generated_count);

        if generated_challenges.is_empty() {
            warn!("No challenges were generated - this may indicate a configuration issue");
            return Ok(ChallengeGenerationResult {
                generated_count: 0,
                solved_count: 0,
                failed_count: 0,
                generation_time_ms: start_time.elapsed().as_millis() as u64,
                total_solve_time_ms: 0,
                challenges: vec![],
            });
        }

        let generation_time = start_time.elapsed().as_millis() as u64;

        // Solve challenges concurrently
        let solve_start = Instant::now();
        let mut challenge_attempts = Vec::new();
        let mut solved_count = 0;
        let mut failed_count = 0;

        // Process challenges in batches to respect concurrency limits
        let chunks: Vec<_> = generated_challenges.chunks(self.config.max_concurrent_solves).collect();

        for chunk in chunks {
            let solve_tasks: Vec<_> = chunk.iter().map(|challenge| {
                let solver = Arc::clone(&self.challenge_solver);
                let challenge_clone = challenge.clone();

                async move {
                    let solve_start = Instant::now();
                    let solve_result = solver.solve_challenge(&challenge_clone.challenge).await?;
                    let solve_time = solve_start.elapsed().as_millis() as u64;

                    let performance_metrics = ChallengePerformanceMetrics {
                        challenge_complexity_score: Self::calculate_complexity_score(&challenge_clone.challenge),
                        solve_time_ms: solve_time,
                        token_usage_estimated: Self::estimate_token_usage(&solve_result),
                        docker_execution_time_ms: solve_result.duration_ms,
                        success_rate: if solve_result.success { 1.0 } else { 0.0 },
                        error_types: Self::extract_error_types(&solve_result),
                    };

                    Ok::<ChallengeAttempt, NinjaError>(ChallengeAttempt {
                        challenge: challenge_clone,
                        solve_result,
                        performance_metrics,
                    })
                }
            }).collect();

            // Wait for all tasks in the current chunk to complete
            let chunk_results = futures::future::join_all(solve_tasks).await;

            for result in chunk_results {
                match result {
                    Ok(attempt) => {
                        if attempt.solve_result.success {
                            solved_count += 1;
                        } else {
                            failed_count += 1;
                        }
                        challenge_attempts.push(attempt);
                    }
                    Err(e) => {
                        error!("Challenge solve failed: {}", e);
                        failed_count += 1;
                    }
                }
            }

            // Small delay between chunks to avoid overwhelming the system
            sleep(Duration::from_secs(1)).await;
        }

        let total_solve_time = solve_start.elapsed().as_millis() as u64;

        info!("Batch complete: {} generated, {} solved, {} failed in {}ms",
              generated_count, solved_count, failed_count, total_solve_time);

        Ok(ChallengeGenerationResult {
            generated_count,
            solved_count,
            failed_count,
            generation_time_ms: generation_time,
            total_solve_time_ms: total_solve_time,
            challenges: challenge_attempts,
        })
    }

    async fn run_performance_analysis(&self) -> Result<()> {
        info!("Running performance analysis");

        let tracker = self.performance_tracker.lock().await;
        let analysis = tracker.generate_analysis_report().await;

        info!("Performance Analysis Results:");
        info!("  Total attempts: {}", analysis.total_attempts);
        info!("  Success rate: {:.2}%", analysis.overall_success_rate * 100.0);
        info!("  Average solve time: {}ms", analysis.average_solve_time_ms);
        info!("  Average complexity: {:.2}", analysis.average_complexity_score);

        // TODO: Implement auto-improvement based on analysis
        if self.config.auto_improvement_enabled {
            info!("Auto-improvement enabled - analyzing patterns for optimization");
            // This would implement the self-improvement loop
        }

        Ok(())
    }

    fn calculate_complexity_score(challenge: &Challenge) -> f64 {
        // Simple heuristic for challenge complexity
        let description_length = challenge.description.len() as f64;
        let test_count = challenge.tests.len() as f64;
        let setup_complexity = challenge.setup_commands.len() as f64;

        // Normalize and combine factors
        (description_length / 1000.0).min(1.0) * 0.4 +
        (test_count / 10.0).min(1.0) * 0.4 +
        (setup_complexity / 5.0).min(1.0) * 0.2
    }

    fn estimate_token_usage(solve_result: &SolveResult) -> u32 {
        // Rough estimation based on generated code length and test results
        let code_tokens = solve_result.generated_code
            .as_ref()
            .map(|code| (code.len() / 4) as u32) // Rough token estimation
            .unwrap_or(0);

        let test_output_tokens: u32 = solve_result.test_results
            .iter()
            .map(|test| (test.output.len() / 4) as u32)
            .sum();

        code_tokens + test_output_tokens
    }

    fn extract_error_types(solve_result: &SolveResult) -> Vec<String> {
        let mut error_types = Vec::new();

        if let Some(error_msg) = &solve_result.error_message {
            if error_msg.contains("timeout") {
                error_types.push("timeout".to_string());
            }
            if error_msg.contains("compile") || error_msg.contains("syntax") {
                error_types.push("compilation_error".to_string());
            }
            if error_msg.contains("test") {
                error_types.push("test_failure".to_string());
            }
        }

        for test_result in &solve_result.test_results {
            if !test_result.passed {
                if test_result.error_output.contains("ImportError") {
                    error_types.push("import_error".to_string());
                } else if test_result.error_output.contains("AssertionError") {
                    error_types.push("assertion_error".to_string());
                } else if test_result.exit_code != 0 {
                    error_types.push("runtime_error".to_string());
                }
            }
        }

        error_types.sort();
        error_types.dedup();
        error_types
    }
}