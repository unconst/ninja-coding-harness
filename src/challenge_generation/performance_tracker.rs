// 🥷 Performance Tracker - Tracks and analyzes Ninja harness performance
//
// This module collects performance metrics across challenge solving attempts
// and provides analysis for the continuous improvement loop.

use crate::challenge::Challenge;
use crate::challenge_solver::SolveResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub timestamp: u64,
    pub challenge_id: String,
    pub difficulty: String,
    pub language: String,
    pub success: bool,
    pub solve_time_ms: u64,
    pub test_count: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub error_types: Vec<String>,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub total_attempts: usize,
    pub overall_success_rate: f64,
    pub average_solve_time_ms: u64,
    pub average_complexity_score: f64,
    pub success_rate_by_difficulty: HashMap<String, f64>,
    pub success_rate_by_language: HashMap<String, f64>,
    pub most_common_errors: Vec<(String, usize)>,
    pub performance_trends: Vec<PerformanceTrend>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub metric: String,
    pub direction: String, // "improving", "declining", "stable"
    pub change_percentage: f64,
    pub confidence: f64,
}

pub struct PerformanceTracker {
    records: Vec<PerformanceRecord>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub async fn record_attempt(&mut self, challenge: &Challenge, solve_result: &SolveResult) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let passed_tests = solve_result.test_results.iter().filter(|t| t.passed).count();
        let failed_tests = solve_result.test_results.len() - passed_tests;

        let error_types = self.extract_error_types_from_result(solve_result);
        let complexity_score = self.calculate_complexity_score(challenge);

        let record = PerformanceRecord {
            timestamp,
            challenge_id: challenge.id.clone(),
            difficulty: challenge.difficulty.to_string(),
            language: challenge.language.clone(),
            success: solve_result.success,
            solve_time_ms: solve_result.duration_ms,
            test_count: solve_result.test_results.len(),
            passed_tests,
            failed_tests,
            error_types,
            complexity_score,
        };

        debug!("Recording performance: success={}, time={}ms, complexity={:.2}",
               record.success, record.solve_time_ms, record.complexity_score);

