//! Streaming Multi-turn Tool Usage Demo
//!
//! This example demonstrates how to use Claude with tools in a streaming multi-turn conversation.
//! It shows how to handle tool calls in real-time as they're being generated, execute them,
//! and continue the conversation with the results.
//!
//! It also shows how to handle tool calls at final messages in the conversation.
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_BASE_URL="http://compass.llm.shopee.io/compass-api"
//! export ANTHROPIC_AUTH_METHOD="bearer"           # or "api_key" for normal usage
//! export ANTHROPIC_API_KEY="your_token_here"      # required if AUTH_METHOD=api_key
//! cargo run --example test_tool_multiturn_streaming
//! ```

use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use anthropic_sdk::Tool;
use anthropic_sdk::{
    types::{
        ContentBlock, ContentBlockDelta, ContentBlockParam, ContentBlocksExt, MessageContent,
        MessageCreateBuilder, MessageStreamEvent, ToolUse as ToolUseBlock,
    },
    Anthropic, Role, ToolFunction, ToolRegistry, ToolResult, ToolResultContent,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub struct EchoTool;

#[async_trait]
impl ToolFunction for EchoTool {
    async fn execute(
        &self,
        input: Value,
    ) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ToolResult::success_json("echo", input))
    }
}

impl EchoTool {
    /// JSON-schema definition that will be sent to Claude.
    pub fn definition() -> Tool {
        Tool::new("echo", "Echo the provided JSON back unchanged")
            .parameter("any", "object", "Arbitrary JSON to echo back")
            .build()
    }
}

/// Simple fake weather tool that returns static weather data.
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

