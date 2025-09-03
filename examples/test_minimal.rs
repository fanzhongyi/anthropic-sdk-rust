//! Minimal Demo for Testing New Features
//!
//! This is the simplest possible demo to verify Prompt caching and Extended thinking.
//! Perfect for quick testing during development.

use anthropic_sdk::{
    types::{CacheControl, MessageCreateBuilder, SystemContentBlock},
    Anthropic,
};

#[tokio::main]
async fn main() {
    println!("🧪 Minimal Feature Test");
    println!("======================\n");

    // Check environment
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("❌ ANTHROPIC_API_KEY not set!");
        eprintln!("💡 Run: export ANTHROPIC_API_KEY=your_key_here");
        return;
    }

    let client = match Anthropic::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Client error: {e}");
            return;
        }
    };

    // Test 1: Extended Thinking (simplest test)
    println!("🤔 Testing Extended Thinking...");
    match client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 4096)
                .thinking(1024)
                .user("What is 15 + 27? Think step by step.")
                .build(),
        )
        .await
    {
        Ok(msg) => {
            println!("✅ Extended thinking request succeeded!");
            println!(
                "📊 Tokens: {} in, {} out",
                msg.usage.input_tokens, msg.usage.output_tokens
            );

            // Print response content
            println!("📄 Response content:");
            for (i, block) in msg.content.iter().enumerate() {
                match block {
                    anthropic_sdk::types::ContentBlock::Text { text } => {
                        println!("   [{i}] Text: {text}");
                    }
                    anthropic_sdk::types::ContentBlock::Thinking {
                        thinking,
                        signature,
                    } => {
                        println!("   [{i}] Thinking: {thinking}");
                        println!("       Signature: {signature}");
                    }
                    _ => {
                        println!("   [{i}] Other content type: {block:?}");
                    }
                }
            }

            // Check for thinking content
            let has_thinking = msg
                .content
                .iter()
                .any(|block| matches!(block, anthropic_sdk::types::ContentBlock::Thinking { .. }));

            if has_thinking {
                println!("🧠 Thinking content detected!");
            } else {
                println!("ℹ️  No thinking content (might be normal for simple questions)");
            }
        }
        Err(e) => println!("❌ Extended thinking failed: {e}"),
    }

    println!();

    // Test 2: Prompt Caching (simplest test)
    println!("💾 Testing Prompt Caching...");
    let system_prompt = "You are a helpful assistant with expertise in mathematics and science.";

    match client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 1024)
                .system(vec![SystemContentBlock::text_with_cache(
                    system_prompt,
                    CacheControl::ephemeral(),
                )])
                .user("Hello!")
                .build(),
        )
        .await
    {
        Ok(msg) => {
            println!("✅ Prompt caching request succeeded!");
            println!(
                "📊 Tokens: {} in, {} out",
                msg.usage.input_tokens, msg.usage.output_tokens
            );

            // Print response content
            println!("📄 Response content:");
            for (i, block) in msg.content.iter().enumerate() {
                match block {
                    anthropic_sdk::types::ContentBlock::Text { text } => {
                        println!("   [{i}] Text: {text}");
                    }
                    anthropic_sdk::types::ContentBlock::Thinking {
                        thinking,
                        signature,
                    } => {
                        println!("   [{i}] Thinking: {thinking}");
                        println!("       Signature: {signature}");
                    }
                    _ => {
                        println!("   [{i}] Other content type: {block:?}");
                    }
                }
            }

            if let Some(cache_tokens) = msg.usage.cache_creation_input_tokens {
                println!("💾 Cache created: {cache_tokens} tokens");
            }

            if let Some(read_tokens) = msg.usage.cache_read_input_tokens {
                println!("📖 Cache read: {read_tokens} tokens");
            }
        }
        Err(e) => println!("❌ Prompt caching failed: {e}"),
    }

    println!("\n🎯 Feature Test Complete!");
    println!("If you see ✅ marks above, your implementation is working!");
}
