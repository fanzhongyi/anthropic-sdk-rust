//! Debug Streaming Demo
//!
//! Simple test to debug streaming issues

use anthropic_sdk::{
    types::{ContentBlock, MessageCreateBuilder},
    Anthropic,
};
use futures::StreamExt;
use std::error::Error;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create client from environment variables
    let client = Anthropic::from_env()?;

    println!("🔄 Creating stream...");
    let stream = client
        .messages()
        .create_stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 4096)
                .thinking(1024) // thinking tokens must be less than max_tokens
                .system("You are a helpful assistant.")
                .user("What is 2+2? Think step by step.")
                .stream(true)
                .build(),
        )
        .await?;

    println!("✅ Stream created successfully");
    println!("🔄 Starting stream processing...");
    io::stdout().flush().unwrap();

    // Process the stream
    tokio::pin!(stream);
    let mut event_count = 0;

    while let Some(result) = stream.next().await {
        event_count += 1;
        println!("📦 Event #{event_count}: {result:?}");
        io::stdout().flush().unwrap();

        match result {
            Ok(event) => {
                use anthropic_sdk::types::streaming::MessageStreamEvent;
                println!(
                    "✅ Processing event type: {:?}",
                    std::mem::discriminant(&event)
                );
                io::stdout().flush().unwrap();

                match event {
                    MessageStreamEvent::ContentBlockStart {
                        content_block,
                        index,
                    } => {
                        println!(
                            "🎬 ContentBlockStart - Index: {}, Type: {:?}",
                            index,
                            std::mem::discriminant(&content_block)
                        );
                        match content_block {
                            ContentBlock::Thinking { .. } => {
                                println!("🤔 Thinking block started");
                            }
                            ContentBlock::Text { .. } => {
                                println!("📝 Text block started");
                            }
                            _ => {
                                println!("❓ Unknown block type: {content_block:?}");
                            }
                        }
                    }
                    MessageStreamEvent::ContentBlockDelta { delta, index } => {
                        use anthropic_sdk::types::streaming::ContentBlockDelta;
                        println!(
                            "📝 ContentBlockDelta - Index: {}, Delta type: {:?}",
                            index,
                            std::mem::discriminant(&delta)
                        );
                        match delta {
                            ContentBlockDelta::ThinkingDelta { thinking } => {
                                println!("🧠 Thinking delta: {} chars", thinking.len());
                                print!(".");
                            }
                            ContentBlockDelta::TextDelta { text } => {
                                println!("💬 Text delta: {} chars", text.len());
                                print!("{text}");
                            }
                            ContentBlockDelta::SignatureDelta { signature } => {
                                println!("✍️ Signature delta: {} chars", signature.len());
                                print!("📋");
                            }
                            ContentBlockDelta::InputJsonDelta { partial_json } => {
                                println!("🔧 Input JSON delta: {} chars", partial_json.len());
                                print!("🛠️");
                            }
                            ContentBlockDelta::CitationsDelta { citation } => {
                                println!("📚 Citations delta: {citation:?}");
                                print!("📖");
                            }
                        }
                    }
                    MessageStreamEvent::MessageStop => {
                        println!("\n🛑 MessageStop received");
                        io::stdout().flush().unwrap();
                    }
                    _ => {
                        println!("❓ Other event: {event:?}");
                    }
                }
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                println!("❌ Stream error: {e}");
                io::stdout().flush().unwrap();
                break;
            }
        }
    }

    println!("\n🏁 Stream processing complete. Total events: {event_count}");
    Ok(())
}
