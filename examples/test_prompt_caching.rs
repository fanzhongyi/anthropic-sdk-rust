//! Prompt Caching Demo
//!
//! This example demonstrates how to use Anthropic's prompt caching feature
//! to reduce costs and latency when working with large prompts that are
//! reused across multiple requests.

use anthropic_sdk::{
    Anthropic,
    types::{
        MessageCreateBuilder,
        SystemContentBlock,
        ContentBlockParam,
        CacheControl,
        Role
    }
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create client from environment variables
    let client = Anthropic::from_env()?;

    // Example 1: Basic prompt caching with system prompt
    println!("=== Example 1: Basic System Prompt Caching ===");
    basic_system_caching(&client).await?;

    // Example 2: Extended cache TTL (1-hour cache)
    println!("\n=== Example 2: Extended Cache TTL (1-hour) ===");
    extended_cache_ttl(&client).await?;

    // Example 3: Caching conversation context
    println!("\n=== Example 3: Conversation Context Caching ===");
    conversation_context_caching(&client).await?;

    // Example 4: Caching large documents
    println!("\n=== Example 4: Large Document Caching ===");
    large_document_caching(&client).await?;

    Ok(())
}

/// Demonstrates basic system prompt caching with 5-minute TTL
async fn basic_system_caching(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let large_system_prompt = r#"
You are an expert software architect with 20+ years of experience in distributed systems,
microservices, cloud architecture, and enterprise software development. You have deep knowledge of:

- System design patterns and best practices
- Scalability and performance optimization
- Security architecture and threat modeling
- Cloud platforms (AWS, GCP, Azure)
- Container orchestration (Kubernetes, Docker)
- Database design and optimization
- API design and integration patterns
- DevOps and CI/CD pipelines
- Monitoring and observability
- Cost optimization strategies

When providing architectural advice, always consider:
1. Scalability requirements
2. Security implications
3. Performance characteristics
4. Operational complexity
5. Cost implications
6. Team expertise and resources
7. Technology ecosystem fit

Provide detailed, actionable recommendations with specific technologies and implementation approaches.
"#;

    // Create message with cached system prompt
    let message = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 1024)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    large_system_prompt,
                    CacheControl::ephemeral()
                )
            ])
            .user("How would you design a scalable e-commerce platform for 1 million users?")
            .build()
    ).await?;

    println!("Response: {}", extract_text_content(&message.content));
    println!("Cache creation tokens: {:?}", message.usage.cache_creation_input_tokens);
    println!("Cache read tokens: {:?}", message.usage.cache_read_input_tokens);

    // Make another request that should hit the cache
    println!("\n--- Making follow-up request (should use cache) ---");

    let follow_up = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 1024)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    large_system_prompt,
                    CacheControl::ephemeral()
                )
            ])
            .user("What about the database architecture for this e-commerce platform?")
            .build()
    ).await?;

    println!("Follow-up response: {}", extract_text_content(&follow_up.content));
    println!("Cache creation tokens: {:?}", follow_up.usage.cache_creation_input_tokens);
    println!("Cache read tokens: {:?}", follow_up.usage.cache_read_input_tokens);

    Ok(())
}

/// Demonstrates extended cache TTL (1-hour cache) for long-running sessions
async fn extended_cache_ttl(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let large_context = r#"
# Project Documentation: Advanced E-commerce Platform

## System Overview
Our e-commerce platform serves millions of users globally with the following key components:

### Architecture Components
1. **Frontend Services**
   - React-based web application
   - React Native mobile apps
   - Progressive Web App (PWA)

2. **API Gateway**
   - Kong API Gateway
   - Rate limiting and throttling
   - Authentication and authorization
   - Request/response transformation

3. **Microservices**
   - User Service (authentication, profiles)
   - Product Service (catalog, inventory)
   - Order Service (shopping cart, checkout)
   - Payment Service (processing, refunds)
   - Notification Service (email, SMS, push)
   - Analytics Service (tracking, reporting)

4. **Data Layer**
   - PostgreSQL (primary data)
   - Redis (caching, sessions)
   - Elasticsearch (search)
   - MongoDB (logs, analytics)

5. **Infrastructure**
   - Kubernetes on AWS EKS
   - Docker containerization
   - Terraform for IaC
   - GitLab CI/CD

### Current Challenges
- Scaling during peak traffic (Black Friday, etc.)
- Cross-service data consistency
- Real-time inventory management
- Personalization at scale
- International expansion requirements

### Performance Metrics
- 99.9% uptime SLA
- <200ms API response times
- <3s page load times
- Support for 10,000 concurrent users
"#;

    // Use extended cache TTL for long-running analysis sessions
    let message = client.messages().create_with_extended_cache(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 1024)
            .system(vec![
                SystemContentBlock::text_with_cache(
                    "You are a senior technical consultant analyzing an e-commerce platform.",
                    CacheControl::ephemeral()
                )
            ])
            .user(vec![
                ContentBlockParam::text_with_cache(
                    large_context,
                    CacheControl::ephemeral()
                ),
                ContentBlockParam::text("Based on this documentation, what are the top 3 scalability bottlenecks you identify?")
            ])
            .build()
    ).await?;

    println!("Analysis: {}", extract_text_content(&message.content));
    println!("Cache creation tokens: {:?}", message.usage.cache_creation_input_tokens);
    println!("Total input tokens: {}", message.usage.total_input_tokens());

    Ok(())
}

