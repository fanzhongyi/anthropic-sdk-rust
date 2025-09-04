//! Minimal example to demonstrate aborting a streaming request mid-flight.
//!
//! Run with:
//!
//! ```bash
//! export ANTHROPIC_API_KEY="sk-..."
//! # optionally: export ANTHROPIC_BASE_URL="http://your-gateway" and ANTHROPIC_AUTH_METHOD
//! cargo run --example streaming_abort_min
//! ```

use anthropic_sdk::types::messages::MessageCreateBuilder;
use anthropic_sdk::types::AnthropicError;
use anthropic_sdk::types::ContentBlocksExt;
use anthropic_sdk::Anthropic;
use std::error::Error;
use std::time::Duration;

const MODEL: &str = "claude-sonnet-4@20250514";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🧪 Streaming abort minimal example");

    // Initialize client from environment
    let client = Anthropic::from_env()?;

    // Build a simple streaming message that produces some text
    let params = MessageCreateBuilder::new(MODEL, 256)
        .user("Please stream a short poem line by line. Make it long enough to see streaming.")
        .stream(true)
        .build();

    let stream = client.messages().create_stream(params).await?;

    // Capture an abort handle before consuming the stream with callbacks
    let abort_handle = stream
        .abort_handle()
        .expect("abort handle should be available for streaming requests");

    // Schedule an abort shortly after starting to stream
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        println!("\n⛔ Aborting stream now (expected to return UserAbort)...");
        abort_handle.abort();
    });

    // Consume some text and then await final result; should return UserAbort
    let result = stream
        .on_text(|delta, _| {
            print!("{}", delta);
        })
        .final_message()
        .await;

    match result {
        Err(AnthropicError::UserAbort) => {
            println!("\n✅ Abort worked: received UserAbort error");
        }
        Ok(msg) => {
            println!(
                "\n⚠️ Stream completed normally (unexpected for this demo). Final text: {}",
                msg.content.get_text()
            );
        }
        Err(other) => {
            println!("\n❌ Unexpected error: {}", other);
        }
    }

    Ok(())
}
