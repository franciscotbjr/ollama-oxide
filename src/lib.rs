//! # ollama-oxide
//!
//! A Rust library for integrating with Ollama's native API.
//!
//! ## Quick Start
//!
//! ### Async Example
//! ```no_run
//! use ollama_oxide::{OllamaClient, OllamaApiAsync, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = OllamaClient::default()?;
//!     let version = client.version().await?;
//!     println!("Ollama version: {}", version.version);
//!     Ok(())
//! }
//! ```
//!
//! ### Sync Example
//! ```no_run
//! use ollama_oxide::{OllamaClient, OllamaApiSync, Result};
//!
//! fn main() -> Result<()> {
//!     let client = OllamaClient::default()?;
//!     let version = client.version_blocking()?;
//!     println!("Ollama version: {}", version.version);
//!     Ok(())
//! }
//! ```

// ============================================================================
// Error Handling
// ============================================================================

mod error;

pub use error::{Error, Result};

// ============================================================================
// Inference Module
// ============================================================================

#[cfg(feature = "inference")]
pub mod inference;

#[cfg(feature = "inference")]
pub use inference::{
    // Chat types
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ChatRole,
    EmbedInput,
    EmbedRequest,
    EmbedResponse,
    FormatSetting,
    GenerateRequest,
    GenerateResponse,
    KeepAliveSetting,
    Logprob,
    ModelOptions,
    ResponseMessage,
    StopSetting,
    ThinkSetting,
    TokenLogprob,
    VersionResponse,
};

// ============================================================================
// HTTP Client Module
// ============================================================================

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "http")]
pub use http::{
    ChatStream, ChatStreamBlocking, ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient,
};

// ============================================================================
// Model Module
// ============================================================================

#[cfg(feature = "model")]
pub mod model;

#[cfg(feature = "model")]
pub use model::{
    CopyRequest, CreateRequest, CreateResponse, DeleteRequest, LicenseSetting, ListResponse,
    ModelDetails, ModelSummary, PsResponse, PullRequest, PullResponse, PushRequest, PushResponse,
    RunningModel, ShowModelDetails, ShowRequest, ShowResponse,
};

// ============================================================================
// Ergonomic Tools Module (requires "tools" feature)
// ============================================================================

#[cfg(feature = "tools")]
pub mod tools;

// Tool types re-exports (requires both "inference" and "tools" features)
#[cfg(feature = "tools")]
pub use tools::{ToolCall, ToolCallFunction, ToolDefinition, ToolFunction};

// ============================================================================
// Prelude
// ============================================================================

pub mod prelude {
    pub use crate::{Error, Result};

    #[cfg(feature = "http")]
    pub use crate::{
        ChatStream, ChatStreamBlocking, ClientConfig, OllamaApiAsync, OllamaApiSync, OllamaClient,
    };

    #[cfg(feature = "inference")]
    pub use crate::{
        // Chat types
        ChatMessage,
        ChatRequest,
        ChatResponse,
        ChatRole,
        // Inference types
        EmbedInput,
        EmbedRequest,
        EmbedResponse,
        FormatSetting,
        GenerateRequest,
        GenerateResponse,
        KeepAliveSetting,
        Logprob,
        ModelOptions,
        ResponseMessage,
        StopSetting,
        ThinkSetting,
        TokenLogprob,
        VersionResponse,
    };

    // Tool types (requires "tools" features)
    #[cfg(feature = "tools")]
    pub use crate::{ToolCall, ToolCallFunction, ToolDefinition, ToolFunction};

    // Model types (requires "model" feature)
    #[cfg(feature = "model")]
    pub use crate::{
        CopyRequest, CreateRequest, CreateResponse, DeleteRequest, LicenseSetting, ListResponse,
        ModelDetails, ModelSummary, PsResponse, RunningModel, ShowModelDetails, ShowRequest,
        ShowResponse,
    };
}
