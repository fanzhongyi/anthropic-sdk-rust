pub mod batches;
pub mod errors;
pub mod files_api;
pub mod messages;
pub mod models;
pub mod models_api;
pub mod shared;
pub mod streaming;
pub mod tools;

// Re-exports for convenience
pub use errors::{AnthropicError, Result};
pub use shared::{
    CacheControl, HasRequestId, RequestId, ResponseEnvelope, ResponseMeta, ServerToolUsage,
    ThinkingConfig, Usage,
};

// Message types
pub use messages::{
    ContentBlock, ContentBlockParam, ContentBlocksExt, ImageSource, Message, MessageContent,
    MessageCreateBuilder, MessageCreateParams, MessageParam, Role, StopReason, SystemContentBlock,
    SystemParam, ToolResultContentParam, ToolResultNestedBlockParam,
};

// Model types
pub use models::Model;

// Streaming types
pub use streaming::{
    ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent, ContentBlockStopEvent,
    MessageDelta, MessageDeltaEvent, MessageDeltaUsage, MessageStartEvent, MessageStopEvent,
    MessageStreamEvent, TextCitation,
};

// Tool types
pub use tools::{
    ImageSource as ToolImageSource, ServerTool, Tool, ToolBuilder, ToolChoice, ToolInputSchema,
    ToolResult, ToolResultBlock, ToolResultContent, ToolUse, ToolValidationError,
    WebSearchParameters,
};

// Batch types
pub use batches::{
    BatchCreateParams, BatchError, BatchList, BatchListParams, BatchRequest, BatchRequestBuilder,
    BatchRequestCounts, BatchResponse, BatchResponseBody, BatchResult, BatchStatus, MessageBatch,
};

// Files API types
pub use files_api::{
    FileDownload, FileList, FileListParams, FileObject, FileOrder, FilePurpose, FileStatus,
    FileUploadParams, StorageInfo, UploadProgress,
};

// Models API types
pub use models_api::{
    ComparisonSummary, CostBreakdown, CostEstimation, CostRange, ModelCapabilities,
    ModelCapability, ModelComparison, ModelList, ModelListParams, ModelObject, ModelPerformance,
    ModelPricing, ModelRecommendation, ModelRequirements, ModelUsageRecommendations,
    PerformanceExpectations, PricingTier, QualityLevel, RecommendedParameters,
};
