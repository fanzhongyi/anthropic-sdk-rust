use anthropic_sdk::{
    Anthropic, BatchCreateParams, BatchError, BatchListParams, BatchRequest, BatchRequestCounts,
    BatchResponse, BatchResponseBody, BatchResult, BatchStatus, ContentBlock, Message,
    MessageBatch, Role, StopReason, Usage,
};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Phase 5.1: Message Batches API Demo");
    println!("=====================================");

    // Initialize client (would normally use real API key)
    let _client = match Anthropic::from_env() {
        Ok(_client) => _client,
        Err(_) => {
            println!("⚠️  ANTHROPIC_API_KEY not set. This is a demo of the batch API structure.");
            simulate_batch_processing().await?;
            return Ok(());
        }
    };

    // Demo 1: Create batch requests
    println!("\n📝 Demo 1: Creating Batch Requests");
    println!("----------------------------------");

    // Create diverse batch requests
    let requests = vec![
        BatchRequest::new("translate_french", "claude-3-haiku@20240307", 1024)
            .user("Translate to French: Hello, how are you today?")
            .system("You are a professional translator")
            .temperature(0.3)
            .build(),
        BatchRequest::new("translate_english", "claude-3-haiku@20240307", 1024)
            .user("Translate to English: Hello, how are you today?")
            .system("You are a professional translator")
            .temperature(0.3)
            .build(),
        BatchRequest::new("translate_french", "claude-3-haiku@20240307", 1024)
            .user("Translate to French: Hello, how are you today?")
            .system("You are a professional translator")
            .temperature(0.3)
            .build(),
        BatchRequest::new("translate_spanish", "claude-3-haiku@20240307", 1024)
            .user("Translate to Spanish: Hello, how are you today?")
            .system("You are a professional translator")
            .temperature(0.3)
            .build(),
        BatchRequest::new("translate_german", "claude-3-haiku@20240307", 1024)
            .user("Translate to German: Hello, how are you today?")
            .system("You are a professional translator")
            .temperature(0.3)
            .build(),
        BatchRequest::new("creative_story", "claude-3-haiku@20240307", 1024)
            .user("Write a short story about a robot learning to paint")
            .system("You are a creative writer")
            .temperature(0.8)
            .build(),
        BatchRequest::new("code_review", "claude-3-haiku@20240307", 1024)
            .user("Review this Python code for best practices:\n\ndef calculate_total(items):\n    total = 0\n    for item in items:\n        total += item['price']\n    return total")
            .system("You are a senior software engineer")
            .temperature(0.2)
            .build(),
        BatchRequest::new("summarize", "claude-3-haiku@20240307", 1024)
            .user("Summarize the following: Claude is a large language model trained by Anthropic.")
            .system("You are a professional summarizer")
            .temperature(0.3)
            .build(),
        BatchRequest::new("summarize", "claude-3-haiku@20240307", 1024)
            .user("Summarize the following: Claude is a large language model trained by Anthropic.")
            .system("You are a professional summarizer")
            .temperature(0.7)
            .build(),
    ];

    println!("✅ Created {} batch requests:", requests.len());
    for (i, request) in requests.iter().enumerate() {
        println!(
            "   {}. {} ({})",
            i + 1,
            request.custom_id,
            request.body.model
        );
    }

    // Demo 2: Create batch with metadata
    println!("\n📦 Demo 2: Creating Batch with Metadata");
    println!("---------------------------------------");

    let mut metadata = HashMap::new();
    metadata.insert("project".to_string(), "translation_demo".to_string());
    metadata.insert("user_id".to_string(), "demo_user_123".to_string());
    metadata.insert("batch_type".to_string(), "mixed_requests".to_string());

    let batch_params = BatchCreateParams::new(requests)
        .with_metadata(metadata)
        .with_completion_window(24); // 24 hours

    println!("✅ Batch parameters configured:");
    println!("   • Requests: {}", batch_params.requests.len());
    println!(
        "   • Completion window: {} hours",
        batch_params.completion_window.unwrap_or(24)
    );
    println!("   • Metadata: {:?}", batch_params.metadata);

    // Demo 3: Batch creation and monitoring (simulated)
    println!("\n🔄 Demo 3: Batch Creation and Monitoring");
    println!("----------------------------------------");

    // In a real scenario, you would do:
    // let batch = client.batches().create(batch_params).await?;
    // println!("✅ Created batch: {}", batch.id);

    // Demo with simulated batch
    let batch = create_mock_batch();
    println!("✅ Created batch: {}", batch.id);
    println!("   • Status: {:?}", batch.processing_status);
    println!("   • Total requests: {}", batch.request_counts.total);
    println!("   • Created at: {}", batch.created_at);

    // Demo 4: Status monitoring simulation
    println!("\n📊 Demo 4: Progress Monitoring");
    println!("------------------------------");

    simulate_batch_progress(&batch).await;

    // Demo 5: Batch listing and management
    println!("\n📋 Demo 5: Batch Listing and Management");
    println!("---------------------------------------");

    let list_params = BatchListParams::new().limit(10).after("batch_20241201_001");

    println!("✅ List parameters:");
    println!("   • Limit: {:?}", list_params.limit);
    println!("   • After: {:?}", list_params.after);

    // Demo 6: Result processing simulation
    println!("\n📄 Demo 6: Result Processing");
    println!("----------------------------");

    simulate_result_processing().await;

    println!("\n🎉 Phase 5.1 Batch API Demo Complete!");
    println!("=====================================");
    println!("✅ Batch Types: MessageBatch, BatchRequest, BatchResult");
    println!("✅ Batch Operations: Create, Monitor, List, Cancel");
    println!("✅ Progress Tracking: Status monitoring and callbacks");
    println!("✅ Result Processing: JSONL parsing and error handling");
    println!("✅ Metadata Support: Custom batch metadata");
    println!("✅ Builder Pattern: Ergonomic request creation");

    Ok(())
}

