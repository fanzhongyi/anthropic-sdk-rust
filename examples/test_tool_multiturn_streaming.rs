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

use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::fs;
use std::path::PathBuf;

use anthropic_sdk::Tool;
use anthropic_sdk::{
    types::{
        ContentBlock, ContentBlockDelta, ContentBlockParam, ContentBlocksExt, MessageContent,
        MessageCreateBuilder, MessageStreamEvent, ToolUse as ToolUseBlock,
        // Added for richer tool_result content mapping
        ToolResultContentParam, ToolResultNestedBlockParam, ToolResultBlock,
        ImageSource, ToolImageSource,
    },
    Anthropic, Role, ToolFunction, ToolRegistry, ToolResult, ToolResultContent,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

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

pub struct GlobTool;

#[async_trait]
impl ToolFunction for GlobTool {
    async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn Error + Send + Sync>> {
        if !input.is_object() {
            return Ok(ToolResult::error(TOOL_NAME, "Input must be an object"));
        }
        let o = input.as_object().unwrap();

        // Parameters
        let base_path_str = o.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let include_hidden = o
            .get("include_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let respect_ignore_files = o
            .get("respect_ignore_files")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let ignore_files: Vec<String> = o
            .get("ignore_files")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec![".gitignore".to_string(), ".ignore".to_string()]);
        let follow_symlinks = o
            .get("follow_symlinks")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let recursive = o.get("recursive").and_then(|v| v.as_bool()).unwrap_or(true);

        let patterns: Vec<String> = o
            .get("patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        if patterns.is_empty() {
            return Ok(ToolResult::error(
                TOOL_NAME,
                "'patterns' must be a non-empty array of strings",
            ));
        }

        // Resolve and validate base path under cwd
        let base_path = PathBuf::from(base_path_str);
        if !base_path.exists() {
            return Ok(ToolResult::error(
                TOOL_NAME,
                format!("Path not found: {}", base_path.display()),
            ));
        }
        if !base_path.is_dir() {
            return Ok(ToolResult::error(
                TOOL_NAME,
                format!("Path is not a directory: {}", base_path.display()),
            ));
        }
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let base_abs = match base_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Ok(ToolResult::error(
                    TOOL_NAME,
                    format!("Canonicalize base failed: {e}"),
                ))
            }
        };
        let cwd_abs = match cwd.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Ok(ToolResult::error(
                    TOOL_NAME,
                    format!("Canonicalize cwd failed: {e}"),
                ))
            }
        };
        if !base_abs.starts_with(&cwd_abs) {
            return Ok(ToolResult::error(
                TOOL_NAME,
                format!(
                    "Path {} is outside of cwd {}",
                    base_abs.display(),
                    cwd_abs.display()
                ),
            ));
        }

        // Build ripgrep-style overrides
        let mut ob = OverrideBuilder::new(&base_path);
        for pat in &patterns {
            if let Err(e) = ob.add(pat) {
                return Ok(ToolResult::error(
                    TOOL_NAME,
                    format!("Invalid pattern '{pat}': {e}"),
                ));
            }
        }
        let overrides = match ob.build() {
            Ok(o) => o,
            Err(e) => {
                return Ok(ToolResult::error(
                    TOOL_NAME,
                    format!("Failed to build overrides: {e}"),
                ))
            }
        };

        // Build walker
        let mut wb = WalkBuilder::new(&base_path);
        wb.hidden(!include_hidden)
            .git_ignore(respect_ignore_files)
            .ignore(respect_ignore_files)
            .parents(respect_ignore_files)
            .follow_links(follow_symlinks)
            .max_depth(if recursive { None } else { Some(1) })
            .overrides(overrides);

        if respect_ignore_files {
            for f in &ignore_files {
                let p = base_path.join(f);
                if p.exists() {
                    wb.add_ignore(p);
                }
            }
        }

        // Walk and collect matches
        const MAX_MATCHES: usize = 5000;
        let mut entries_json = Vec::new();
        let mut truncated = false;

        for dent in wb.build() {
            if truncated {
                break;
            }
            let dent = match dent {
                Ok(d) => d,
                Err(_) => continue,
            };
            let path = dent.path().to_path_buf();
            if path == base_path {
                continue;
            }
            let ty = match dent.file_type() {
                Some(t) => t,
                None => continue,
            };
            // Only emit files
            if !ty.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let is_hidden = file_name.starts_with('.');
            if is_hidden && !include_hidden {
                continue;
            }
            let rel_path = match path.strip_prefix(&base_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => file_name.clone(),
            };
            let meta = match fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            entries_json.push(json!({
                "name": file_name,
                "path": rel_path,
                "kind": "file",
                "size": meta.len(),
                "is_hidden": is_hidden,
            }));
            if entries_json.len() >= MAX_MATCHES {
                truncated = true;
            }
        }

        let result = json!({
            "root": base_path.to_string_lossy(),
            "patterns": patterns,
            "include_hidden": include_hidden,
            "respect_ignore_files": respect_ignore_files,
            "ignore_files": ignore_files,
            "follow_symlinks": follow_symlinks,
            "recursive": recursive,
            "total_matches": entries_json.len(),
            "truncated": truncated,
            "entries": entries_json,
        });

        Ok(ToolResult::success_json(TOOL_NAME, result))
    }

    fn validate_input(&self, input: &Value) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !input.is_object() {
            return Err("Input must be a JSON object".into());
        }
        let o = input.as_object().unwrap();
        if let Some(v) = o.get("path") {
            if !v.is_string() {
                return Err("'path' must be a string".into());
            }
        }
        if let Some(v) = o.get("include_hidden") {
            if !v.is_boolean() {
                return Err("'include_hidden' must be a boolean".into());
            }
        }
        if let Some(v) = o.get("respect_ignore_files") {
            if !v.is_boolean() {
                return Err("'respect_ignore_files' must be a boolean".into());
            }
        }
        if let Some(v) = o.get("ignore_files") {
            let arr = v
                .as_array()
                .ok_or("'ignore_files' must be an array of strings")?;
            if !arr.iter().all(|x| x.as_str().is_some()) {
                return Err("'ignore_files' must be an array of strings".into());
            }
        }
        if let Some(v) = o.get("follow_symlinks") {
            if !v.is_boolean() {
                return Err("'follow_symlinks' must be a boolean".into());
            }
        }
        if let Some(v) = o.get("recursive") {
            if !v.is_boolean() {
                return Err("'recursive' must be a boolean".into());
            }
        }
        if let Some(v) = o.get("patterns") {
            let arr = v
                .as_array()
                .ok_or("'patterns' must be an array of strings")?;
            if arr.is_empty() || !arr.iter().all(|x| x.as_str().is_some()) {
                return Err("'patterns' must be a non-empty array of strings".into());
            }
        }
        Ok(())
    }

    fn timeout_seconds(&self) -> u64 {
        TIMEOUT_SECONDS
    }
}

