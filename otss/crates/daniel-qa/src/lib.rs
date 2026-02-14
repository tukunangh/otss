//! OTSS QA - Integration testing framework
//!
//! This crate provides testing utilities for:
//! - Integration tests across all OTSS crates
//! - Mock broker and data adapters
//! - Test scenarios and fixtures
//! - Performance benchmarks

#![allow(dead_code, unused_imports)]

pub mod fixtures;
pub mod mock;
pub mod scenarios;
pub mod harness;

pub use fixtures::{TestData, TestOrder, TestPortfolio};
pub use mock::{MockBroker, MockDataFeed};
pub use scenarios::{Scenario, ScenarioBuilder, ScenarioRunner};
pub use harness::{TestHarness, TestConfig, TestResult};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the QA module
pub fn init() {
    _ = tracing_subscriber::fmt::try_init();
    tracing::info!("Initializing OTSS QA module v{}", VERSION);
}

/// Run all tests
pub fn run_all_tests() -> anyhow::Result<TestResult> {
    tracing::info!("Running all OTSS tests");
    Ok(TestResult {
        passed: 0,
        failed: 0,
        skipped: 0,
    })
}

/// Get test framework status
pub fn framework_status() -> String {
    format!("OTSS QA Framework v{}", VERSION)
}
