//! Extended Thinking Demo
//!
//! This example demonstrates how to use Anthropic's extended thinking feature
//! to get more detailed reasoning from Claude models. Extended thinking allows
//! Claude to "think out loud" before providing responses, leading to more
//! thoughtful and well-reasoned answers.

use anthropic_sdk::{
    types::{ContentBlock, MessageCreateBuilder, ThinkingConfig},
    Anthropic,
};
use futures::StreamExt;
use std::error::Error;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create client from environment variables
    let client = Anthropic::from_env()?;

    // Example 1: Basic extended thinking
    println!("=== Example 1: Basic Extended Thinking ===");
    basic_extended_thinking(&client).await?;

    // Example 2: Streaming with extended thinking
    println!("\n=== Example 2: Streaming Extended Thinking ===");
    streaming_extended_thinking(&client).await?;

    // Example 3: Complex problem solving with thinking
    println!("\n=== Example 3: Complex Problem Solving ===");
    complex_problem_solving(&client).await?;

    // Example 4: Code review with detailed thinking
    println!("\n=== Example 4: Code Review with Thinking ===");
    code_review_with_thinking(&client).await?;

    Ok(())
}

/// Demonstrates basic extended thinking for mathematical problem solving
async fn basic_extended_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let message = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 8192)
                .thinking(2048) // thinking tokens must be less than max_tokens
                .system("You are a mathematics tutor. Show your reasoning process clearly.")
                .user(
                    r#"
A company has 120 employees. Due to budget constraints, they need to reduce their workforce by 15%.
However, they also plan to hire 8 new employees in specialized roles.
After these changes, they want to give all remaining employees a bonus.
If the total bonus budget is $50,000, how much bonus will each employee receive?
Please show your step-by-step reasoning.
"#,
                )
                .build(),
        )
        .await?;

    println!("=== Response ===");
    for block in &message.content {
        match block {
            ContentBlock::Thinking {
                thinking,
                signature,
            } => {
                println!("🤔 Thinking process:");
                println!("{thinking}");
                println!("Signature: {signature}");
                println!();
            }
            ContentBlock::Text { text } => {
                println!("📝 Final answer:");
                println!("{text}");
            }
            _ => {}
        }
    }

    println!(
        "Usage - Input tokens: {}, Output tokens: {}",
        message.usage.input_tokens, message.usage.output_tokens
    );

    Ok(())
}

/// Demonstrates streaming extended thinking for real-time reasoning display
async fn streaming_extended_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let stream = client
        .messages()
        .create_stream(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 3000)
                .thinking(2000) // thinking tokens must be less than max_tokens
                .system(
                    "You are an expert system architect. Think through problems systematically.",
                )
                .user(
                    r#"
Design a distributed caching system that can handle 1 million requests per second
with 99.9% availability. Consider:
- Cache consistency models
- Partitioning strategies
- Failure recovery mechanisms
- Cost optimization
- Geographic distribution

Walk through your design decisions step by step.
"#,
                )
                .stream(true)
                .build(),
        )
        .await?;

    println!("=== Streaming Response ===");

    let mut thinking_content = String::new();
    let mut response_content = String::new();
    let mut _in_thinking = false;

    // Process the stream
    tokio::pin!(stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                use anthropic_sdk::types::streaming::MessageStreamEvent;
                match event {
                    MessageStreamEvent::ContentBlockStart {
                        content_block,
                        index: _,
                    } => match content_block {
                        ContentBlock::Thinking { .. } => {
                            println!("🤔 Starting to think...");
                            _in_thinking = true;
                        }
                        ContentBlock::Text { .. } => {
                            if !thinking_content.is_empty() {
                                println!("\n💭 Thinking complete. Final reasoning:");
                                println!("{thinking_content}");
                                println!("\n📝 Response:");
                            }
                            _in_thinking = false;
                        }
                        _ => {}
                    },
                    MessageStreamEvent::ContentBlockDelta { delta, index: _ } => {
                        use anthropic_sdk::types::streaming::ContentBlockDelta;
                        match delta {
                            ContentBlockDelta::ThinkingDelta { thinking } => {
                                thinking_content.push_str(&thinking);
                                // Show thinking progress in real-time (abbreviated)
                                print!(".");
                                io::stdout().flush().unwrap(); // Ensure immediate output
                                if thinking_content.len() % 100 == 0 {
                                    println!(" (thinking...)");
                                    io::stdout().flush().unwrap();
                                }
                            }
                            ContentBlockDelta::TextDelta { text } => {
                                response_content.push_str(&text);
                                print!("{text}");
                                io::stdout().flush().unwrap(); // Ensure immediate output
                            }
                            _ => {
                                println!("🔍 Unknown delta type: {delta:?}");
                            }
                        }
                    }
                    MessageStreamEvent::MessageStop => {
                        println!("\n\n=== Stream Complete ===");
                        break;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Stream error: {e}");
                break;
            }
        }
    }

    Ok(())
}

