//! Combined Prompt Caching and Extended Thinking Demo
//!
//! This example demonstrates how to effectively combine prompt caching with
//! extended thinking for optimal performance and reasoning quality.

use anthropic_sdk::{
    types::{
        CacheControl, ContentBlock, ContentBlockParam, MessageCreateBuilder, Role,
        SystemContentBlock, ThinkingConfig,
    },
    Anthropic,
};
use futures::StreamExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    // Example 1: Cached context with thinking for analysis
    println!("=== Example 1: Cached Context + Thinking Analysis ===");
    cached_context_with_thinking(&client).await?;

    // Example 2: Streaming with both features
    println!("\n=== Example 2: Streaming with Cache + Thinking ===");
    streaming_cache_and_thinking(&client).await?;

    // Example 3: Multi-turn conversation with persistent cache
    println!("\n=== Example 3: Multi-turn Cached Conversation ===");
    multi_turn_cached_conversation(&client).await?;

    Ok(())
}

/// Demonstrates using cached large context with extended thinking for deep analysis
async fn cached_context_with_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let large_codebase_context = r#"
// Main application structure
class ECommerceApp {
    constructor() {
        this.userService = new UserService();
        this.productService = new ProductService();
        this.orderService = new OrderService();
        this.paymentService = new PaymentService();
        this.notificationService = new NotificationService();
    }

    async processOrder(userId, items, paymentInfo) {
        try {
            // Validate user
            const user = await this.userService.getUser(userId);
            if (!user) throw new Error('User not found');

            // Check inventory
            for (const item of items) {
                const product = await this.productService.getProduct(item.productId);
                if (!product || product.stock < item.quantity) {
                    throw new Error(`Insufficient stock for ${item.productId}`);
                }
            }

            // Calculate total
            let total = 0;
            for (const item of items) {
                const product = await this.productService.getProduct(item.productId);
                total += product.price * item.quantity;
            }

            // Process payment
            const paymentResult = await this.paymentService.processPayment(
                paymentInfo, total
            );

            if (!paymentResult.success) {
                throw new Error('Payment failed');
            }

            // Create order
            const order = await this.orderService.createOrder({
                userId,
                items,
                total,
                paymentId: paymentResult.id
            });

            // Update inventory
            for (const item of items) {
                await this.productService.updateStock(
                    item.productId,
                    -item.quantity
                );
            }

            // Send confirmation
            await this.notificationService.sendOrderConfirmation(
                user.email,
                order
            );

            return order;

        } catch (error) {
            console.error('Order processing failed:', error);
            throw error;
        }
    }
}

class UserService {
    async getUser(userId) {
        // Database query simulation
        return await db.query('SELECT * FROM users WHERE id = ?', [userId]);
    }
}

class ProductService {
    async getProduct(productId) {
        return await db.query('SELECT * FROM products WHERE id = ?', [productId]);
    }

    async updateStock(productId, delta) {
        return await db.query(
            'UPDATE products SET stock = stock + ? WHERE id = ?',
            [delta, productId]
        );
    }
}

class OrderService {
    async createOrder(orderData) {
        const orderId = await db.query(
            'INSERT INTO orders (user_id, total, payment_id) VALUES (?, ?, ?)',
            [orderData.userId, orderData.total, orderData.paymentId]
        );

        // Insert order items
        for (const item of orderData.items) {
            await db.query(
                'INSERT INTO order_items (order_id, product_id, quantity) VALUES (?, ?, ?)',
                [orderId, item.productId, item.quantity]
            );
        }

        return { id: orderId, ...orderData };
    }
}

class PaymentService {
    async processPayment(paymentInfo, amount) {
        // Simulate payment processing
        if (Math.random() > 0.1) { // 90% success rate
            return { success: true, id: 'pay_' + Date.now() };
        } else {
            return { success: false, error: 'Payment declined' };
        }
    }
}

