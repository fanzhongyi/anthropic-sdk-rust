//! Quick Demo for Prompt Caching and Extended Thinking
//!
//! This is a simple demo to verify the new Prompt caching and Extended thinking features.
//! Run with: cargo run --example quick_demo
//!
//! Make sure to set your ANTHROPIC_API_KEY environment variable first:
//! export ANTHROPIC_API_KEY=your_api_key_here

use anthropic_sdk::{
    Anthropic,
    types::{
        MessageCreateBuilder,
        SystemContentBlock,
        ContentBlockParam,
        CacheControl,
        ThinkingConfig,
        ContentBlock
    }
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Anthropic SDK Demo - Prompt Caching & Extended Thinking");
    println!("================================================================\n");

    // Create client from environment variables
    let client = match Anthropic::from_env() {
        Ok(client) => {
            println!("✅ Client initialized successfully");
            client
        },
        Err(e) => {
            eprintln!("❌ Failed to initialize client: {e}");
            eprintln!("💡 Make sure to set ANTHROPIC_API_KEY environment variable");
            return Err(Box::new(e) as Box<dyn Error>);
            // return Err(e);
        }
    };
    // let client = Anthropic::from_env()?;
    println!("✅ Client initialized successfully");

    // Test 1: Basic Extended Thinking
    println!("\n📝 Test 1: Basic Extended Thinking");
    println!("----------------------------------");
    test_basic_thinking(&client).await?;

    // Test 2: Basic Prompt Caching
    println!("\n💾 Test 2: Basic Prompt Caching");
    println!("-------------------------------");
    test_basic_caching(&client).await?;

    // Test 3: Combined Features
    println!("\n🔄 Test 3: Combined Caching + Thinking");
    println!("--------------------------------------");
    test_combined_features(&client).await?;

    println!("\n🎉 All tests completed successfully!");
    println!("✨ Your Prompt caching and Extended thinking implementation is working!");

    Ok(())
}

/// Test basic extended thinking functionality
async fn test_basic_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("🤔 Testing extended thinking with a math problem...");

    let message = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 4096)
            .thinking(2048) // Allow 2048 tokens for thinking
            .system("You are a math tutor. Show your reasoning clearly.")
            .user("If I have 15 apples and give away 1/3 of them, then buy 8 more apples, how many apples do I have? Think through this step by step.")
            .build()
    ).await?;

    println!("📊 Usage stats:");
    println!("  Input tokens: {}", message.usage.input_tokens);
    println!("  Output tokens: {}", message.usage.output_tokens);

    // Check if we got thinking content
    let mut found_thinking = false;
    let mut found_text = false;

    for block in &message.content {
        match block {
            ContentBlock::Thinking { thinking, signature } => {
                found_thinking = true;
                println!("🧠 Found thinking content ({} chars)", thinking.len());
                println!("🔐 Signature: {}...", &signature[..20.min(signature.len())]);

                // Show first few lines of thinking
                let lines: Vec<&str> = thinking.lines().take(3).collect();
                for line in lines {
                    println!("   💭 {line}");
                }
                if thinking.lines().count() > 3 {
                    println!("   💭 ... (and {} more lines)", thinking.lines().count() - 3);
                }
            },
            ContentBlock::Text { text } => {
                found_text = true;
                println!("📝 Final answer: {}", text.chars().take(100).collect::<String>() + "...");
            },
            _ => {}
        }
    }

    if found_thinking {
        println!("✅ Extended thinking is working!");
    } else {
        println!("⚠️  No thinking content found - this might be expected for some models");
    }

    if found_text {
        println!("✅ Text response received!");
    }

    Ok(())
}