pub const TOOL_NAME: &str = "glob";
pub const DESCRIPTION: &str = "Match files using glob patterns (read-only). Supports multiple patterns with negation and optional ignore files.";
pub const TIMEOUT_SECONDS: u64 = 10;

impl GlobTool {
    pub fn definition() -> Tool {
        Tool::new(TOOL_NAME, DESCRIPTION)
            .parameter(
                "path",
                "string",
                "Base directory to search. Must be under project cwd. Defaults to '.'",
            )
            .array_parameter(
                "patterns",
                "Glob patterns. Use leading '!' for exclusion. Example: ['**/*.rs', '!target/**']",
                "string",
            )
            .parameter(
                "include_hidden",
                "boolean",
                "Include entries whose names start with '.' (default: false)",
            )
            .parameter(
                "respect_ignore_files",
                "boolean",
                "If true (default), read ignore files (see 'ignore_files') and apply their patterns.",
            )
            .array_parameter(
                "ignore_files",
                "Ignore files to read from the base path (default ['.gitignore', '.ignore']).",
                "string",
            )
            .parameter(
                "follow_symlinks",
                "boolean",
                "Follow symlinks when recursing (default: false).",
            )
            .parameter(
                "recursive",
                "boolean",
                "Recurse into subdirectories when searching (default: true).",
            )
            .required("patterns")
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
        .register("glob", GlobTool::definition(), Box::new(GlobTool))
        .expect("Failed to register glob tool");

    registry
}

// pub const TEST_QUERY: &str = "Echo hello! and tell me is there any shell script in current directory?";
// pub const TEST_QUERY: &str = "Echo hello! and 帮我找下这个目录中所有的test开头 .rs结尾的文件";
// pub const TEST_QUERY: &str = "告诉我现在目录下有哪些文件，这是个什么项目，最后帮我找下这个目录中的所有prompt.rs文件";
pub const TEST_QUERY: &str = "帮我找下这个目录中的所有shell 脚本文件，不要查找子目录, 名字里包含git的不要，最后猜测下这些shell文件干嘛用的？";

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
    for tool in tool_decls.clone().into_iter() {
        println!("- {} {}", tool.name, tool.description);
    }

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
                .user(TEST_QUERY)
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
                .user(TEST_QUERY)
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
        .user(TEST_QUERY)
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
        .map(|tr| {
            let content_param: Option<ToolResultContentParam> = match tr.content {
                ToolResultContent::Text(t) => Some(ToolResultContentParam::Text(t)),
                ToolResultContent::Json(v) => {
                    // Render JSON as a single nested text block within Blocks for richer structure
                    let pretty = serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string());
                    Some(ToolResultContentParam::Blocks(vec![
                        ToolResultNestedBlockParam::Text { text: pretty, cache_control: None }
                    ]))
                },
                ToolResultContent::Blocks(blocks) => {
                    // Map tool result blocks to nested tool_result content blocks supported by messages API
                    let nested: Vec<ToolResultNestedBlockParam> = blocks
                        .into_iter()
                        .map(|b| match b {
                            ToolResultBlock::Text { text } => ToolResultNestedBlockParam::Text {
                                text,
                                cache_control: None,
                            },
                            ToolResultBlock::Image { source } => {
                                // Convert ToolImageSource -> messages::ImageSource
                                match source {
                                    ToolImageSource::Base64 { media_type, data } => {
                                        ToolResultNestedBlockParam::Image {
                                            source: ImageSource::Base64 { media_type, data },
                                            cache_control: None,
                                        }
                                    }
                                }
                            }
                        })
                        .collect();
                    Some(ToolResultContentParam::Blocks(nested))
                }
            };
            ContentBlockParam::ToolResult {
                tool_use_id: tr.tool_use_id,
                content: content_param,
                is_error: tr.is_error,
                cache_control: None,
            }
        })
        .collect();

    // Build follow-up message with tool results
    let follow_up_builder = MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
        .user(TEST_QUERY)
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
            MessageStreamEvent::InputJson { partial_json, snapshot } => {
                println!("🔧 Input JSON event: {partial_json} (snapshot: {snapshot})");
            }
            _ => {}
        })
        .final_message()
        .await?;

    println!("\n🌊 Final conversation: {final_follow_up:?}");
    println!("\n✨ Final conversation summary:");
    println!(
        "📊 Total tokens used: {}",
        final_follow_up.usage.input_tokens + final_follow_up.usage.output_tokens
    );
    println!("🔧 Tools executed: {}", completed_tools.len());

    Ok(())
}
