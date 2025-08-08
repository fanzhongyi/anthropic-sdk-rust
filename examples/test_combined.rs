//! Debug Combined Features Demo
//!
//! This is a simplified version to debug the hanging issue in combined_features_demo

use anthropic_sdk::{
    Anthropic,
    types::{
        MessageCreateBuilder,
        SystemContentBlock,
        CacheControl,
        ThinkingConfig,
        ContentBlock,
    }
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🔧 Debug Combined Features Demo Starting...");

    let client = Anthropic::from_env()?;
    println!("✅ Client created successfully");

    // Test 1: Simple cached request without thinking
    println!("\n=== Test 1: Simple Cached Request ===");
    test_simple_cache(&client).await?;

    // Test 2: Simple thinking request without cache
    println!("\n=== Test 2: Simple Thinking Request ===");
    test_simple_thinking(&client).await?;

    // Test 3: Combined cache + thinking (non-streaming)
    println!("\n=== Test 3: Combined Cache + Thinking ===");
    test_combined_cache_thinking(&client).await?;

    // Test 4: Simple streaming (no cache, no thinking)
    println!("\n=== Test 4: Simple Streaming ===");
    test_simple_streaming(&client).await?;

    println!("\n🎉 All tests completed successfully!");
    Ok(())
}

async fn test_simple_cache(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("📤 Testing simple cache...");
    
    let message = client.messages().create(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 1024)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    "You are a helpful assistant. This is cached content.",
                    CacheControl::ephemeral()
                )
            ])
            .user("What is 2+2?")
            .build()
    ).await?;

    println!("📥 Response received!");
    println!("💾 Cache creation: {:?}", message.usage.cache_creation_input_tokens);
    println!("📊 Input tokens: {}, Output tokens: {}", 
             message.usage.input_tokens, message.usage.output_tokens);

    Ok(())
}

async fn test_simple_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("📤 Testing simple thinking...");
    
    let message = client.messages().create(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 2048)
            .thinking(1024)
            .user("What is 3+3? Think step by step.")
            .build()
    ).await?;

    println!("📥 Response received!");
    for (i, block) in message.content.iter().enumerate() {
        match block {
            ContentBlock::Thinking { thinking, .. } => {
                println!("🤔 Thinking block {}: {} chars", i, thinking.len());
            },
            ContentBlock::Text { text } => {
                println!("📝 Text block {i}: {text}");
            },
            _ => {
                println!("❓ Other block {i}: {block:?}");
            }
        }
    }

    Ok(())
}

async fn test_combined_cache_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("📤 Testing combined cache + thinking...");
    
    let message = client.messages().create(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 2048)
            .thinking_config(ThinkingConfig::enabled(1024))
            .system(vec![
                SystemContentBlock::text_with_cache(
                    "You are a math tutor. Always show your reasoning step by step.",
                    CacheControl::ephemeral()
                )
            ])
            .user("What is 5*7? Show your work.")
            .build()
    ).await?;

    println!("📥 Response received!");
    println!("💾 Cache creation: {:?}", message.usage.cache_creation_input_tokens);
    
    for (i, block) in message.content.iter().enumerate() {
        match block {
            ContentBlock::Thinking { thinking, .. } => {
                println!("🤔 Thinking block {}: {} chars", i, thinking.len());
            },
            ContentBlock::Text { text } => {
                println!("📝 Text block {i}: {text}");
            },
            _ => {}
        }
    }

    Ok(())
}

async fn test_simple_streaming(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("📤 Testing simple streaming...");
    
    use futures::StreamExt;
    
    let stream = client.messages().create_stream(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 1024)
            .user("Count from 1 to 5.")
            .stream(true)
            .build()
    ).await?;

    println!("📡 Processing stream...");
    
    let mut event_count = 0;
    let timeout_duration = std::time::Duration::from_secs(30);
    let start_time = std::time::Instant::now();
    
    tokio::pin!(stream);
    while let Some(result) = stream.next().await {
        if start_time.elapsed() > timeout_duration {
            println!("⏰ Stream timeout after 30 seconds");
            break;
        }
        
        match result {
            Ok(event) => {
                event_count += 1;
                use anthropic_sdk::types::streaming::MessageStreamEvent;
                match event {
                    MessageStreamEvent::MessageStart { .. } => {
                        println!("📨 Event {event_count}: message_start");
                    },
                    MessageStreamEvent::ContentBlockStart { .. } => {
                        println!("📨 Event {event_count}: content_block_start");
                    },
                    MessageStreamEvent::ContentBlockDelta { delta, .. } => {
                        use anthropic_sdk::types::streaming::ContentBlockDelta;
                        if let ContentBlockDelta::TextDelta { text } = delta {
                            print!("{text}");
                        }
                    },
                    MessageStreamEvent::MessageStop => {
                        println!("\n📨 Event {event_count}: message_stop - Stream complete!");
                        break;
                    },
                    _ => {
                        println!("📨 Event {event_count}: other");
                    }
                }
            },
            Err(e) => {
                println!("❌ Stream error: {e}");
                break;
            }
        }
    }

    println!("📊 Total events processed: {event_count}");
    Ok(())
}