        self.records.push(record);
    }

    pub async fn generate_analysis_report(&self) -> PerformanceAnalysis {
        info!("Generating performance analysis from {} records", self.records.len());

        if self.records.is_empty() {
            return PerformanceAnalysis {
                total_attempts: 0,
                overall_success_rate: 0.0,
                average_solve_time_ms: 0,
                average_complexity_score: 0.0,
                success_rate_by_difficulty: HashMap::new(),
                success_rate_by_language: HashMap::new(),
                most_common_errors: Vec::new(),
                performance_trends: Vec::new(),
                recommendations: Vec::new(),
            };
        }

        let total_attempts = self.records.len();
        let successful_attempts = self.records.iter().filter(|r| r.success).count();
        let overall_success_rate = successful_attempts as f64 / total_attempts as f64;

        let average_solve_time_ms = if !self.records.is_empty() {
            self.records.iter().map(|r| r.solve_time_ms).sum::<u64>() / self.records.len() as u64
        } else {
            0
        };

        let average_complexity_score = if !self.records.is_empty() {
            self.records.iter().map(|r| r.complexity_score).sum::<f64>() / self.records.len() as f64
        } else {
            0.0
        };

        let success_rate_by_difficulty = self.calculate_success_rate_by_field(|r| &r.difficulty);
        let success_rate_by_language = self.calculate_success_rate_by_field(|r| &r.language);
        let most_common_errors = self.calculate_most_common_errors();
        let performance_trends = self.calculate_performance_trends();
        let recommendations = self.generate_recommendations(&performance_trends, overall_success_rate);

        PerformanceAnalysis {
            total_attempts,
            overall_success_rate,
            average_solve_time_ms,
            average_complexity_score,
            success_rate_by_difficulty,
            success_rate_by_language,
            most_common_errors,
            performance_trends,
            recommendations,
        }
    }

    fn calculate_success_rate_by_field<F>(&self, field_extractor: F) -> HashMap<String, f64>
    where
        F: Fn(&PerformanceRecord) -> &String,
    {
        let mut field_stats: HashMap<String, (usize, usize)> = HashMap::new();

        for record in &self.records {
            let field_value = field_extractor(record);
            let entry = field_stats.entry(field_value.clone()).or_insert((0, 0));
            entry.0 += 1; // total count
            if record.success {
                entry.1 += 1; // success count
            }
        }

        field_stats
            .into_iter()
            .map(|(field, (total, success))| {
                (field, success as f64 / total as f64)
            })
            .collect()
    }

    fn calculate_most_common_errors(&self) -> Vec<(String, usize)> {
        let mut error_counts: HashMap<String, usize> = HashMap::new();

        for record in &self.records {
            for error_type in &record.error_types {
                *error_counts.entry(error_type.clone()).or_insert(0) += 1;
            }
        }

        let mut errors: Vec<(String, usize)> = error_counts.into_iter().collect();
        errors.sort_by(|a, b| b.1.cmp(&a.1));
        errors.truncate(10); // Top 10 most common errors
        errors
    }

    fn calculate_performance_trends(&self) -> Vec<PerformanceTrend> {
        let mut trends = Vec::new();

        if self.records.len() < 5 {
            return trends; // Not enough data for trend analysis
        }

        // Analyze success rate trend
        if let Some(success_trend) = self.calculate_metric_trend("success_rate", |r| if r.success { 1.0 } else { 0.0 }) {
            trends.push(success_trend);
        }

        // Analyze solve time trend
        if let Some(time_trend) = self.calculate_metric_trend("solve_time_ms", |r| r.solve_time_ms as f64) {
            trends.push(time_trend);
        }

        // Analyze complexity handling trend
        if let Some(complexity_trend) = self.calculate_metric_trend("complexity_score", |r| r.complexity_score) {
            trends.push(complexity_trend);
        }

        trends
    }

    fn calculate_metric_trend<F>(&self, metric_name: &str, value_extractor: F) -> Option<PerformanceTrend>
    where
        F: Fn(&PerformanceRecord) -> f64,
    {
        let n = self.records.len();
        if n < 5 {
            return None;
        }

        // Use sliding window approach - compare first and last halves
        let half = n / 2;
        let first_half: Vec<f64> = self.records[..half].iter().map(&value_extractor).collect();
        let second_half: Vec<f64> = self.records[n-half..].iter().map(&value_extractor).collect();

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let change_percentage = if first_avg != 0.0 {
            ((second_avg - first_avg) / first_avg) * 100.0
        } else {
            0.0
        };

        let (direction, confidence) = if change_percentage.abs() < 5.0 {
            ("stable".to_string(), 0.7)
        } else if change_percentage > 0.0 {
            ("improving".to_string(), 0.8)
        } else {
            ("declining".to_string(), 0.8)
        };

        Some(PerformanceTrend {
            metric: metric_name.to_string(),
            direction,
            change_percentage,
            confidence,
        })
    }

    fn generate_recommendations(&self, trends: &[PerformanceTrend], success_rate: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Success rate recommendations
        if success_rate < 0.5 {
            recommendations.push("Overall success rate is below 50%. Consider reviewing code generation prompts or improving test validation.".to_string());
        } else if success_rate > 0.8 {
            recommendations.push("Excellent success rate! Consider increasing challenge difficulty for more growth.".to_string());
        }

        // Trend-based recommendations
        for trend in trends {
            match trend.metric.as_str() {
                "success_rate" => {
                    if trend.direction == "declining" && trend.change_percentage.abs() > 10.0 {
                        recommendations.push("Success rate is declining. Review recent failed attempts for patterns.".to_string());
                    }
                }
                "solve_time_ms" => {
                    if trend.direction == "declining" && trend.change_percentage.abs() > 15.0 {
                        recommendations.push("Solve times are increasing. Consider optimizing LLM prompts or Docker execution.".to_string());
                    }
                }
                _ => {}
            }
        }

        // Error pattern recommendations
        let error_counts = self.calculate_most_common_errors();
        if let Some((most_common_error, count)) = error_counts.first() {
            if *count > self.records.len() / 3 {
                recommendations.push(format!("Most common error is '{}' ({}% of attempts). Focus optimization here.", most_common_error, (count * 100) / self.records.len()));
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Performance looks stable. Continue monitoring trends.".to_string());
        }

        recommendations
    }

    fn extract_error_types_from_result(&self, solve_result: &SolveResult) -> Vec<String> {
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

    fn calculate_complexity_score(&self, challenge: &Challenge) -> f64 {
        // Simple heuristic for challenge complexity
        let description_length = challenge.description.len() as f64;
        let test_count = challenge.tests.len() as f64;
        let setup_complexity = challenge.setup_commands.len() as f64;

        // Normalize and combine factors
        (description_length / 1000.0).min(1.0) * 0.4 +
        (test_count / 10.0).min(1.0) * 0.4 +
        (setup_complexity / 5.0).min(1.0) * 0.2
    }
}