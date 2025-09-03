use anthropic_sdk::{
    default_retry, AnthropicError, RetryCondition, RetryExecutor, RetryPolicy, RetryResult,
    TokenCounter,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Phase 4.4: Enhanced Infrastructure Demo");
    println!("===========================================\n");

    // Token Counting System Demo
    println!("📊 Token Counting System");
    println!("-----------------------");

    // Create a token counter
    let counter = TokenCounter::new();

    // Estimate cost before making request
    let estimated_cost = counter.estimate_cost("claude-3-5-sonnet-latest", 1000, 500);
    println!(
        "💰 Estimated cost for 1000 input + 500 output tokens: ${:.4}",
        estimated_cost
    );

    // Simulate some usage for demonstration
    let usage1 = anthropic_sdk::types::Usage {
        input_tokens: 1200,
        output_tokens: 800,
        cache_creation_input_tokens: Some(50),
        cache_read_input_tokens: Some(100),
        cache_creation: None,
        server_tool_use: None,
        service_tier: None,
    };

    let cost_breakdown = counter.record_usage("claude-3-5-sonnet-latest", &usage1);
    println!("\n📈 Cost Breakdown:");
    println!("{}", cost_breakdown);

    // Retry System Demo
    println!("\n\n🔄 Retry System");
    println!("---------------");

    // Create different retry policies
    let basic_retry = default_retry();
    println!(
        "🔧 Basic retry policy: {} retries, {}ms initial delay",
        basic_retry.get_policy().max_retries,
        basic_retry.get_policy().initial_delay.as_millis()
    );

    let custom_policy = RetryPolicy::exponential()
        .max_retries(5)
        .initial_delay(Duration::from_millis(200))
        .max_delay(Duration::from_secs(10))
        .multiplier(1.5)
        .jitter(true)
        .retry_conditions(vec![
            RetryCondition::RateLimit,
            RetryCondition::ServerError,
            RetryCondition::Timeout,
        ]);

    let custom_executor = RetryExecutor::new(custom_policy);

    // Simulate retry scenarios
    println!("\n🎭 Retry Scenarios:");

    // Scenario 1: Success on first try
    let result1 = custom_executor
        .execute(|| async { Ok::<String, AnthropicError>("Success!".to_string()) })
        .await;

    match result1 {
        RetryResult::Success(value) => println!("  ✅ {}", value),
        RetryResult::Failed(error) => println!("  ❌ {}", error),
    }

    // Get usage summary
    let summary = counter.get_summary();
    println!("\n📊 Usage Summary:");
    println!("{}", summary);

    println!("\n✨ Phase 4.4 Infrastructure Demo Complete!");
    println!("🚀 Token counting and retry system working perfectly!");

    Ok(())
}
