use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder, MessageStreamEvent, ContentBlockDelta};
use anthropic_sdk::types::ContentBlock;
use std::time::Duration;
use std::io::{self, Write};
use futures::StreamExt;
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
    println!("🚀 Testing Anthropic SDK Streaming with Custom Gateway...\n");

    // Get API key from environment
    let api_key = env::var("CUSTOM_BEARER_TOKEN")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("⚠️  No API key found. Please set CUSTOM_BEARER_TOKEN or ANTHROPIC_API_KEY");

    let base_url = env::var("CUSTOM_BASE_URL")
        .expect("⚠️  No base URL found. Please set CUSTOM_BASE_URL");

    // Custom configuration for custom gateway
    let config = ClientConfig::new(&api_key)
        .with_base_url(&base_url)
        .with_timeout(Duration::from_secs(30));

    let client = Anthropic::with_config(config)?;

    println!("✅ Client created with custom gateway configuration");
    println!("📡 Base URL: {base_url}");
    println!("🤖 Model: claude-3-haiku@20240307\n");

    // Test 1: Callback-based streaming
    println!("🧪 Test 1: Callback-based streaming...");

    if api_key != "dummy-key" {
        match client.messages()
            .create_with_builder("claude-3-haiku@20240307", 200)
            .user("Write a short haiku about programming")
            .temperature(0.8)
            .stream_send()
            .await
        {
            Ok(stream) => {
                println!("✅ Stream created successfully!");
                println!("📡 Streaming response...\n");

                let final_message = stream
                    .on_text(|delta, _snapshot| {
                        print!("{delta}");
                        io::stdout().flush().unwrap();
                    })
                    .on_error(|error| {
                        eprintln!("\n❌ Stream error: {error}");
                    })
                    .on_end(|| {
                        println!("\n✅ Stream completed!");
                    })
                    .final_message()
                    .await?;

                println!("📊 Usage: {} input, {} output tokens",
                    final_message.usage.input_tokens,
                    final_message.usage.output_tokens);
            }
            Err(e) => {
                println!("❌ Streaming test FAILED!");
                println!("🚨 Error: {e}");
            }
        }
    } else {
        println!("⚠️  Skipping streaming test with dummy API key");
    }

    // Test 2: Manual stream iteration
    println!("\n🧪 Test 2: Manual stream iteration...");

    if api_key != "dummy-key" {
        let params = MessageCreateBuilder::new("claude-3-haiku@20240307", 150)
            .user("Count from 1 to 3")
            .stream(true)
            .build();

        match client.messages().create_stream(params).await {
            Ok(mut stream) => {
                println!("✅ Stream created successfully!");
                println!("📡 Processing events manually...\n");

                let mut content = String::new();

                while let Some(event) = stream.next().await {
                    match event? {
                        MessageStreamEvent::MessageStart { message } => {
                            println!("📨 Message started: {}", message.id);
                        }
                        MessageStreamEvent::ContentBlockStart { index, .. } => {
                            println!("📝 Content block {index} started");
                        }
                        MessageStreamEvent::ContentBlockDelta { delta, .. } => {
                            if let ContentBlockDelta::TextDelta { text } = delta {
                                print!("{text}");
                                content.push_str(&text);
                                io::stdout().flush().unwrap();
                            }
                        }
                        MessageStreamEvent::MessageDelta { usage, .. } => {
                            println!("\n📊 Usage: {} output tokens", usage.output_tokens);
                        }
                        MessageStreamEvent::MessageStop => {
                            println!("\n✅ Stream completed!");
                            break;
                        }
                        _ => {}
                    }
                }

                println!("📜 Complete response: {content}");
            }
            Err(e) => {
                println!("❌ Manual streaming test FAILED!");
                println!("🚨 Error: {e}");
            }
        }
    } else {
        println!("⚠️  Skipping manual streaming test with dummy API key");
    }

    // Test 3: Regular message (for comparison)
    println!("\n🧪 Test 3: Regular message (non-streaming)...");
    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 100)
                .user("Hello! Just say 'SDK configured correctly'")
                .build()
        )
        .await;

    match response {
        Ok(msg) => {
            println!("✅ Regular message PASSED!");
            let text = extract_text_from_content(&msg.content);
            println!("📝 Response: {text}");
        }
        Err(e) => {
            println!("❌ Regular message FAILED!");
            println!("🚨 Error: {e}");

            if api_key == "dummy-key" {
                println!("💡 This is expected with dummy API key.");
            }
        }
    }

    println!("\n🎉 All streaming tests completed!");

    println!("\n💡 To test with real API calls:");
    println!("   export CUSTOM_BEARER_TOKEN='your-custom-gateway-key'");
    println!("   export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("   cargo run --example test_custom_streaming");

    Ok(())
}