/// Demonstrates extended thinking for complex algorithmic problem solving
async fn complex_problem_solving(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let message = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 15000)
                .thinking_config(ThinkingConfig::enabled(10000)) // Use explicit config
                .system(
                    r#"
You are an expert algorithm designer and computer scientist.
When solving problems, think through multiple approaches, analyze their trade-offs,
and choose the best solution with clear reasoning.
"#,
                )
                .user(
                    r#"
Design an algorithm to find the shortest path between any two nodes in a dynamic graph
where edges can be added or removed frequently. The graph has the following constraints:

- Up to 100,000 nodes
- Up to 1,000,000 edges
- Edge weights are positive integers (1-1000)
- About 1000 edge updates per second
- About 100 shortest path queries per second
- Memory usage should be reasonable (< 10GB)

Provide a detailed algorithm design with:
1. Data structures used
2. Time complexity analysis
3. Space complexity analysis
4. Handling of dynamic updates
5. Query processing strategy
6. Alternative approaches considered

Include pseudocode for the main operations.
"#,
                )
                .build(),
        )
        .await?;

    println!("=== Complex Algorithm Design ===");

    for block in message.content.iter() {
        match block {
            ContentBlock::Thinking {
                thinking,
                signature,
            } => {
                println!("🧠 Detailed Reasoning Process:");
                println!("{thinking}");
                println!("\n🔐 Verification signature: {signature}");
                println!("\n{}\n", "=".repeat(80));
            }
            ContentBlock::Text { text } => {
                println!("🎯 Final Algorithm Design:");
                println!("{text}");
            }
            _ => {}
        }
    }

    println!("\n📊 Token Usage:");
    println!("  Input tokens: {}", message.usage.input_tokens);
    println!("  Output tokens: {}", message.usage.output_tokens);
    println!("  Total tokens: {}", message.usage.total_tokens());

    Ok(())
}

/// Demonstrates extended thinking for code review with detailed analysis
async fn code_review_with_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let code_to_review = r#"
```python
class DatabaseManager:
    def __init__(self, connection_string):
        self.connection_string = connection_string
        self.connection = None
        self.cache = {}

    def connect(self):
        if not self.connection:
            self.connection = sqlite3.connect(self.connection_string)
        return self.connection

    def execute_query(self, query, params=None):
        conn = self.connect()
        cursor = conn.cursor()

        # Check cache first
        cache_key = f"{query}_{params}"
        if cache_key in self.cache:
            return self.cache[cache_key]

        if params:
            cursor.execute(query, params)
        else:
            cursor.execute(query)

        result = cursor.fetchall()

        # Cache the result
        self.cache[cache_key] = result

        return result

    def insert_user(self, name, email):
        query = f"INSERT INTO users (name, email) VALUES ('{name}', '{email}')"
        return self.execute_query(query)

    def get_user_by_id(self, user_id):
        query = "SELECT * FROM users WHERE id = ?"
        return self.execute_query(query, (user_id,))

    def update_user_email(self, user_id, new_email):
        query = f"UPDATE users SET email = '{new_email}' WHERE id = {user_id}"
        return self.execute_query(query)
```
"#;

    let message = client.messages().create(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 15000)
            .thinking(2000) // thinking tokens must be less than max_tokens
            .system(r#"
You are a senior software engineer conducting a thorough code review.
Analyze the code for:
- Security vulnerabilities
- Performance issues
- Best practices violations
- Design patterns
- Error handling
- Code maintainability
- Potential bugs

Think through each aspect systematically before providing your review.
"#)
            .user(format!("Please review this Python database manager class:\n\n{code_to_review}\n\nProvide a comprehensive analysis with specific recommendations for improvement."))
            .build()
    ).await?;

    println!("=== Code Review with Extended Thinking ===");

    for block in &message.content {
        match block {
            ContentBlock::Thinking {
                thinking,
                signature,
            } => {
                println!("🔍 Review Analysis Process:");

                // Split thinking into sections for better readability
                let sections: Vec<&str> = thinking.split("\n\n").collect();
                for (i, section) in sections.iter().enumerate() {
                    if !section.trim().is_empty() {
                        println!("{}. {}", i + 1, section.trim());
                        println!();
                    }
                }

                println!("🔐 Analysis signature: {signature}");
                println!("\n{}\n", "=".repeat(80));
            }
            ContentBlock::Text { text } => {
                println!("📋 Final Code Review Report:");
                println!("{text}");
            }
            _ => {}
        }
    }

    // Show cache usage if available
    if let Some(cache_read) = message.usage.cache_read_input_tokens {
        println!("\n💾 Cache Performance:");
        println!("  Cache read tokens: {cache_read}");
        if let Some(cache_creation) = message.usage.cache_creation_input_tokens {
            println!("  Cache creation tokens: {cache_creation}");
        }
    }

    Ok(())
}
