//! Basic Demo to Exercise README Features
//!
//! This example is intended to be executed against either Anthropic's production
//! endpoint or a compatible gateway (e.g. Compass). It showcases a subset of
//! the feature snippets that appear in the README so you can quickly verify
//! they compile **and** work end-to-end.
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_BASE_URL="http://compass.llm.shopee.io/compass-api"
//! export ANTHROPIC_AUTH_METHOD="bearer"           # or "api_key" for normal usage
//! export ANTHROPIC_API_KEY="your_token_here"      # required if AUTH_METHOD=api_key
//! cargo run --example test_basic
//! ```

use anthropic_sdk::{
    types::{
        ContentBlock, ContentBlockParam, ContentBlocksExt, MessageContent, MessageCreateBuilder,
    }, Anthropic, Role, ToolFunction, ToolRegistry, ToolResult, ToolResultContent
};
use anthropic_sdk::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;

pub struct EchoTool;

#[async_trait]
impl ToolFunction for EchoTool {
    async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
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
    async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let weather_data = json!({
            "location": input["location"].as_str().unwrap_or("Unknown"),
            "temperature": "80°C",
            "condition": "Sunny",
            "humidity": "45%",
            "wind": "5 mph"
        });
        Ok(ToolResult::success_json("weather", weather_data))
        // Ok(ToolResult::error("weather", "No this place"))
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
        .register("get_weather", WeatherTool::definition(), Box::new(WeatherTool))
        .expect("failed to register weather tool");

    registry
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🧪 Basic Anthropic SDK Demo");
    println!("{}", "=".repeat(60));

    // Create client from environment (ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_METHOD, etc.)
    let client = Anthropic::from_env()?;
    println!("✅ Client initialized successfully\n");

    // let models = client.models().list(None).await?;
    // for model in models.data {
    //     println!("Model: {} ({})", model.display_name, model.id);
    // }

    let registry = demo_registry();
    let tool_decls = registry.get_tool_definitions();
    println!("🔧 Tool registry contains {} tools", tool_decls.len());

    let mut msg_builder = MessageCreateBuilder::new("claude-3-haiku@20240307", 256)
        .user("What's the weather like in Paris?")
        .tools(tool_decls);

    let tool_msg = client.messages().create(msg_builder.clone().build()).await?;

    for block in &tool_msg.content {
        match block {
            ContentBlock::ToolUse { name, input, .. } => {
                println!("🔧 Claude requested tool '{name}' with input {input}");
            }
            ContentBlock::Text { text } => {
                println!("📝 Claude: {text}");
            }
            _ => {}
        }
    }

    // Convert assistant response content blocks into MessageContent blocks for next round
    let assistant_content_blocks: Vec<ContentBlockParam> = tool_msg
        .content
        .iter()
        .map(|b| match b {
            ContentBlock::Text { text } => {
                println!("text: {text}");
                ContentBlockParam::Text { text: text.clone(), cache_control: None }
            }
            ContentBlock::Image { source } => {
                ContentBlockParam::Image { source: source.clone(), cache_control: None }
            }
            ContentBlock::ToolUse { id, name, input } => {
                println!("tool_use: {id} {name} {input:?}");
                ContentBlockParam::ToolUse { id: id.clone(), name: name.clone(), input: input.clone(), cache_control: None }
            }
            ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                ContentBlockParam::ToolResult { tool_use_id: tool_use_id.clone(), content: content.clone(), is_error: *is_error, cache_control: None }
            }
            ContentBlock::Thinking { thinking, signature } => {
                ContentBlockParam::Thinking { thinking: thinking.clone(), signature: signature.clone(), cache_control: None }
            }
            ContentBlock::RedactedThinking { data } => {
                ContentBlockParam::RedactedThinking { data: data.clone(), cache_control: None }
            }
        })
        .collect();
    msg_builder = msg_builder.message(Role::Assistant, MessageContent::Blocks(assistant_content_blocks));

    // Check for tool calls.
    let mut pending_calls = Vec::new();
    for block in &tool_msg.content {
        if let ContentBlock::ToolUse { id, name, input } = block {
            pending_calls.push((id.clone(), name.clone(), input.clone()));
        }
    }

    if pending_calls.is_empty() {
        // No tool calls ⇒ final answer.
        let out = tool_msg.content.get_text();
        if out.is_empty() {
            println!("🤔 Claude said nothing.");
        } else {
            println!("📝 Claude said: {out}");
        };
        return Ok(());
    }

    let mut results = Vec::new();
    for (id, name, input) in pending_calls {
        let tool_use = anthropic_sdk::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
        };
        match registry.execute(&tool_use).await {
            Ok(tr) => {
                results.push(tr);
            }
            Err(err) => {
                results.push(ToolResult {
                    tool_use_id: id,
                    content: ToolResultContent::Text(format!("Execution error: {err}")),
                    is_error: Some(true),
                });
            }
        }
    }

    let tool_result_blocks: Vec<ContentBlockParam> = results
        .into_iter()
        .map(|tr| ContentBlockParam::ToolResult {
            tool_use_id: tr.tool_use_id,
            content: match tr.content {
                ToolResultContent::Text(t) => Some(t),
                ToolResultContent::Json(v) => Some(v.to_string()),
                ToolResultContent::Blocks(_) => None, // simplify
            },
            is_error: tr.is_error,
            cache_control: None,
        })
        .collect();

    msg_builder = msg_builder.message(Role::User, MessageContent::Blocks(tool_result_blocks));

    let tool_msg_follow_up = client.messages().create(msg_builder.clone().build()).await?;

    for block in &tool_msg_follow_up.content {
        match block {
            ContentBlock::ToolUse { name, input, .. } => {
                println!("🔧 Claude requested tool '{name}' with input {input}");
            }
            ContentBlock::Text { text } => {
                println!("📝 Claude: {text}");
            }
            _ => {}
        }
    }

    Ok(())
}
