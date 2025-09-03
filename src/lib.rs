//! # Anthropic SDK for Rust
//!
//! This crate provides a Rust SDK for the Anthropic API, offering feature parity
//! with the official TypeScript SDK.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use anthropic_sdk::Anthropic;
//!
//! // Create client from environment variable ANTHROPIC_API_KEY
//! let client = Anthropic::from_env()?;
//!
//! // Or create with explicit API key
//! let client = Anthropic::new("your-api-key")?;
//!
//! // Test the connection
//! client.test_connection().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Messages API**: Create and stream Claude conversations
//! - **Authentication**: Automatic API key management
//! - **Error Handling**: Comprehensive error types
//! - **Logging**: Configurable tracing support
//! - **Async/Await**: Built on tokio for high performance
//!

pub mod client;
pub mod config;
pub mod files;
pub mod http;
pub mod resources;
pub mod streaming;
pub mod tokens;
pub mod tools;
pub mod types;
pub mod utils;

// Re-exports for public API
pub use client::Anthropic;
pub use config::{ClientConfig, LogLevel};
pub use files::{to_file, File, FileBuilder, FileConstraints, FileData, FileError, FileSource};
pub use http::auth::AuthMethod;
pub use http::{api_retry, default_retry, RetryCondition, RetryExecutor, RetryPolicy, RetryResult};
pub use resources::{BatchesResource, FilesResource, MessagesResource, ModelsResource};
pub use streaming::MessageStream;
pub use tokens::{ModelPrice, ModelUsage, RequestUsage, TokenCounter, UsageStats, UsageSummary};
pub use tools::{
    ConversationConfig, ToolConversation, ToolError, ToolExecutionConfig, ToolExecutor,
    ToolFunction, ToolRegistry,
};
pub use types::{
    AnthropicError,
    BatchCreateParams,
    BatchError,
    BatchList,
    BatchListParams,
    BatchRequest,
    BatchRequestBuilder,
    BatchRequestCounts,
    BatchResponse,
    BatchResponseBody,
    BatchResult,
    BatchStatus,
    CacheControl,
    ComparisonSummary,
    ContentBlock,
    ContentBlockDelta,
    ContentBlockParam,
    CostBreakdown,
    CostEstimation,
    CostRange,
    FileDownload,
    FileList,
    FileListParams,
    // Files API types (Beta)
    FileObject,
    FileOrder,
    FilePurpose,
    FileStatus,
    FileUploadParams,
    ImageSource,
    Message,
    // Batch types (Beta)
    MessageBatch,
    MessageContent,
    MessageCreateBuilder,
    MessageCreateParams,
    MessageDelta,
    MessageDeltaUsage,
    MessageParam,
    // Streaming types
    MessageStreamEvent,
    Model,
    ModelCapabilities,
    ModelCapability,
    ModelComparison,
    ModelList,
    ModelListParams,
    // Models API types
    ModelObject,
    ModelPerformance,
    ModelPricing,
    ModelRecommendation,
    ModelRequirements,
    ModelUsageRecommendations,
    PerformanceExpectations,
    PricingTier,
    QualityLevel,
    RecommendedParameters,
    RequestId,
    Result,
    Role,
    ServerTool,
    StopReason,
    StorageInfo,
    SystemContentBlock,
    SystemParam,
    TextCitation,
    ThinkingConfig,
    // Tool types
    Tool,
    ToolBuilder,
    ToolChoice,
    ToolResult,
    ToolResultContent,
    ToolUse,
    ToolValidationError,
    UploadProgress,
    Usage,
    WebSearchParameters,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User-Agent string for HTTP requests
pub const USER_AGENT: &str = concat!("anthropic-sdk-rust/", env!("CARGO_PKG_VERSION"));

// Convenient type aliases
pub type Error = AnthropicError;

// Module re-exports for organized access
pub use types as types_module;
