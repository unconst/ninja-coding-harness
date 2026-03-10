// 🥷 Ninja Coding Harness - Library Interface
//
// This library provides a self-improving coding challenge solver
// with continuous feedback loops and synthetic challenge generation.

pub mod challenge;
pub mod config;
pub mod error;
pub mod llm;
pub mod executor_simple;
pub mod challenge_solver;
pub mod challenge_generation;

pub use challenge::*;
pub use config::*;
pub use error::*;
pub use llm::*;
pub use executor_simple::*;
pub use challenge_solver::*;
pub use challenge_generation::*;