class NotificationService {
    async sendOrderConfirmation(email, order) {
        console.log(`Sending confirmation to ${email} for order ${order.id}`);
        // Email sending logic here
    }
}
"#;

    // Use extended cache TTL and thinking for thorough analysis
    let message = client.messages().create(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 9000)
            .thinking_config(ThinkingConfig::enabled(2000))
            .system(vec![
                SystemContentBlock::text_with_cache(
                    "You are a senior software architect and security expert. Analyze code thoroughly for architecture, security, performance, and maintainability issues.",
                    CacheControl::ephemeral()
                )
            ])
            .user(vec![
                ContentBlockParam::text("Please analyze this e-commerce order processing code:"),
                ContentBlockParam::text_with_cache(
                    large_codebase_context,
                    CacheControl::ephemeral()
                ),
                ContentBlockParam::text("Focus on: 1) Race conditions 2) Error handling 3) Security vulnerabilities 4) Performance bottlenecks")
            ])
            .build()
    ).await?;

    println!("=== Deep Code Analysis ===");

    for block in &message.content {
        match block {
            ContentBlock::Thinking { thinking, .. } => {
                println!("🧠 Analysis Process:");
                // Show key thinking points
                let lines: Vec<&str> = thinking.lines().collect();
                for (i, line) in lines.iter().take(20).enumerate() {
                    // Show first 20 lines
                    if !line.trim().is_empty() {
                        println!("  {}: {}", i + 1, line);
                    }
                }
                if lines.len() > 20 {
                    println!("  ... ({} more lines of analysis)", lines.len() - 20);
                }
                println!();
            }
            ContentBlock::Text { text } => {
                println!("📋 Final Analysis Report:");
                println!("{text}");
            }
            _ => {}
        }
    }

    println!("\n💾 Performance Metrics:");
    println!(
        "  Cache creation tokens: {:?}",
        message.usage.cache_creation_input_tokens
    );
    println!(
        "  Cache read tokens: {:?}",
        message.usage.cache_read_input_tokens
    );
    println!(
        "  Total input tokens: {}",
        message.usage.total_input_tokens()
    );
    println!("  Output tokens: {}", message.usage.output_tokens);

    Ok(())
}