async fn simulate_batch_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Simulating batch processing workflow...");

    // Create sample requests
    let requests = vec![
        BatchRequest::new("demo1", "claude-3-haiku@20240307", 1024)
            .user("What is the capital of France?")
            .build(),
        BatchRequest::new("demo2", "claude-3-haiku@20240307", 1024)
            .user("Explain quantum computing in simple terms")
            .build(),
    ];

    let params = BatchCreateParams::new(requests);
    println!(
        "✅ Batch configured with {} requests",
        params.requests.len()
    );

    // Simulate batch lifecycle
    let mut batch = create_mock_batch();

    println!("\n📊 Simulating batch lifecycle:");
    let statuses = [
        BatchStatus::Validating,
        BatchStatus::InProgress,
        BatchStatus::Finalizing,
        BatchStatus::Completed,
    ];

    for (i, status) in statuses.iter().enumerate() {
        batch.processing_status = *status;
        batch.request_counts.completed = (i as u32 * 25).min(100);

        println!(
            "   • {:?}: {}% complete",
            status,
            batch.completion_percentage()
        );
        sleep(Duration::from_millis(500)).await;
    }

    println!("✅ Batch processing simulation complete!");
    Ok(())
}

async fn simulate_batch_progress(_batch: &MessageBatch) {
    println!("📈 Monitoring batch progress...");

    let progress_steps = [
        (BatchStatus::Validating, 0, "Validating input requests"),
        (BatchStatus::InProgress, 25, "Processing requests (1/4)"),
        (BatchStatus::InProgress, 50, "Processing requests (2/4)"),
        (BatchStatus::InProgress, 75, "Processing requests (3/4)"),
        (BatchStatus::Finalizing, 90, "Finalizing results"),
        (BatchStatus::Completed, 100, "All requests completed"),
    ];

    for (status, progress, description) in progress_steps {
        println!("   • {status:?} - {progress}% - {description}");

        if status.is_terminal() {
            println!("   ✅ Batch reached terminal state");
        } else if status.is_processing() {
            println!("   ⏳ Batch is still processing...");
        }

        sleep(Duration::from_millis(800)).await;
    }
}

async fn simulate_result_processing() {
    println!("📤 Processing batch results...");

    let sample_results = vec![
        create_mock_result(
            "translate_french",
            true,
            "Bonjour, comment allez-vous aujourd'hui ?",
        ),
        create_mock_result("translate_spanish", true, "Hola, ¿cómo estás hoy?"),
        create_mock_result("translate_german", true, "Hallo, wie geht es dir heute?"),
        create_mock_result(
            "creative_story",
            true,
            "In a small workshop filled with canvases...",
        ),
        create_mock_result("code_review", false, "Rate limit exceeded"),
    ];

    println!("✅ Processing {} results:", sample_results.len());

    let mut successful = 0;
    let mut failed = 0;

    for result in sample_results {
        match result.response.body {
            BatchResponseBody::Success(_) => {
                successful += 1;
                println!("   ✅ {}: Success", result.custom_id);
            }
            BatchResponseBody::Error(ref error) => {
                failed += 1;
                println!("   ❌ {}: Error - {}", result.custom_id, error.message);
            }
        }
    }

    println!("\n📊 Results Summary:");
    println!("   • Successful: {successful}");
    println!("   • Failed: {failed}");
    println!(
        "   • Success rate: {:.1}%",
        (successful as f64 / (successful + failed) as f64) * 100.0
    );
}

fn create_mock_batch() -> MessageBatch {
    use chrono::Utc;

    MessageBatch {
        id: "batch_20241201_demo_001".to_string(),
        object_type: "message_batch".to_string(),
        processing_status: BatchStatus::Validating,
        request_counts: BatchRequestCounts {
            total: 5,
            completed: 0,
            failed: 0,
        },
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        ended_at: None,
        input_file_id: "file_input_demo_001".to_string(),
        output_file_id: None,
        error_file_id: None,
        metadata: {
            let mut map = HashMap::new();
            map.insert("demo".to_string(), "phase_5_1".to_string());
            map
        },
    }
}

fn create_mock_result(custom_id: &str, success: bool, content: &str) -> BatchResult {
    let body = if success {
        BatchResponseBody::Success(Message {
            id: format!("msg_{custom_id}"),
            type_: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: content.to_string(),
            }],
            model: "claude-3-haiku@20240307".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            usage: Usage {
                input_tokens: 50,
                output_tokens: 20,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
                cache_creation: None,
                server_tool_use: None,
                service_tier: None,
            },
            request_id: None,
        })
    } else {
        BatchResponseBody::Error(BatchError {
            error_type: "rate_limit_error".to_string(),
            message: content.to_string(),
            details: HashMap::new(),
        })
    };

    BatchResult {
        custom_id: custom_id.to_string(),
        response: BatchResponse {
            status_code: if success { 200 } else { 429 },
            headers: HashMap::new(),
            body,
        },
    }
}
