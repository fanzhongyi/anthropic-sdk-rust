//! Demonstrate unified with_response() style APIs for both non-streaming and streaming calls.
//!
//! Run with:
//!   export ANTHROPIC_API_KEY="sk-..."
//!   cargo run --example with_response_demo

use anthropic_sdk::types::messages::MessageCreateBuilder;
use anthropic_sdk::types::ContentBlocksExt;
use anthropic_sdk::Anthropic;
use std::error::Error;

const MODEL: &str = "claude-sonnet-4@20250514";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Anthropic::from_env()?;

    println!("\n== Non-streaming with_response ==");
    let nonstream_params = MessageCreateBuilder::new(MODEL, 128)
        .user("Say hello and include today's date in one sentence.")
        .build();

    let non_env = client
        .messages()
        .create_with_response(nonstream_params)
        .await?;

    println!("status: {}", non_env.response.status);
    println!(
        "request_id (meta): {:?}",
        non_env.response.request_id.as_ref().map(|r| r.as_str())
    );
    println!("headers: {:?}", non_env.response.headers);
    println!("text: {}", non_env.data.content.get_text());
    // Message.request_id should match response meta when the server returned it
    if let (Some(mid), Some(rid)) = (
        non_env.data.request_id.as_ref().map(|r| r.as_str()),
        non_env.response.request_id.as_ref().map(|r| r.as_str()),
    ) {
        println!("message.request_id == meta.request_id ? {}", mid == rid);
    }

    println!("\n== Streaming with_response ==");
    let stream_params = MessageCreateBuilder::new(MODEL, 256)
        .user("Please stream a short poem about rivers, line by line.")
        .build();

    let stream_env = client
        .messages()
        .create_stream_with_response(stream_params)
        .await?;

    println!("status: {}", stream_env.response.status);
    println!(
        "request_id (meta): {:?}",
        stream_env.response.request_id.as_ref().map(|r| r.as_str())
    );

    let stream = stream_env.data.on_text(|delta, _| {
        print!("{}", delta);
    });

    let final_msg = stream.final_message().await?;
    println!("\n[final]\n{}", final_msg.content.get_text());

    if let (Some(mid), Some(rid)) = (
        final_msg.request_id.as_ref().map(|r| r.as_str()),
        stream_env.response.request_id.as_ref().map(|r| r.as_str()),
    ) {
        println!(
            "stream.final.message.request_id == meta.request_id ? {}",
            mid == rid
        );
    }

    Ok(())
}
