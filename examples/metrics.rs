//! Measure streaming metrics such as TTFT (time to first token) using callbacks.
//!
//! Run:
//!   export ANTHROPIC_API_KEY="sk-..."
//!   cargo run --example metrics_ttft

use anthropic_sdk::tokens::TokenCounter;
use anthropic_sdk::types::messages::MessageCreateBuilder;
use anthropic_sdk::types::ContentBlockDelta;
use anthropic_sdk::types::MessageStreamEvent;
use anthropic_sdk::Anthropic;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const MODEL: &str = "claude-sonnet-4@20250514";
// const MODEL: &str = "claude-3-5-haiku@20241022";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Anthropic::from_env()?;

    // Prepare a prompt that streams for a bit so TTFT/throughput are visible
    let params = MessageCreateBuilder::new(MODEL, 512)
        .user("Write a poem about rivers and mountains. Use multiple lines and some longer sentences so we can observe streaming speed.")
        .build();

    // Start timing before initiating the streaming request
    let start_ts = Instant::now();

    // Get stream with response metadata so we can also print request_id/status
    let env = client
        .messages()
        .create_stream_with_response(params)
        .await?;

    println!("status: {}", env.response.status);
    println!(
        "request_id: {:?}",
        env.response.request_id.as_ref().map(|r| r.as_str())
    );

    // Shared counters and timers
    let msg_start_ts: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let first_text_ts: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let total_chars = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let text_arrival_ts: Arc<Mutex<Vec<Instant>>> = Arc::new(Mutex::new(Vec::new()));

    // Clone handles for the callback
    let msg_start_ts_c = msg_start_ts.clone();
    let first_text_ts_c = first_text_ts.clone();
    let total_chars_c = total_chars.clone();
    let text_arrival_ts_c = text_arrival_ts.clone();

    let stream = env.data.on_stream_event(move |event, _snapshot| {
        match event {
            MessageStreamEvent::MessageStart { .. } => {
                let mut guard = msg_start_ts_c.lock().unwrap();
                if guard.is_none() {
                    *guard = Some(Instant::now());
                }
            }
            MessageStreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::TextDelta { text },
                ..
            } => {
                // Record the first time we see any text
                let mut fg = first_text_ts_c.lock().unwrap();
                if fg.is_none() {
                    *fg = Some(Instant::now());
                }
                total_chars_c.fetch_add(text.len(), std::sync::atomic::Ordering::Relaxed);
                // Optionally print incremental text
                text_arrival_ts_c.lock().unwrap().push(Instant::now());
                print!("{text}");
            }
            _ => {}
        }
    });

    let final_msg = stream.final_message().await?;
    let end_ts = Instant::now();

    // Compute metrics (use microseconds to avoid showing 0 on very fast responses)
    let ttft1_ms = msg_start_ts
        .lock()
        .unwrap()
        .map(|t| t.duration_since(start_ts).as_micros() as f64 / 1000.0);
    let ttft2_ms = first_text_ts
        .lock()
        .unwrap()
        .map(|t| t.duration_since(start_ts).as_micros() as f64 / 1000.0);
    let total_ms = end_ts.duration_since(start_ts).as_micros() as f64 / 1000.0;

    let chars = total_chars.load(std::sync::atomic::Ordering::Relaxed);
    let cps = if total_ms > 0.0 {
        (chars as f64) / (total_ms / 1000.0)
    } else {
        0.0
    };

    println!("\n\n== Metrics ==");
    match ttft1_ms {
        Some(v) => println!("TTFT1 (to message_start): {v:.3} ms"),
        None => println!("TTFT1 (to message_start): N/A"),
    }
    match ttft2_ms {
        Some(v) => println!("TTFT2 (to first text): {v:.3} ms"),
        None => println!("TTFT2 (to first text): N/A"),
    }
    println!("Total time: {total_ms:.3} ms");
    println!("Chars: {chars} (~{cps:.2} chars/sec)");

    // Output tokens and throughput
    let out_tokens = final_msg.usage.output_tokens;
    println!("Output tokens: {out_tokens}");
    if let Some(ft) = *first_text_ts.lock().unwrap() {
        let gen_ms = end_ts.duration_since(ft).as_micros() as f64 / 1000.0;
        let tpot_ms_per_token = if out_tokens > 0 {
            gen_ms / out_tokens as f64
        } else {
            f64::NAN
        };
        let tokens_per_sec = if gen_ms > 0.0 {
            (out_tokens as f64) / (gen_ms / 1000.0)
        } else {
            0.0
        };
        println!("TPOT: {tpot_ms_per_token:.3} ms/token");
        println!("Tokens/sec: {tokens_per_sec:.2}");
    } else {
        println!("TPOT: N/A");
        println!("Tokens/sec: N/A");
    }

    // Text delta arrival interval stats
    let mut intervals_ms: Vec<f64> = {
        let stamps = text_arrival_ts.lock().unwrap();
        let mut v = Vec::new();
        for w in stamps.windows(2) {
            let dt = w[1].duration_since(w[0]).as_micros() as f64 / 1000.0;
            v.push(dt);
        }
        v
    };
    if !intervals_ms.is_empty() {
        intervals_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mean = intervals_ms.iter().copied().sum::<f64>() / intervals_ms.len() as f64;
        let p50_idx =
            ((intervals_ms.len() as f64 * 0.5).floor() as usize).min(intervals_ms.len() - 1);
        let p95_idx =
            ((intervals_ms.len() as f64 * 0.95).floor() as usize).min(intervals_ms.len() - 1);
        let p50 = intervals_ms[p50_idx];
        let p95 = intervals_ms[p95_idx];
        println!(
            "Text delta interval: mean={:.3} ms, p50={:.3} ms, p95={:.3} ms (n={})",
            mean,
            p50,
            p95,
            intervals_ms.len()
        );
    } else {
        println!("Text delta interval: N/A");
    }

    // Cost estimation using TokenCounter (pricing table has sensible defaults)
    let counter = TokenCounter::new();
    let cost = counter.record_usage(MODEL, &final_msg.usage);
    println!("\n== Cost ==");
    println!("{cost}");

    println!("Stop reason: {:?}", final_msg.stop_reason);

    Ok(())
}
