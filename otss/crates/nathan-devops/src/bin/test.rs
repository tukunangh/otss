//! Test binary for otss-devops
//!
//! This executable verifies that the devops crate compiles correctly.

use tokio;

#[tokio::main]
async fn main() {
    println!("OTSS DevOps - Test Binary");
    println!("=========================");

    otss_devops::init();
    println!("{}" , otss_devops::get_pipeline_status());

    let status = otss_devops::health_check();
    println!("Health Status: {:?}", status);

    println!("\nDevOps crate test completed successfully.");
}
