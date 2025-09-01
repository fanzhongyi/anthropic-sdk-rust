use anthropic_sdk::types::streaming::MessageStreamEvent;
use anthropic_sdk::{
    types::{ContentBlockDelta, MessageCreateBuilder},
    Anthropic,
};
use futures::StreamExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🧪 Basic Anthropic SDK Streaming Demo");
    println!("{}", "=".repeat(60));

    // Create client from environment (ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_METHOD, etc.)
    let client = Anthropic::from_env()?;
    println!("✅ Client initialized successfully\n");

    println!("\n0. Testing basic streaming ...\n");
    let stream = client
        .messages()
        .stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 64)
                .user("Please count from 1 to 8 and say hello!")
                .build(),
        )
        .await?;

    let basic_response = stream
        .on_text(|delta, _| {
            print!("{delta}");
        })
        .final_message()
        .await?;
    println!("\nBasic Streaming Response: {basic_response:?}\n");

    println!("\n1. Streaming Response (only get final_message)\n");
    let stream = client
        .messages()
        .stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 64)
                .user("Please count from 1 to 8 and say hello!")
                .build(),
        )
        .await?;
    let final_message = stream.final_message().await?;
    println!("Claude: {final_message:?}");

    println!("\n2. Streaming Response (next) \n");
    let stream = client
        .messages()
        .stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 64)
                .user("Please count from 1 to 8 and say hello!")
                .build(),
        )
        .await?;
    tokio::pin!(stream);
    let mut count = 0;
    let max_iterations = 50; // Limit to prevent infinite loop

    while let Some(event) = stream.next().await {
        println!("Event {}: {event:?}", count + 1);
        count += 1;
        if count >= max_iterations {
            println!("Reached max iterations, breaking...");
            break;
        }
        if event? == MessageStreamEvent::MessageStop {
            println!("Stream ended manually as we received a MessageStop event");
        }
    }
    println!("Stream ended after {count} events");

    println!("\n3. Streaming Response (for_each pattern)\n");
    let stream = client
        .messages()
        .stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 64)
                .user("Please count from 1 to 8 and say hello!")
                .build(),
        )
        .await?;
    let mut count = 0;
    stream
        .for_each(|event| {
            count += 1;
            async move {
                match event {
                    Ok(MessageStreamEvent::ContentBlockDelta {
                        delta: ContentBlockDelta::TextDelta { text: _ },
                        ..
                    }) => {
                        // print!("{text} || ");
                    }
                    Ok(MessageStreamEvent::MessageStop) => {
                        println!("\n[MessageStop received - stream will auto-terminate]");
                    }
                    Ok(MessageStreamEvent::Text { text }) => {
                        println!("Text: {text}");
                    }
                    Ok(event) => {
                        println!("Unhandled event: {event:?}");
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                    }
                }
            }
        })
        .await;
    println!("For_each pattern completed automatically after {count} events");

    Ok(())
}
