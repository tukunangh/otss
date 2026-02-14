//! Test binary for otss-data
//!
//! This executable verifies that the data crate compiles correctly.

#[tokio::main]
async fn main() {
    println!("OTSS Data - Test Binary");
    println!("======================");

    otss_data::init();
    
    let sources = otss_data::available_sources();
    println!("Available data sources:");
    for source in sources {
        println!("  - {:?}", source);
    }

    println!("\nData crate test completed successfully.");
}