/// Convenience helper that constructs a `ToolRegistry` pre-populated with the
/// two demo tools above. Useful for quick tests.
pub fn demo_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    registry
        .register("echo", EchoTool::definition(), Box::new(EchoTool))
        .expect("failed to register echo tool");

    registry
        .register(
            "get_weather",
            WeatherTool::definition(),
            Box::new(WeatherTool),
        )
        .expect("failed to register weather tool");

    registry
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // tracing_subscriber::fmt::init();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("🧪 Streaming Multi-turn Tool Usage Demo");
    println!("{}", "=".repeat(60));

    // Create client from environment
    let client = Anthropic::from_env()?;
    println!("✅ Client initialized successfully\n");

    let registry = demo_registry();
    let tool_decls = registry.get_tool_definitions();
    println!("🔧 Tool registry contains {} tools", tool_decls.len());

    // First, test basic streaming without tools to ensure it works
    println!("🌊 Testing basic streaming without tools...\n");
    let basic_stream = client
        .messages()
        .create_stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
                .user("Say hello!")
                .build(),
        )
        .await?;

    let basic_response = basic_stream
        .on_text(|delta, _| {
            print!("{delta}");
        })
        .final_message()
        .await?;

    println!(
        "\n✅ Basic streaming works! Text-Only Response: {}\n",
        basic_response.content.get_text()
    );

    for (i, block) in basic_response.content.iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                println!("{block:?}");
                println!("Block {i}: Text: {text}");
            }
            ContentBlock::ToolUse { id, name, input } => {
                println!("{block:?}");
                println!("Block {i}: ToolUse: {name} {id} {input}");
            }
            _ => {
                println!("Block {i}: Other type");
            }
        }
    }

    println!("🌊 Testing basic streaming with tools fetching final message...\n");
    let basic_stream = client
        .messages()
        .create_stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
                .tools(tool_decls.clone())
                .user("Echo hello! and tell me the weather in Beijing and Paris")
                .build(),
        )
        .await?;

    let basic_response = basic_stream.final_message().await?;

    println!(
        "\n✅ Basic streaming with tools works! Text-Only Response: {}\n",
        basic_response.content.get_text()
    );

    for (i, block) in basic_response.content.iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                println!("Block {i}: Text: {text}");
            }
            ContentBlock::ToolUse { id, name, input } => {
                println!("Block {i}: ToolUse: {name} {id} {input}");
            }
            _ => {
                println!("Block {i}: Other type");
            }
        }
    }

    // Now test with tools - use non-streaming first to establish baseline
    println!("🔧 Testing non-streaming tool usage first...\n");
    let non_stream_msg = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
                .user("Echo hello! and tell me the weather in Beijing and Paris")
                .tools(tool_decls.clone())
                .build(),
        )
        .await?;

    println!(
        "Non-streaming response has {} content blocks",
        non_stream_msg.content.len()
    );

    for (i, block) in non_stream_msg.content.iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                println!("Block {i}: Text: {text}");
            }
            ContentBlock::ToolUse { id, name, input } => {
                println!("Block {i}: ToolUse: {name} {id} {input}");
            }
            _ => {
                println!("Block {i}: Other type");
            }
        }
    }

    // Now try streaming with tools - use the proper streaming API
    println!("\n🌊 Testing streaming with tools...\n");

    let stream_params = MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
        .user("Echo hello! and tell me the weather in Beijing and Paris")
        .tools(tool_decls)
        .stream(true)
        .build();

    // Debug: print the parameters being sent
    println!("🔍 Streaming request parameters:");
    println!("- Model: {}", stream_params.model);
    println!(
        "- Tools count: {}",
        stream_params.tools.as_ref().map(|t| t.len()).unwrap_or(0)
    );
    if let Some(tools) = &stream_params.tools {
        for tool in tools {
            println!("  - Tool: {} {}", tool.name, tool.description);
        }
    }

    // Track tool uses as they come in
    let tool_uses = Arc::new(Mutex::new(Vec::<ToolUseBlock>::new()));
    let tool_uses_clone = tool_uses.clone();

    // Use the proper streaming API with callbacks
    let tool_stream = client.messages().create_stream(stream_params).await?;

    let stream_response = tool_stream
        .on_text(|delta, _| {
            print!("{delta}");
        })
        .on_stream_event(move |event, current_message| {
            match event {
                MessageStreamEvent::MessageStart { .. } => {
                    println!("📝 Message started");
                }
                MessageStreamEvent::ContentBlockStart {
                    content_block: ContentBlock::ToolUse { id, name, input },
                    index,
                } => {
                    println!("🔧 Tool use started: {name} ({id}) at index {index}");
                    let tool_use = ToolUseBlock {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    };
                    tool_uses_clone.lock().unwrap().push(tool_use);
                }
                MessageStreamEvent::ContentBlockDelta {
                    delta: ContentBlockDelta::InputJsonDelta { partial_json },
                    index,
                } => {
                    println!("🔧 Tool input delta at index {index}: {partial_json}");
                    // Update the tool use with the latest input - but only if it's valid JSON
                    if let Ok(parsed) = serde_json::from_str(partial_json) {
                        let mut tools = tool_uses_clone.lock().unwrap();
                        // Find the tool at this index and update it
                        if let Some(tool_use) = tools.iter_mut().rev().find(|t| {
                            // Find the most recent tool that matches this stream position
                            println!("🔧 Comparing tool at index {index} to tool at {t:?}");
                            true // For now, just update the last one
                        }) {
                            tool_use.input = parsed;
                        }
                    }
                }
                MessageStreamEvent::InputJson {
                    partial_json,
                    snapshot,
                } => {
                    println!(
                        "🔧 Input JSON event: {partial_json} (snapshot: {})",
                        serde_json::to_string(&snapshot).unwrap_or_else(|_| "invalid".to_string())
                    );
                    // This is an independent event that provides JSON parsing snapshots
                    // It's informational and doesn't need to update specific tool indices
                }
                MessageStreamEvent::ContentBlockStop {
                    index,
                    content_block,
                } => {
                    println!("🔧 Content block stopped at index {index}");
                    // First try to use the final content_block from the event if available
                    let final_content_block = if let Some(cb) = content_block {
                        Some(cb)
                    } else {
                        // Fall back to getting from current message
                        current_message.content.get(*index)
                    };

                    if let Some(ContentBlock::ToolUse { id, name, input }) = final_content_block {
                        println!("🔧 Final tool use: {name} ({id}) with input: {input}");
                        // Update the tool use with the final complete input
                        let mut tools = tool_uses_clone.lock().unwrap();
                        if let Some(existing) = tools.iter_mut().find(|t| t.id == *id) {
                            existing.input = input.clone();
                        } else {
                            // If somehow we missed the ContentBlockStart, add it now
                            let final_tool_use = ToolUseBlock {
                                id: id.clone(),
                                name: name.clone(),
                                input: input.clone(),
                            };
                            tools.push(final_tool_use);
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

    let completed_tools: Vec<ToolUseBlock> = tool_uses.lock().unwrap().clone();

    if completed_tools.is_empty() {
        println!("❌ No tool calls detected in streaming response!");
        println!("Final response: {}", stream_response.content.get_text());
        return Ok(());
    }

    println!(
        "✅ Found {} tool call(s) in streaming response",
        completed_tools.len()
    );

    // Execute the tools
    let mut tool_results = Vec::new();
    for tool_use in &completed_tools {
        println!(
            "🔧 Executing tool '{}' with input: {}",
            tool_use.name, tool_use.input
        );

        match registry.execute(tool_use).await {
            Ok(result) => {
                println!("✅ Tool '{}' executed successfully", tool_use.name);
                tool_results.push(result);
            }
            Err(err) => {
                println!("❌ Tool '{}' failed: {}", tool_use.name, err);
                tool_results.push(ToolResult {
                    tool_use_id: tool_use.id.clone(),
                    content: ToolResultContent::Text(format!("Execution error: {err}")),
                    is_error: Some(true),
                });
            }
        }
    }

    // Convert the streaming message to assistant content blocks for the next turn
    let assistant_content_blocks: Vec<ContentBlockParam> = stream_response
        .content
        .iter()
        .map(|b| match b {
            ContentBlock::Text { text } => ContentBlockParam::Text {
                text: text.clone(),
                cache_control: None,
            },
            ContentBlock::ToolUse { id, name, input } => ContentBlockParam::ToolUse {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
                cache_control: None,
            },
            _ => ContentBlockParam::Text {
                text: "".to_string(),
                cache_control: None,
            },
        })
        .collect();

    // Convert tool results to content blocks for the user message
    let tool_result_blocks: Vec<ContentBlockParam> = tool_results
        .into_iter()
        .map(|tr| ContentBlockParam::ToolResult {
            tool_use_id: tr.tool_use_id,
            content: match tr.content {
                ToolResultContent::Text(t) => Some(t),
                ToolResultContent::Json(v) => Some(v.to_string()),
                ToolResultContent::Blocks(_) => None,
            },
            is_error: tr.is_error,
            cache_control: None,
        })
        .collect();

    // Build follow-up message with tool results
    let follow_up_builder = MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
        .user("Echo hello! and tell me the weather in Beijing and Paris")
        .message(
            Role::Assistant,
            MessageContent::Blocks(assistant_content_blocks),
        )
        .message(Role::User, MessageContent::Blocks(tool_result_blocks));

    println!("🌊 Starting second streaming turn...\n");

    // Create the second streaming request with tool results
    let follow_up_stream = client
        .messages()
        .create_stream(follow_up_builder.build())
        .await?;

    // Process the follow-up stream
    let final_follow_up = follow_up_stream
        .on_text(|delta, _| {
            print!("{delta}");
        })
        .on_stream_event(|event, _| match event {
            MessageStreamEvent::MessageStart { .. } => {
                print!("💬 ");
            }
            MessageStreamEvent::MessageStop => {
                println!("\n🏁 Conversation completed!");
            }
            _ => {}
        })
        .final_message()
        .await?;

    println!("\n✨ Final conversation summary:");
    println!(
        "📊 Total tokens used: {}",
        final_follow_up.usage.input_tokens + final_follow_up.usage.output_tokens
    );
    println!("🔧 Tools executed: {}", completed_tools.len());

    Ok(())
}
