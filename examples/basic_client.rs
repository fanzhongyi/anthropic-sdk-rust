use anthropic_sdk::{Anthropic, ClientConfig, LogLevel, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Anthropic Rust SDK - Phase 1 Foundation Demo");
    println!("{}", "=".repeat(50));

    // Example 1: Create client with API key
    println!("\n📝 Creating client with API key...");
    let client = Anthropic::new("demo-api-key")?;
    println!("✅ Client created successfully!");
    println!("   Base URL: {}", client.config().base_url);
    println!("   Timeout: {:?}", client.config().timeout);

    // Example 2: Create client with custom configuration
    println!("\n⚙️  Creating client with custom config...");
    let config = ClientConfig::new("demo-api-key")
        .with_timeout(Duration::from_secs(30))
        .with_max_retries(3)
        .with_log_level(LogLevel::Info)
        .with_base_url("https://api.anthropic.com");
    
    let custom_client = Anthropic::with_config(config)?;
    println!("✅ Custom client created!");
    println!("   Timeout: {:?}", custom_client.config().timeout);
    println!("   Max retries: {}", custom_client.config().max_retries);

    // Example 3: Test connection (validation)
    println!("\n🔗 Testing client connection...");
    match client.test_connection().await {
        Ok(_) => println!("✅ Connection test passed!"),
        Err(e) => println!("❌ Connection test failed: {e}"),
    }

    // Example 4: Demonstrate error handling
    println!("\n⚠️  Demonstrating error handling...");
    match Anthropic::new("") {
        Ok(_) => println!("❌ This shouldn't happen"),
        Err(e) => println!("✅ Caught expected error: {e}"),
    }

    println!("\n🎯 Phase 1 Foundation Complete!");
    println!("Ready for Phase 2: Messages API implementation");
    
    Ok(())
} 
