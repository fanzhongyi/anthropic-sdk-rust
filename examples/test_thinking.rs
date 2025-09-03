//! Simple Thinking Test
//!
//! This example tests basic thinking functionality with minimal complexity
//! to debug streaming issues.

use anthropic_sdk::{
    types::{ContentBlock, MessageCreateBuilder},
    Anthropic,
};
use futures::StreamExt;
use std::error::Error;
use std::io::{self, Write};

pub const MODEL: &str = "claude-sonnet-4@20250514";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    println!("🔧 Simple Thinking Test Starting...");

    let client = Anthropic::from_env()?;
    println!("✅ Client created successfully");

    println!("\n=== Test 1: Non-streaming thinking ===");
    test_non_streaming_thinking(&client).await?;

    println!("\n=== Test 2: Streaming thinking ===");
    test_streaming_thinking(&client).await?;

    println!("\n🎉 All tests completed successfully!");
    Ok(())
}

/// Test basic non-streaming thinking
async fn test_non_streaming_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("📤 Sending non-streaming thinking request...");

    let message = client
        .messages()
        .create(
            MessageCreateBuilder::new(MODEL, 2048)
                .thinking(1024) // Minimum thinking budget required
                .user("What is 2+2? Think step by step.")
                .build(),
        )
        .await?;

    println!("📥 Response received!");

    for (i, block) in message.content.iter().enumerate() {
        match block {
            ContentBlock::Thinking {
                thinking,
                signature,
            } => {
                println!("🤔 Thinking block {i}: {thinking}");
                println!("🔐 Signature: {signature}");
            }
            ContentBlock::Text { text } => {
                println!("📝 Text block {i}: {text}");
            }
            _ => {
                println!("❓ Other block {i}: {block:?}");
            }
        }
    }

    println!(
        "📊 Usage - Input: {}, Output: {}",
        message.usage.input_tokens, message.usage.output_tokens
    );

    Ok(())
}

/// Test basic streaming thinking
async fn test_streaming_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("📤 Sending streaming thinking request...");

    let stream = client
        .messages()
        .create_stream(
            MessageCreateBuilder::new(MODEL, 2048)
                .thinking(1024) // Minimum thinking budget required
                .user("What is 3+3? Think step by step.")
                .stream(true)
                .build(),
        )
        .await?;

    println!("📡 Stream created, processing events...");

    let mut event_count = 0;
    let mut thinking_events = 0;
    let mut text_events = 0;
    let mut signature_events = 0;

    // Process the stream with timeout protection
    tokio::pin!(stream);

    // Add a manual timeout
    let timeout_duration = std::time::Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    while let Some(result) = stream.next().await {
        if start_time.elapsed() > timeout_duration {
            println!("⏰ Manual timeout after 30 seconds");
            break;
        }

        match result {
            Ok(event) => {
                event_count += 1;
                print!("📨 Event {event_count}: ");
                io::stdout().flush().unwrap();

                use anthropic_sdk::types::streaming::MessageStreamEvent;
                match event {
                    MessageStreamEvent::MessageStart { .. } => {
                        println!("message_start");
                    }
                    MessageStreamEvent::ContentBlockStart {
                        content_block,
                        index,
                    } => {
                        println!(
                            "content_block_start[{}] - {:?}",
                            index,
                            match &content_block {
                                ContentBlock::Thinking { .. } => "thinking",
                                ContentBlock::Text { .. } => "text",
                                _ => "other",
                            }
                        );
                    }
                    MessageStreamEvent::ContentBlockDelta { delta, index } => {
                        use anthropic_sdk::types::streaming::ContentBlockDelta;
                        match delta {
                            ContentBlockDelta::ThinkingDelta { thinking } => {
                                println!("thinking_delta[{index}]: {thinking}");
                            }
                            ContentBlockDelta::TextDelta { text } => {
                                println!("text_delta[{index}]: {text}");
                            }
                            ContentBlockDelta::SignatureDelta { signature } => {
                                println!("signature_delta[{index}]: {signature}");
                            }
                            _ => {
                                println!("other_delta[{index}]: {delta:?}");
                            }
                        }
                    }
                    MessageStreamEvent::ContentBlockStop { index, .. } => {
                        println!("content_block_stop[{index}]");
                    }
                    MessageStreamEvent::Thinking { thinking } => {
                        thinking_events += 1;
                        println!("thinking event: {}", thinking);
                    }
                    MessageStreamEvent::Signature { signature } => {
                        signature_events += 1;
                        println!("signature event: {signature}");
                    }
                    MessageStreamEvent::Text { text } => {
                        text_events += 1;
                        println!("text event: {}", text);
                    }
                    MessageStreamEvent::InputJson { partial_json } => {
                        println!("input_json: {}", partial_json);
                    }
                    MessageStreamEvent::MessageDelta { .. } => {
                        println!("message_delta");
                    }
                    MessageStreamEvent::MessageStop => {
                        println!("message_stop - Stream complete! 🎉");
                        break;
                    }
                }
            }
            Err(e) => {
                println!("❌ Stream error: {e}");
                break;
            }
        }
    }

    println!("\n📈 Stream Statistics:");
    println!("  Total events: {event_count}");
    println!("  Thinking events: {thinking_events}");
    println!("  Text events: {text_events}");
    println!("  Signature events: {signature_events}");

    Ok(())
}
