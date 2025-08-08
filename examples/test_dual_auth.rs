use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder};
use anthropic_sdk::types::ContentBlock;
use std::time::Duration;
use std::env;

// Helper function to extract text content from response
fn extract_text_from_content(content: &[ContentBlock]) -> String {
    content.iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Dual Authentication Test (Anthropic + Custom Gateway)");
    println!("========================================================\n");

    // Test 1: Standard Anthropic API
    println!("🧪 Test 1: Standard Anthropic API");
    test_anthropic_auth().await?;

    // Test 2: Custom Gateway Bearer token authentication
    println!("\n🧪 Test 2: Custom Gateway (Bearer token)");
    test_custom_auth().await?;

    // Test 3: Compare both approaches
    println!("\n🧪 Test 3: Side-by-side comparison");
    compare_auth_methods().await?;

    print_configuration_guide().await?;

    Ok(())
}

async fn test_anthropic_auth() -> Result<(), Box<dyn std::error::Error>> {
    match env::var("ANTHROPIC_API_KEY") {
        Ok(_api_key) => {
            println!("   📡 URL: https://api.anthropic.com");
            println!("   🔑 Using x-api-key authentication");

            let client = Anthropic::from_env()?;

            let response = client.messages()
                .create(
                    MessageCreateBuilder::new("claude-3-haiku@20240307", 50)
                        .user("Hello from standard Anthropic API!")
                        .build()
                )
                .await?;

            println!("   ✅ Standard Anthropic API works!");
            println!("   📝 Response: {}", extract_text_from_content(&response.content));
        }
        Err(_) => {
            println!("   ⚠️  ANTHROPIC_API_KEY not set, skipping standard API test");
        }
    }

    Ok(())
}

async fn test_custom_auth() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CUSTOM_BEARER_TOKEN")
        .expect("Need CUSTOM_BEARER_TOKEN for custom gateway test");

    let base_url = env::var("CUSTOM_BASE_URL")
        .expect("Need CUSTOM_BASE_URL for custom gateway test");

    println!("   📡 URL: {base_url}");
    println!("   🔑 Using Bearer token authentication");

    // Method 1: Using with_base_url and standard bearer auth
    let config = ClientConfig::new(api_key.clone())
        .with_base_url(&base_url)
        .with_timeout(Duration::from_secs(30));

    let client = Anthropic::with_config(config)?;

    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 50)
                .user("Hello from Custom Gateway!")
                .build()
        )
        .await?;

    println!("   ✅ Custom Gateway (Method 1) works!");
    println!("   📝 Response: {}", extract_text_from_content(&response.content));

    // Method 2: Using for_custom_gateway convenience method (if available)
    let custom_config = ClientConfig::new(api_key)
        .with_base_url(&base_url);

    let custom_client = Anthropic::with_config(custom_config)?;

    let response2 = custom_client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 50)
                .user("Hello from Custom convenience config!")
                .build()
        )
        .await?;

    println!("   ✅ Custom Gateway (Method 2) works!");
    println!("   📝 Response: {}", extract_text_from_content(&response2.content));

    Ok(())
}

async fn compare_auth_methods() -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔍 Comparing authentication methods:");

    // Anthropic Standard
    if env::var("ANTHROPIC_API_KEY").is_ok() {
        println!("   📊 Standard Anthropic:");
        println!("      • URL: https://api.anthropic.com");
        println!("      • Auth: x-api-key header");
        println!("      • Usage: Anthropic::from_env()");
    }

    // Custom Gateway
    if env::var("CUSTOM_BEARER_TOKEN").is_ok() && env::var("CUSTOM_BASE_URL").is_ok() {
        let base_url = env::var("CUSTOM_BASE_URL").unwrap();
        println!("   📊 Custom Gateway:");
        println!("      • URL: {base_url}");
        println!("      • Auth: Bearer token");
        println!("      • Usage: ClientConfig::new(token).with_base_url(url)");
    }

    Ok(())
}

async fn print_configuration_guide() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Configuration Guide:");
    println!("=======================");

    println!("\n🔸 Standard Anthropic API:");
    println!("export ANTHROPIC_API_KEY='your-anthropic-api-key'");
    println!("    let client = Anthropic::from_env();");

    println!("\n🔸 Custom Gateway (Bearer Token):");
    println!("export CUSTOM_BEARER_TOKEN='your-bearer-token'");
    println!("export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("    let config = ClientConfig::new(token).with_base_url(url);");

    println!("\n🔸 Custom Gateway (Environment Variables):");
    println!("export ANTHROPIC_API_KEY='your-bearer-token'");
    println!("export ANTHROPIC_BASE_URL='https://your-gateway.example.com/v1/anthropic'");

    println!("\n🚀 Quick Setup:");
    println!("===============");

    println!("\n1. For standard Anthropic API:");
    println!("   export ANTHROPIC_API_KEY='your-anthropic-key'");
    println!("   cargo run --example test_dual_auth");

    println!("\n2. For custom gateway:");
    println!("   export CUSTOM_BEARER_TOKEN='your-real-bearer-token'");
    println!("   export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("   cargo run --example test_dual_auth");

    println!("\n📚 Available Examples:");
    println!("• cargo run --example custom_gateway_production  # With real token");
    println!("• cargo run --example custom_gateway_demo        # This demo");
    println!("• cargo run --example test_custom_gateway        # Basic test");
    println!("• cargo run --example test_custom_streaming      # Test streaming");

    println!("\n🎯 Both authentication methods are fully supported! 🚀");

    Ok(())
}