/// Demonstrates streaming with both caching and thinking for real-time analysis
async fn streaming_cache_and_thinking(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let system_prompt = "You are an expert system designer. Think through complex problems step by step, considering multiple approaches and trade-offs.";
    let problem_context = r#"
Design Requirements for Real-time Analytics Platform:

Business Context:
- Financial trading company processing 10M+ transactions/day
- Need real-time fraud detection (< 100ms response time)
- Regulatory compliance requirements (SOX, PCI-DSS)
- Global operations across 15+ countries
- 24/7 uptime requirement (99.99% SLA)

Technical Constraints:
- Legacy mainframe systems for core banking
- Hybrid cloud infrastructure (AWS + on-premise)
- Data sovereignty requirements in EU/Asia
- Budget: $2M annually for infrastructure
- Team: 12 engineers (mixed seniority)

Current Pain Points:
- Batch processing causing 4-hour delays in fraud detection
- Manual compliance reporting taking 2 weeks/month
- System downtime during peak trading hours
- Inconsistent data quality across regions
- Scaling issues during market volatility
"#;

    let stream = client.messages().create_stream(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 4096)
            .thinking_config(ThinkingConfig::enabled(2048))
            .system(vec![
                SystemContentBlock::text_with_cache(
                    system_prompt,
                    CacheControl::ephemeral()
                )
            ])
            .user(vec![
                ContentBlockParam::text_with_cache(
                    problem_context,
                    CacheControl::ephemeral()
                ),
                ContentBlockParam::text("Design a comprehensive real-time analytics architecture that addresses all these requirements. Include specific technology choices, data flow, and implementation phases.")
            ])
            .stream(true)
            .build()
    ).await?;

    println!("=== Streaming Architecture Design ===");

    let mut thinking_buffer = String::new();
    let mut response_buffer = String::new();
    let mut thinking_complete = false;

    tokio::pin!(stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                use anthropic_sdk::types::streaming::MessageStreamEvent;
                match event {
                    MessageStreamEvent::ContentBlockStart { content_block, .. } => {
                        match content_block {
                            ContentBlock::Thinking { .. } => {
                                println!("🤔 Analyzing requirements and designing solution...");
                            }
                            ContentBlock::Text { .. } => {
                                if !thinking_complete && !thinking_buffer.is_empty() {
                                    println!("\n💭 Key Design Considerations:");
                                    // Show thinking summary
                                    let thinking_lines: Vec<&str> =
                                        thinking_buffer.lines().collect();
                                    for line in thinking_lines.iter().take(10) {
                                        if line.contains("approach")
                                            || line.contains("consider")
                                            || line.contains("trade-off")
                                        {
                                            println!("  • {}", line.trim());
                                        }
                                    }
                                    thinking_complete = true;
                                    println!("\n🏗️ Detailed Architecture Design:");
                                }
                            }
                            _ => {}
                        }
                    }
                    MessageStreamEvent::ContentBlockDelta { delta, .. } => {
                        use anthropic_sdk::types::streaming::ContentBlockDelta;
                        match delta {
                            ContentBlockDelta::ThinkingDelta { thinking } => {
                                thinking_buffer.push_str(&thinking);
                                // Show progress indicator
                                if thinking_buffer.len() % 200 == 0 {
                                    print!(".");
                                }
                            }
                            ContentBlockDelta::TextDelta { text } => {
                                response_buffer.push_str(&text);
                                print!("{text}");
                            }
                            _ => {}
                        }
                    }
                    MessageStreamEvent::MessageDelta { usage, .. } => {
                        // Show real-time usage updates
                        if usage.cache_read_input_tokens.is_some()
                            || usage.cache_creation_input_tokens.is_some()
                        {
                            println!(
                                "\n📊 Cache Usage - Read: {:?}, Creation: {:?}",
                                usage.cache_read_input_tokens, usage.cache_creation_input_tokens
                            );
                        }
                    }
                    MessageStreamEvent::MessageStop => {
                        println!("\n\n✅ Architecture design complete!");
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

/// Demonstrates multi-turn conversation with persistent caching
async fn multi_turn_cached_conversation(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let expert_context = r#"
You are Dr. Sarah Chen, a senior cloud architect with 15+ years of experience at major tech companies.
Your expertise includes:
- Distributed systems design and implementation
- Cloud-native architectures (AWS, GCP, Azure)
- Microservices and containerization
- DevOps and infrastructure automation
- Performance optimization and scalability
- Security and compliance in cloud environments

You provide practical, experienced-based advice with specific examples and best practices.
"#;

    println!("=== Multi-turn Conversation with Cached Context ===");

    // Turn 1: Initial question with cached context
    println!(
        "\n👤 User: How do I design a microservices architecture for a large e-commerce platform?"
    );

    let response1 = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 4096)
                .thinking(2048)
                .system(vec![SystemContentBlock::text_with_cache(
                    expert_context,
                    CacheControl::ephemeral(),
                )])
                .user(
                    "How do I design a microservices architecture for a large e-commerce platform?",
                )
                .build(),
        )
        .await?;

    let response1_text = extract_text_content(&response1.content);
    println!("🤖 Dr. Chen: {response1_text}");
    println!(
        "💾 Cache creation: {:?} tokens",
        response1.usage.cache_creation_input_tokens
    );

    // Turn 2: Follow-up question (should use cache)
    println!("\n👤 User: What about data consistency between microservices?");

    let response2 = client
        .messages()
        .create(
            MessageCreateBuilder::new("claude-sonnet-4@20250514", 4096)
                .thinking(2048)
                .system(vec![SystemContentBlock::text_with_cache(
                    expert_context,
                    CacheControl::ephemeral(),
                )])
                .message(
                    Role::User,
                    "How do I design a microservices architecture for a large e-commerce platform?",
                )
                .message(Role::Assistant, response1_text.clone())
                .user("What about data consistency between microservices?")
                .build(),
        )
        .await?;

    let response2_text = extract_text_content(&response2.content);
    println!("🤖 Dr. Chen: {response2_text}");
    println!(
        "💾 Cache read: {:?} tokens",
        response2.usage.cache_read_input_tokens
    );

    // Turn 3: Technical deep-dive
    println!("\n👤 User: Can you show me a specific example of implementing the Saga pattern?");

    let response3 = client.messages().create(
        MessageCreateBuilder::new("claude-sonnet-4@20250514", 3072)
            .thinking(2048)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    expert_context,
                    CacheControl::ephemeral()
                )
            ])
            .message(Role::User, "How do I design a microservices architecture for a large e-commerce platform?")
            .message(Role::Assistant, response1_text)
            .message(Role::User, "What about data consistency between microservices?")
            .message(Role::Assistant, response2_text)
            .user("Can you show me a specific example of implementing the Saga pattern for order processing?")
            .build()
    ).await?;

    let response3_text = extract_text_content(&response3.content);
    println!("🤖 Dr. Chen: {response3_text}");
    println!(
        "💾 Cache read: {:?} tokens",
        response3.usage.cache_read_input_tokens
    );

    // Show total conversation efficiency
    let total_cache_savings = response2.usage.cache_read_input_tokens.unwrap_or(0)
        + response3.usage.cache_read_input_tokens.unwrap_or(0);
    println!("\n📈 Conversation Efficiency:");
    println!("  Total cache read tokens: {total_cache_savings}");
    println!(
        "  Estimated cost savings: ~{:.2}%",
        (total_cache_savings as f64
            / (response1.usage.input_tokens
                + response2.usage.input_tokens
                + response3.usage.input_tokens) as f64)
            * 100.0
    );

    Ok(())
}

/// Helper function to extract text content from message content blocks
fn extract_text_content(content: &[ContentBlock]) -> String {
    content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join("\n")
}
