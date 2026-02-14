//! Test binary for otss-integration
//!
//! This executable verifies that the integration crate compiles correctly.

#[tokio::main]
async fn main() {
    println!("OTSS Integration - Test Binary");
    println!("==============================");

    otss_integration::init();

    let adapters = otss_integration::available_adapters();
    println!("Available broker adapters:");
    for adapter in adapters {
        println!("  - {}", adapter);
    }

    let _config = otss_integration::default_connection_config();
    println!("✓ Connection config created");

    println!("\nIntegration crate test completed successfully.");
}
