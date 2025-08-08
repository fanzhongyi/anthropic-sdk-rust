use serde::{Deserialize, Serialize};

/// Request ID for tracking API requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Usage information for API requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    /// The number of input tokens which were used
    pub input_tokens: u32,
    
    /// The number of output tokens which were used
    pub output_tokens: u32,
    
    /// The number of input tokens used to create the cache entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    
    /// The number of input tokens read from the cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    
    /// Detailed cache creation breakdown (for 1-hour cache beta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation: Option<CacheCreation>,
    
    /// Server tool usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<ServerToolUsage>,
    
    /// Service tier used for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}

/// Server tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerToolUsage {
    /// Number of web search tool requests made
    pub web_search_requests: u32,
}

impl Usage {
    /// Get the total number of tokens used
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
    
    /// Get the total input tokens including cache tokens
    pub fn total_input_tokens(&self) -> u32 {
        self.input_tokens 
            + self.cache_creation_input_tokens.unwrap_or(0)
            + self.cache_read_input_tokens.unwrap_or(0)
    }
}

/// Cache control configuration for prompt caching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheControl {
    /// The type of cache control
    #[serde(rename = "type")]
    pub cache_type: CacheType,
    
    /// Time-to-live for the cache entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<CacheTtl>,
}

/// Cache type for prompt caching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    /// Ephemeral cache with configurable TTL
    Ephemeral,
}

/// Cache time-to-live options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheTtl {
    /// 5-minute cache duration
    #[serde(rename = "5m")]
    FiveMinutes,
    
    /// 1-hour cache duration (requires beta header)
    #[serde(rename = "1h")]
    OneHour,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: None,
        }
    }
}

impl CacheControl {
    /// Create a new ephemeral cache control with default 5-minute TTL
    pub fn ephemeral() -> Self {
        Self::default()
    }
    
    /// Create a new ephemeral cache control with 5-minute TTL
    pub fn ephemeral_5m() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: Some(CacheTtl::FiveMinutes),
        }
    }
    
    /// Create a new ephemeral cache control with 1-hour TTL
    pub fn ephemeral_1h() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: Some(CacheTtl::OneHour),
        }
    }
}

/// Extended cache creation breakdown for detailed billing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheCreation {
    /// Number of tokens written to 5-minute ephemeral cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral_5m_input_tokens: Option<u32>,
    
    /// Number of tokens written to 1-hour ephemeral cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral_1h_input_tokens: Option<u32>,
}

/// Extended thinking configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThinkingConfig {
    /// The type of thinking mode
    #[serde(rename = "type")]
    pub thinking_type: ThinkingType,
    
    /// Maximum number of tokens Claude can use for thinking
    pub budget_tokens: u32,
}

/// Type of extended thinking mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingType {
    /// Extended thinking is enabled
    Enabled,
}

impl ThinkingConfig {
    /// Create a new thinking configuration
    pub fn new(budget_tokens: u32) -> Self {
        Self {
            thinking_type: ThinkingType::Enabled,
            budget_tokens,
        }
    }
    
    /// Create a thinking configuration with enabled type
    pub fn enabled(budget_tokens: u32) -> Self {
        Self::new(budget_tokens)
    }
}

/// Base trait for responses that include request IDs
pub trait HasRequestId {
    fn request_id(&self) -> Option<&RequestId>;
} 
