//! Simple streaming tool test to isolate the issue
//!
//! This is a minimal test to debug the streaming tool call problem

use anthropic_sdk::Tool;
use anthropic_sdk::{
    types::{ContentBlock, ContentBlockDelta, MessageCreateBuilder, MessageStreamEvent},
    Anthropic, ToolFunction, ToolRegistry, ToolResult,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;

pub struct WeatherTool;

#[async_trait]
impl ToolFunction for WeatherTool {
    async fn execute(
        &self,
        input: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let weather_data = json!({
            "location": input["location"].as_str().unwrap_or("Unknown"),
            "temperature": "22°C",
            "condition": "Sunny",
            "humidity": "45%",
            "wind": "5 mph"
        });
        Ok(ToolResult::success_json("weather", weather_data))
    }
}

impl WeatherTool {
    pub fn definition() -> Tool {
        Tool::new("get_weather", "Get current weather for a location")
            .parameter("location", "string", "City name")
            .required("location")
            .build()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🧪 Simple Streaming Tool Test");
    println!("{}", "=".repeat(40));

    // Create client from environment
    let client = Anthropic::from_env()?;
    println!("✅ Client initialized successfully\n");

    // Setup tool registry
    let mut registry = ToolRegistry::new();
    registry.register(
        "get_weather",
        WeatherTool::definition(),
        Box::new(WeatherTool),
    )?;
    let tool_decls = registry.get_tool_definitions();

    println!("🔧 Testing streaming with tools...\n");

    // Create streaming request with tools
    let stream_params = MessageCreateBuilder::new("claude-3-haiku@20240307", 256)
        .user("What's the weather like in Paris?")
        .tools(tool_decls)
        .stream(true)
        .build();

    let tool_stream = client.messages().create_stream(stream_params).await?;

    // Process the stream and collect the final message
    let stream_response = tool_stream
        .on_text(|delta, _| {
            print!("{delta}");
        })
        .on_stream_event(|event, current_message| {
            match event {
                MessageStreamEvent::MessageStart { .. } => {
                    println!("📝 Message started");
                }
                MessageStreamEvent::ContentBlockStart { content_block, index } => {
                    match content_block {
                        ContentBlock::ToolUse { id, name, input } => {
                            println!("🔧 Tool use started: {name} ({id}) at index {index}");
                            println!("   Input so far: {input}");
                        }
                        ContentBlock::Text { text } => {
                            println!("📝 Text block started at index {index}: '{text}'");
                        }
                        _ => {
                            println!("🔧 Other content block started at index {index}: {content_block:?}");
                        }
                    }
                }
                MessageStreamEvent::ContentBlockDelta { delta, index } => {
                    match delta {
                        ContentBlockDelta::InputJsonDelta { partial_json } => {
                            println!("🔧 Tool input update at index {index}: {partial_json}");
                        }
                        ContentBlockDelta::TextDelta { text: _ } => {
                            // Text delta is handled by on_text callback
                        }
                        _ => {
                            println!("🔧 Other delta at index {index}: {delta:?}");
                        }
                    }
                }
                MessageStreamEvent::ContentBlockStop { index, content_block } => {
                    println!("🔧 Content block stopped at index {index}");
                    // Print the final state of this content block
                    let final_content_block = if let Some(cb) = content_block {
                        Some(cb)
                    } else {
                        current_message.content.get(*index)
                    };

                    if let Some(content_block) = final_content_block {
                        match content_block {
                            ContentBlock::ToolUse { id, name, input } => {
                                println!("   Final tool use: {name} ({id}) with input: {input}");
                            }
                            ContentBlock::Text { text } => {
                                println!("   Final text: '{text}'");
                            }
                            _ => {
                                println!("   Final content: {content_block:?}");
                            }
                        }
                    }
                }
                MessageStreamEvent::MessageStop => {
                    println!("\n🏁 Message completed");
                }
                _ => {
                    println!("🔧 Other event: {event:?}");
                }
            }
        })
        .final_message()
        .await?;

    println!("\n✅ Stream completed!");
    println!(
        "Final message has {} content blocks",
        stream_response.content.len()
    );

    // Print all content blocks
    for (i, block) in stream_response.content.iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                println!("Block {i}: Text: '{text}'");
            }
            ContentBlock::ToolUse { id, name, input } => {
                println!("Block {i}: ToolUse: {name} ({id}) with input: {input}");
            }
            _ => {
                println!("Block {i}: Other: {block:?}");
            }
        }
    }

    // Check if we have any tool calls
    let tool_calls: Vec<_> = stream_response
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::ToolUse { id, name, input } = block {
                Some((id.clone(), name.clone(), input.clone()))
            } else {
                None
            }
        })
        .collect();

    if tool_calls.is_empty() {
        println!("❌ No tool calls found in streaming response!");
        return Ok(());
    }

    println!("✅ Found {} tool call(s)", tool_calls.len());

    // Execute the tools
    for (id, name, input) in tool_calls {
        println!("🔧 Executing tool '{name}' with input: {input}");

        let tool_use = anthropic_sdk::ToolUse { id, name, input };
        match registry.execute(&tool_use).await {
            Ok(result) => {
                println!("✅ Tool executed successfully: {result:?}");
            }
            Err(err) => {
                println!("❌ Tool execution failed: {err}");
            }
        }
    }

    println!("\n🎉 Test completed successfully!");
    Ok(())
}