/// Test basic prompt caching functionality
async fn test_basic_caching(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("💾 Testing prompt caching with a large system prompt...");

    let large_system_prompt = r#"
You are an expert software architect with extensive experience in:
- Distributed systems design and scalability
- Cloud-native architectures and microservices
- Database design and optimization
- Security best practices and threat modeling
- Performance optimization and monitoring
- DevOps and infrastructure automation
- API design and integration patterns
- Cost optimization strategies

When providing advice, always consider:
1. Scalability and performance implications
2. Security and compliance requirements
3. Operational complexity and maintenance
4. Cost-effectiveness and resource efficiency
5. Team expertise and development velocity
6. Technology ecosystem compatibility
7. Future growth and evolution needs

Provide detailed, actionable recommendations with specific examples.
"#;

    // First request - should create cache
    println!("📤 Making first request (should create cache)...");
    let message1 = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 512)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    large_system_prompt,
                    CacheControl::ephemeral_5m()
                )
            ])
            .user("What's the most important principle in system design?")
            .build()
    ).await?;

    println!("📊 First request usage:");
    println!("  Input tokens: {}", message1.usage.input_tokens);
    println!("  Cache creation tokens: {:?}", message1.usage.cache_creation_input_tokens);
    println!("  Cache read tokens: {:?}", message1.usage.cache_read_input_tokens);

    // Second request - should use cache
    println!("📤 Making second request (should use cache)...");
    let message2 = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 512)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    large_system_prompt,
                    CacheControl::ephemeral_5m()
                )
            ])
            .user("How do you ensure high availability in distributed systems?")
            .build()
    ).await?;

    println!("📊 Second request usage:");
    println!("  Input tokens: {}", message2.usage.input_tokens);
    println!("  Cache creation tokens: {:?}", message2.usage.cache_creation_input_tokens);
    println!("  Cache read tokens: {:?}", message2.usage.cache_read_input_tokens);

    // Verify caching worked
    if message1.usage.cache_creation_input_tokens.is_some() {
        println!("✅ Cache creation detected in first request!");
    }

    if message2.usage.cache_read_input_tokens.is_some() {
        println!("✅ Cache read detected in second request!");
        let cache_savings = message2.usage.cache_read_input_tokens.unwrap();
        println!("💰 Cache saved {cache_savings} tokens on second request");
    } else {
        println!("⚠️  No cache read detected - cache might not be working or expired");
    }

    Ok(())
}

/// Test combined caching and thinking features
async fn test_combined_features(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    println!("🔄 Testing combined caching + thinking...");

    let context = r#"
Project Context: Building a Real-time Chat Application

Requirements:
- Support 10,000 concurrent users
- Message delivery in under 100ms
- End-to-end encryption
- File sharing capabilities
- Mobile and web clients
- 99.9% uptime SLA

Current Architecture:
- Node.js backend with Express
- MongoDB for message storage
- Redis for session management
- WebSocket connections for real-time communication
- AWS deployment with load balancers

Challenges:
- Scaling WebSocket connections
- Message ordering and consistency
- Offline message sync
- Performance under high load
"#;

    let message = client.messages().create_with_extended_cache(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 2048)
            .thinking_config(ThinkingConfig::enabled(1000))
            .system(vec![
                SystemContentBlock::text_with_cache(
                    "You are a senior system architect specializing in real-time applications.",
                    CacheControl::ephemeral_1h()
                )
            ])
            .user(vec![
                ContentBlockParam::text("Given this project context:"),
                ContentBlockParam::text_with_cache(
                    context,
                    CacheControl::ephemeral_1h()
                ),
                ContentBlockParam::text("What are the top 3 architectural improvements you'd recommend? Think through the trade-offs carefully.")
            ])
            .build()
    ).await?;

    println!("📊 Combined features usage:");
    println!("  Total input tokens: {}", message.usage.total_input_tokens());
    println!("  Output tokens: {}", message.usage.output_tokens);
    println!("  Cache creation: {:?}", message.usage.cache_creation_input_tokens);
    println!("  Cache read: {:?}", message.usage.cache_read_input_tokens);

    // Analyze response
    let mut has_thinking = false;
    let mut has_text = false;

    for block in &message.content {
        match block {
            ContentBlock::Thinking { thinking, .. } => {
                has_thinking = true;
                println!("🧠 Thinking detected ({} chars)", thinking.len());
            },
            ContentBlock::Text { text } => {
                has_text = true;
                println!("📝 Response length: {} chars", text.len());
            },
            _ => {}
        }
    }

    if has_thinking && has_text {
        println!("✅ Combined features working perfectly!");
    } else {
        println!("⚠️  Some features might not be fully working");
    }

    // Show cache efficiency
    if let Some(cache_tokens) = message.usage.cache_read_input_tokens {
        let total_input = message.usage.total_input_tokens();
        let cache_percentage = (cache_tokens as f64 / total_input as f64) * 100.0;
        println!("💰 Cache efficiency: {cache_percentage:.1}% of input tokens were cached");
    }

    Ok(())
}

