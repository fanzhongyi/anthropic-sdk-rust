use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder};
use anthropic_sdk::types::ContentBlock;
use dotenvy::dotenv;
use std::time::Duration;

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

/// Get model name from environment or use default
fn get_model_name() -> String {
    dotenv().ok();
    std::env::var("CUSTOM_MODEL_NAME")
        .unwrap_or_else(|_| "claude-3-7-sonnet@20250219".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    println!("🚀 Custom Gateway Production Configuration");
    println!("==========================================");

    // Get credentials from environment variables
    let bearer_token = std::env::var("CUSTOM_BEARER_TOKEN")
        .or_else(|_| std::env::var("CUSTOM_API_KEY"))
        .map_err(|_| "Please set CUSTOM_BEARER_TOKEN or CUSTOM_API_KEY environment variable")?;

    // Get base URL from environment (required for security)
    let base_url = std::env::var("CUSTOM_BASE_URL")
        .map_err(|_| "Please set CUSTOM_BASE_URL environment variable for your custom gateway")?;

    println!("🧪 Method 1: Using for_custom_gateway() convenience method");

    // Method 1: Using the convenience method
    let config1 = ClientConfig::new(bearer_token.clone())
        .for_custom_gateway(&base_url)
        .with_timeout(Duration::from_secs(30));

    let client1 = Anthropic::with_config(config1)?;

    // Test the client
    let response = client1.messages()
        .create(
            MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 100)
                .user("Hello! Please confirm the custom gateway integration is working.")
                .build()
        )
        .await?;

    if let Some(text) = response.content.first() {
        println!("✅ Response received: {text:?}");
    }

    println!("\n🎉 Your custom gateway is fully integrated with the Rust SDK!");

    println!("\nConfiguration examples:");
    println!("🔸 Environment variables (.env file):");
    println!("   CUSTOM_BEARER_TOKEN='your-token'");
    println!("   CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("   ANTHROPIC_AUTH_METHOD='bearer'");

    Ok(())
}

async fn test_client(client: &Anthropic, method_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 100)
                .user(format!("Hello! Confirm {method_name} is working correctly."))
                .build()
        )
        .await;

    match response {
        Ok(msg) => {
            println!("   ✅ SUCCESS!");
            let text = extract_text_from_content(&msg.content);
            println!("   📝 Response: {text}");
            println!("   📊 Usage: {} input, {} output tokens",
                msg.usage.input_tokens, msg.usage.output_tokens);
        }
        Err(e) => {
            println!("   ❌ FAILED: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