/// Demonstrates caching conversation context for multi-turn conversations
async fn conversation_context_caching(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    // Simulate a conversation with cached context
    let conversation_history = [("user", "I'm building a new SaaS application for project management. Can you help me design the architecture?"),
        ("assistant", "I'd be happy to help you design a SaaS project management application! Let me break this down into key architectural components:\n\n## Core Architecture\n\n1. **Frontend Layer**\n   - React or Vue.js SPA\n   - Mobile apps (React Native or Flutter)\n   - Real-time updates via WebSockets\n\n2. **API Layer**\n   - RESTful APIs with GraphQL for complex queries\n   - Authentication/authorization (JWT + OAuth)\n   - Rate limiting and API versioning\n\n3. **Business Logic**\n   - Microservices for different domains:\n     - User management\n     - Project management\n     - Task management\n     - Collaboration features\n     - Reporting and analytics\n\n4. **Data Layer**\n   - PostgreSQL for relational data\n   - Redis for caching and sessions\n   - File storage (AWS S3 or similar)\n\nWhat specific aspects would you like me to elaborate on?"),
        ("user", "Let's focus on the real-time collaboration features. How would you implement real-time updates for multiple users working on the same project?"),
        ("assistant", "Excellent question! Real-time collaboration is crucial for project management tools. Here's how I'd implement it:\n\n## Real-time Architecture\n\n### 1. WebSocket Infrastructure\n```\nClient ←→ Load Balancer ←→ WebSocket Servers ←→ Message Broker ←→ Database\n```\n\n### 2. Key Components\n\n**WebSocket Connection Management**\n- Use Socket.io or native WebSockets\n- Connection pooling and load balancing\n- Automatic reconnection with exponential backoff\n- Room-based subscriptions (project-level, task-level)\n\n**Message Broker (Redis Pub/Sub or Apache Kafka)**\n- Handles message routing between servers\n- Ensures scalability across multiple WebSocket instances\n- Message persistence for offline users\n\n**Event Types**\n- Task updates (status, assignee, description)\n- Comment additions\n- File uploads/changes\n- User presence (who's online, viewing what)\n- Cursor positions for collaborative editing\n\n### 3. Implementation Strategy\n\n**Operational Transform (OT) or Conflict-Free Replicated Data Types (CRDTs)**\n- For simultaneous edits on the same content\n- Ensures consistency without locking\n\n**Optimistic Updates**\n- Apply changes immediately on client\n- Rollback if server rejects\n- Show pending state during sync\n\nWould you like me to dive deeper into any of these aspects?")];

    // Build conversation with cached context
    let mut builder = MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 1024);

    // Add conversation history with caching for the lengthy parts
    for (role, content) in conversation_history.iter() {
        let role = match *role {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => continue,
        };

        if content.len() > 500 {
            // Cache longer messages
            builder = builder.message(role, vec![
                ContentBlockParam::text_with_cache(
                    content.to_string(),
                    CacheControl::ephemeral()
                )
            ]);
        } else {
            builder = builder.message(role, content.to_string());
        }
    }

    // Add new user question
    builder = builder.user("Now I want to add file collaboration features. How would you implement real-time collaborative document editing?");

    let message = client.messages().create(builder.build()).await?;

    println!("Collaborative editing response: {}", extract_text_content(&message.content));
    println!("Cache read tokens: {:?}", message.usage.cache_read_input_tokens);

    Ok(())
}

/// Demonstrates caching large documents for analysis
async fn large_document_caching(client: &Anthropic) -> Result<(), Box<dyn Error>> {
    let large_document = include_str!("../README.md"); // Use the project README as example

    // First request - analyze the document
    let analysis = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 1024)
            .system("You are a technical documentation expert. Analyze documents for clarity, completeness, and areas for improvement.")
            .user(vec![
                ContentBlockParam::text("Please analyze this README file:"),
                ContentBlockParam::text_with_cache(
                    large_document,
                    CacheControl::ephemeral()
                )
            ])
            .build()
    ).await?;

    println!("Document analysis: {}", extract_text_content(&analysis.content));
    println!("Cache creation tokens: {:?}", analysis.usage.cache_creation_input_tokens);

    // Follow-up request using cached document
    let suggestions = client.messages().create(
        MessageCreateBuilder::new("claude-3-7-sonnet@20250219", 1024)
            .system("You are a technical documentation expert. Analyze documents for clarity, completeness, and areas for improvement.")
            .user(vec![
                ContentBlockParam::text("Based on the same README file:"),
                ContentBlockParam::text_with_cache(
                    large_document,
                    CacheControl::ephemeral()
                ),
                ContentBlockParam::text("What specific improvements would you suggest for the installation section?")
            ])
            .build()
    ).await?;

    println!("\nInstallation improvements: {}", extract_text_content(&suggestions.content));
    println!("Cache read tokens: {:?}", suggestions.usage.cache_read_input_tokens);

    Ok(())
}

/// Helper function to extract text content from message content blocks
fn extract_text_content(content: &[anthropic_sdk::types::ContentBlock]) -> String {
    content.iter()
        .filter_map(|block| match block {
            anthropic_sdk::types::ContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join("\n")
}
