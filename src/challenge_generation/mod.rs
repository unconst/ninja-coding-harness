// 🥷 Ninja Challenge Generation Module
//
// This module integrates SWE-Forge methodology for generating synthetic coding challenges
// from real GitHub PRs. It provides the foundation for the continuous improvement loop.

pub mod swe_forge_adapter;
pub mod challenge_generator;
pub mod performance_tracker;

pub use challenge_generator::*;
pub use swe_forge_adapter::*;
pub use performance_tracker::*;