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
        CacheControl, ContentBlock, ContentBlockParam, ContentBlocksExt, MessageContent, MessageCreateBuilder, SystemContentBlock, ThinkingConfig,
    }, Anthropic
};
use anthropic_sdk::types::streaming::{MessageStreamEvent, ContentBlockDelta};
// use serde_json::json;  // removed unused import
use futures::StreamExt;
use anthropic_sdk::Tool;
use std::error::Error;
use base64::engine::general_purpose;
use base64::Engine;
use std::path::Path;

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

    // ------------------------------------------------------------------
    // 1  Simple message (Quick-start snippet)
    // ------------------------------------------------------------------
    println!("1  Simple Message Request\n");
    let msg = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 256)
                .user("Hello, Claude! How are you today?")
                .build(),
        )
        .await?;

    println!("Claude: {}\n", msg.content.get_text());

    // ------------------------------------------------------------------
    // 2  Advanced parameters (temperature / top-p / stop)
    // ------------------------------------------------------------------
    println!("2  Advanced Parameters\n");
    let params = MessageCreateBuilder::new("claude-3-haiku@20240307", 512)
        .user("Write a creative story about space exploration.")
        .temperature(0.8)
        .top_p(0.9)
        .top_k(50)
        .stop_sequences(vec!["THE END".to_string()])
        .build();

    let msg = client.messages().create(params).await?;
    println!("Story Tokens – in: {}, out: {}\n", msg.usage.input_tokens, msg.usage.output_tokens);

    // ------------------------------------------------------------------
    // 3  Extended Thinking
    // ------------------------------------------------------------------
    println!("3  Extended Thinking\n");
    let msg = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 2048)
                .thinking_config(ThinkingConfig::enabled(1024))
                .user("What is 37*42? Think step by step.")
                .build(),
        )
        .await?;

    for block in &msg.content {
        match block {
            ContentBlock::Thinking { thinking, .. } => println!("🤔 {thinking}\n"),
            ContentBlock::Text { text } => println!("📝 {text}\n"),
            _ => {}
        }
    }

    // ------------------------------------------------------------------
    // 4  Prompt caching (ephemeral)
    // ------------------------------------------------------------------
    println!("4  Prompt Caching\n");
    let system_prompt = "You are a helpful assistant with expertise in mathematics and science.";
    let msg = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 256)
                .system(vec![SystemContentBlock::text_with_cache(
                    system_prompt,
                    CacheControl::ephemeral(),
                )])
                .user("Hello again!")
                .build(),
        )
        .await?;

    println!("Tokens – in: {}, out: {}", msg.usage.input_tokens, msg.usage.output_tokens);
    if let Some(created) = msg.usage.cache_creation_input_tokens {
        println!("💾 Cache created ({created} tokens)");
    }
    if let Some(read) = msg.usage.cache_read_input_tokens {
        println!("📖 Cache read ({read} tokens)");
    }

    // ------------------------------------------------------------------
    // 5  Vision (image + text)
    // ------------------------------------------------------------------
    println!("5  Vision – Describe Image\n");
    // Load local image and encode to base64 (replace with your own path if needed)
    let img_path = Path::new("fixtures/158fcfbb6eff1f82ff4dc62beec9abd4.jpg");
    let img_bytes = std::fs::read(img_path)?;
    let img_base64 = general_purpose::STANDARD.encode(&img_bytes);
    let params = MessageCreateBuilder::new("claude-sonnet-4@20250514", 8192)
        .thinking(2048)
        .user(MessageContent::Blocks(vec![
            ContentBlockParam::text("What do you see in this image? Deep think it and say it in Chinese,上面有文字吗,上面有汉字吗，和小猫有关吗"),
            ContentBlockParam::image_base64("image/jpeg", &img_base64),
        ]))
        .build();

    // Gateway may reject dummy data; we still ensure it compiles
    match client.messages().create(params).await {
        Ok(resp) => println!("Vision response text: {}", resp.content.get_text()),
        Err(e) => println!("(Expected) Vision request failed: {e}"),
    }

    // ------------------------------------------------------------------
    // 6  Streaming Response (real-time tokens)
    // ------------------------------------------------------------------
    println!("6  Streaming Response\n");
    let stream = client
        .messages()
        .stream(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 128)
                .user("Please count from 1 to 5.")
                .build(),
        )
        .await?;
    tokio::pin!(stream);
    while let Some(event) = stream.next().await {
        match event? {
            MessageStreamEvent::ContentBlockDelta { delta: ContentBlockDelta::TextDelta { text }, .. } => {
                print!("{text}");
            }
            MessageStreamEvent::MessageStop => {
                println!(" <- stream end\n");
                break;
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------------
    // 7  Multi-turn Conversation
    // ------------------------------------------------------------------
    println!("7  Multi-turn Conversation\n");
    let convo = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 256)
                .user("Hi Claude! What's your name?")
                .assistant("Hello! I'm Claude, an AI assistant.")
                .user("Nice to meet you. What's 2+2?")
                .user("Nice to meet you. What's 5+2?")
                .build(),
        )
        .await?;
    // println!("Claude: {}\n", convo.content.get_text());
    for block in &convo.content {
        if let ContentBlock::Text { text } = block {
            println!("📝 Claude: {text} ");
        }
    }

    // ------------------------------------------------------------------
    // 8  Tool Usage Demo
    // ------------------------------------------------------------------
    println!("8  Tool Usage Demo\n");
    let weather_tool = Tool::new("get_weather", "Get current weather for a location")
        .parameter("location", "string", "City name")
        .required("location")
        .build();

    let tool_msg = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-3-haiku@20240307", 256)
                .user("What's the weather like in Paris?")
                .tools(vec![weather_tool.clone()])
                .build(),
        )
        .await?;
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

    // ------------------------------------------------------------------
    // 9  File Management Demo (upload & list)
    // ------------------------------------------------------------------
    println!("9  File Management Demo\n");
    let doc_path = Path::new("fixtures/sample.txt");
    if doc_path.exists() {
        let content = std::fs::read(doc_path)?;
        use anthropic_sdk::types::{FileUploadParams, FilePurpose};
        let upload_params = FileUploadParams::new(
            content,
            "sample.txt",
            "text/plain",
            FilePurpose::Upload,
        );
        match client.files().upload(upload_params).await {
            Ok(file_obj) => {
                println!("Uploaded file '{}' (id: {})", file_obj.filename, file_obj.id);
                let files = client.files().list(None).await?;
                println!("Total files: {}", files.data.len());
            }
            Err(e) => println!("File upload failed: {e}"),
        }
    } else {
        println!("sample.txt not found – skipping file upload demo");
    }

    println!("\n🎯 Demo Complete! All README feature snippets executed.");
    Ok(())
}
