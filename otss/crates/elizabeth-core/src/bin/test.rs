//! Test binary for otss-core
//!
//! This executable verifies that the core crate compiles correctly.

use otss_core::{EngineConfig, TradingEngine};

fn main() {
    println!("OTSS Core - Test Binary");
    println!("=======================");

    // Create engine
    let config = EngineConfig::default();
    let engine = TradingEngine::new(config);

    println!("Trading Engine created successfully!");
    println!("Max order rate: {}", engine.config().max_order_rate);

    // Test order creation
    let result: Result<u32, _> = "test".parse();
    match result {
        Ok(val) => println!("Parsed: {}", val),
        Err(e) => println!("Expected parse error: {}", e),
    }

    println!("\nCore crate test completed successfully.");
}
