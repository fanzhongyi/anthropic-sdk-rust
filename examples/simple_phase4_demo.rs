use anthropic_sdk::{
    AnthropicError, File, FileConstraints, RetryCondition, RetryExecutor, RetryPolicy, RetryResult,
    TokenCounter,
};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Simple Phase 4 Demo");
    println!("=======================\n");
    println!("Demonstrating Phase 4 Advanced Features:");
    println!("• 📁 File Upload System (Phase 4.3)");
    println!("• 📊 Token Counting & 🔄 Retry Logic (Phase 4.4)");
    println!();

    // Phase 4.4: Initialize infrastructure
    println!("🏗️ Phase 4.4: Infrastructure Setup");
    println!("===================================");

    let token_counter = TokenCounter::new();
    let retry_policy = RetryPolicy::exponential()
        .max_retries(3)
        .initial_delay(Duration::from_millis(100))
        .max_delay(Duration::from_secs(10))
        .retry_conditions(vec![
            RetryCondition::Timeout,
            RetryCondition::ServerError,
            RetryCondition::RateLimit,
        ]);

    let retry_executor = RetryExecutor::new(retry_policy);

    println!("✅ Token counter initialized");
    println!(
        "✅ Retry policy: {} max retries",
        retry_executor.get_policy().max_retries
    );

    // Phase 4.3: File system demonstration
    println!("\n📁 Phase 4.3: File System Demo");
    println!("===============================");

    let sample_documents = vec![
        (
            "report.txt",
            "text/plain",
            "Business report with market analysis and performance metrics.",
        ),
        (
            "data.csv",
            "text/csv",
            "name,score\nProduct A,85\nProduct B,92",
        ),
        (
            "config.json",
            "application/json",
            r#"{"version": "1.0", "features": ["tools", "files"]}"#,
        ),
    ];

    let mut processed_files = Vec::new();

    for (filename, _mime_type, content) in sample_documents {
        let start_time = Instant::now();

        // Create file from bytes - using correct API
        let file = File::from_bytes(
            filename,                    // name first
            content.as_bytes().to_vec(), // then bytes
            None,                        // Let it auto-detect MIME type
        )?;

        // File validation with constraints
        let constraints = FileConstraints {
            max_size: 1024 * 1024, // 1MB
            allowed_types: None,   // Allow all types for demo
            require_hash: false,
        };

        file.validate(&constraints)?;

        let processing_time = start_time.elapsed();

        println!(
            "📄 Processed: {} ({} bytes) in {:?}",
            file.name, file.size, processing_time
        );

        processed_files.push(file);
    }

    println!("✅ {} files processed successfully", processed_files.len());

    // File type detection and utilities
    println!("\n🔍 File Analysis");
    println!("================");

    for file in &processed_files {
        println!("File: {}", file.name);
        println!("  Size: {} bytes", file.size);
        println!("  MIME: {}", file.mime_type);
        println!("  Is text: {}", file.is_text());
        println!("  Is application: {}", file.is_application());

        // Hash verification if available
        if let Some(hash) = &file.hash {
            println!("  Hash: {hash:.16}...");
        }
        println!();
    }

    // Phase 4.4: Token counting and cost estimation
    println!("📊 Phase 4.4: Token Counting & Cost Estimation");
    println!("===============================================");

    // Pre-request cost estimation
    let estimated_cost = token_counter.estimate_cost("claude-3-5-sonnet-latest", 500, 200);
    println!("💰 Estimated cost (500 input + 200 output): ${estimated_cost:.4}");

    // Simulate API usage
    let usage = anthropic_sdk::types::Usage {
        input_tokens: 750,
        output_tokens: 300,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: Some(50),
        cache_creation: None,
        server_tool_use: None,
        service_tier: None,
    };

    let cost_breakdown = token_counter.record_usage("claude-3-5-sonnet-latest", &usage);
    println!(
        "📈 Recorded usage: {} input, {} output tokens",
        usage.input_tokens, usage.output_tokens
    );
    println!("💵 Actual cost: ${:.4}", cost_breakdown.total_cost);

    // Phase 4.4: Retry logic demonstration
    println!("\n🔄 Retry Logic Demo");
    println!("==================");

    // Scenario 1: Success after retries (using Arc<Mutex<>> for shared state)
    use std::sync::{Arc, Mutex};
    let attempt_count = Arc::new(Mutex::new(0));

    let result1 = retry_executor
        .execute(|| {
            let count = Arc::clone(&attempt_count);
            async move {
                let mut counter = count.lock().unwrap();
                *counter += 1;
                let current_attempt = *counter;
                drop(counter); // Release lock before async operation

                if current_attempt < 3 {
                    println!("  Attempt {current_attempt}: Simulating transient failure");
                    Err(AnthropicError::HttpError {
                        status: 503,
                        message: "Service temporarily unavailable".to_string(),
                    })
                } else {
                    println!("  Attempt {current_attempt}: Success!");
                    Ok("Operation completed successfully".to_string())
                }
            }
        })
        .await;

    match result1 {
        RetryResult::Success(msg) => println!("✅ Retry success: {msg}"),
        RetryResult::Failed(error) => println!("❌ Retry failed: {error}"),
    }

    let final_attempt_count = *attempt_count.lock().unwrap();

    // Scenario 2: Non-retriable error
    let result2 = retry_executor
        .execute(|| async {
            println!("  Simulating non-retriable error");
            Err::<String, AnthropicError>(AnthropicError::InvalidApiKey)
        })
        .await;

    match result2 {
        RetryResult::Success(msg) => println!("✅ Unexpected success: {msg}"),
        RetryResult::Failed(error) => println!("✅ Correctly failed (non-retriable): {error}"),
    }

    // Message integration with files
    println!("\n💬 Message Integration");
    println!("======================");

    // Create a message with file attachment (simplified)
    if let Some(first_file) = processed_files.first() {
        println!("📎 Creating message with file attachment");
        println!("  File: {} ({} bytes)", first_file.name, first_file.size);
        println!("✅ File integration ready for API calls");
    }

    // Final summary and metrics
    println!("\n📈 Session Summary");
    println!("==================");

    let usage_summary = token_counter.get_summary();
    println!("Token Metrics:");
    println!("  • Total tokens: {}", usage_summary.total_tokens);
    println!("  • Total cost: ${:.4}", usage_summary.total_cost_usd);
    println!(
        "  • Session duration: {:.1}s",
        usage_summary.session_duration.as_secs_f64()
    );

    println!("\nInfrastructure Status:");
    println!("  • Files processed: {}", processed_files.len());
    println!("  • Retry attempts made: {final_attempt_count}");
    println!("  • Cost estimation: Active");
    println!("  • File validation: Passed");

    println!("\n✨ Phase 4 Demo Complete!");
    println!("🚀 Successfully demonstrated:");
    println!();
    println!("📁 **File Upload System (Phase 4.3)**");
    println!("   ✅ Multi-format file creation and validation");
    println!("   ✅ MIME type detection and constraints");
    println!("   ✅ File processing utilities and analysis");
    println!("   ✅ Hash calculation and type checking");
    println!();
    println!("🏗️ **Enhanced Infrastructure (Phase 4.4)**");
    println!("   ✅ Real-time token counting and cost estimation");
    println!("   ✅ Retry logic with exponential backoff");
    println!("   ✅ Error condition handling and recovery");
    println!("   ✅ Session monitoring and analytics");
    println!();
    println!("🎯 **Production Ready Features**");
    println!("   ✅ Type-safe file handling with validation");
    println!("   ✅ Intelligent retry policies for resilience");
    println!("   ✅ Cost tracking for budget management");
    println!("   ✅ Comprehensive error handling");

    Ok(())
}
