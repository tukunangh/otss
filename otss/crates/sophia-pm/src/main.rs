//! OTSS PM - Main entry point
//!
//! This binary provides the project management CLI and orchestration.

use tokio;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("OTSS Project Manager");
    println!("===================");
    println!("{}", otss_pm::status());

    // Initialize PM module
    otss_pm::init();

    // Create and start orchestrator
    let config = otss_pm::SystemConfig::default();
    let mut orchestrator = otss_pm::Orchestrator::new(config);
    
    println!("\nSystem config:");
    println!("  Name: {}", orchestrator.config().name);
    println!("  Environment: {}", orchestrator.config().environment);
    println!("  Services: {} active", orchestrator.enabled_services().len());

    // Start system
    if let Err(e) = orchestrator.start().await {
        eprintln!("Failed to start system: {}", e);
        std::process::exit(1);
    }

    println!("\nSystem started successfully");
    println!("Current state: {:?}", orchestrator.state());

    // In a real system, we'd wait for shutdown signal here
    println!("\nPM main executable running...");
    println!("Press Ctrl+C to shutdown");

    // Wait awhile then graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Stop system
    if let Err(e) = orchestrator.stop().await {
        eprintln!("Failed to stop system: {}", e);
    }

    println!("\nShutdown complete.");
}
