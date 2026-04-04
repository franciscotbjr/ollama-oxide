//! Async API trait and implementations

use crate::{
    ChatRequest, ChatResponse, EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse,
    Result, VersionResponse,
};

use super::streaming::ChatStream;

#[cfg(feature = "model")]
use crate::{
    CopyRequest, CreateRequest, CreateResponse, DeleteRequest, ListResponse, PsResponse,
    PullRequest, PullResponse, PushRequest, PushResponse, ShowRequest, ShowResponse,
};

use async_trait::async_trait;

use super::OllamaClient;
use super::endpoints::Endpoints;

/// Async API operations trait
///
/// This trait defines all asynchronous methods for interacting with the Ollama API.
/// All methods return futures that can be awaited.
///
/// # Thread Safety
///
/// Implementations of this trait must be `Send + Sync` to support concurrent usage
/// across async tasks.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiAsync};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = OllamaClient::default()?;
///     let version = client.version().await?;
///     println!("Ollama version: {}", version.version);
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait OllamaApiAsync: Send + Sync {
    /// Get Ollama server version (async)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.version().await?;
    /// println!("Version: {}", response.version);
    /// # Ok(())
    /// # }
    /// ```
    async fn version(&self) -> Result<VersionResponse>;

    /// List locally available models (async)
    ///
    /// Returns a list of models installed on the Ollama server with their details.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.list_models().await?;
    /// for model in &response.models {
    ///     println!("Model: {}", model.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn list_models(&self) -> Result<ListResponse>;

    /// Copy a model (async)
    ///
    /// Creates a copy of an existing model with a new name. This is useful for
    /// creating backups or variants of models without downloading them again.
    ///
    /// # Arguments
    ///
    /// * `request` - Copy request containing source and destination model names
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Source model doesn't exist (404)
    /// - Destination model name is invalid
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, CopyRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = CopyRequest::new("llama3.1", "llama3.1-backup");
    /// client.copy_model(&request).await?;
    /// println!("Model copied successfully!");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn copy_model(&self, request: &CopyRequest) -> Result<()>;

    /// List currently running models (async)
    ///
    /// Returns a list of models that are currently loaded in memory and ready
    /// for inference. This includes information about VRAM usage, context length,
    /// and expiration time.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.list_running_models().await?;
    /// for model in &response.models {
    ///     println!("Running: {} (VRAM: {:?})", model.model, model.size_vram);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn list_running_models(&self) -> Result<PsResponse>;

    /// Delete a model (async)
    ///
    /// Permanently removes a model from the Ollama server. This operation
    /// cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `request` - Delete request containing the model name to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model doesn't exist (404)
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, DeleteRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = DeleteRequest::new("llama3.1-backup");
    /// client.delete_model(&request).await?;
    /// println!("Model deleted successfully!");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn delete_model(&self, request: &DeleteRequest) -> Result<()>;

    /// Show detailed information about a model (async)
    ///
    /// Retrieves comprehensive metadata including parameters,
    /// license, capabilities, and model-specific configuration.
    ///
    /// # Arguments
    ///
    /// * `request` - ShowRequest containing the model name
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The model does not exist (404)
    /// - Network error occurs
    /// - Response cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, ShowRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    ///
    /// // Basic request
    /// let request = ShowRequest::new("llama3.1");
    /// let response = client.show_model(&request).await?;
    /// println!("Capabilities: {:?}", response.capabilities);
    ///
    /// // Verbose request
    /// let verbose_request = ShowRequest::verbose("llama3.1");
    /// let verbose_response = client.show_model(&verbose_request).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn show_model(&self, request: &ShowRequest) -> Result<ShowResponse>;

    /// Generate embeddings for text (async)
    ///
    /// Creates vector embeddings representing the input text(s).
    /// Embeddings are useful for semantic search, similarity comparison,
    /// and machine learning tasks.
    ///
    /// # Arguments
    ///
    /// * `request` - Embed request containing model name and input text(s)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model doesn't exist (404)
    /// - Input exceeds context window and truncate is false
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    ///
    /// # Examples
    ///
    /// Single text embedding:
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, EmbedRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    /// let response = client.embed(&request).await?;
    /// println!("Embedding dimensions: {:?}", response.dimensions());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Multiple text embeddings:
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, EmbedRequest, EmbedInput};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = EmbedRequest::new(
    ///     "nomic-embed-text",
    ///     EmbedInput::multiple(["First text", "Second text"])
    /// );
    /// let response = client.embed(&request).await?;
    /// println!("Got {} embeddings", response.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse>;

    /// Generate text completion (async, non-streaming)
    ///
    /// Generates a text completion for the provided prompt.
    /// This method uses non-streaming mode.
    ///
    /// # Arguments
    ///
    /// * `request` - Generate request containing model, prompt, and options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model doesn't exist (404)
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, GenerateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = GenerateRequest::new("qwen3:0.6b", "Why is the sky blue?");
    /// let response = client.generate(&request).await?;
    /// println!("Response: {:?}", response.text());
    /// # Ok(())
    /// # }
    /// ```
    async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse>;

    /// Chat completion (async, non-streaming)
    ///
    /// Generates the next message in a chat conversation.
    /// This method uses non-streaming mode.
    ///
    /// # Arguments
    ///
    /// * `request` - Chat request containing model, messages, and options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model doesn't exist (404)
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    ///
    /// # Examples
    ///
    /// Basic chat:
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, ChatRequest, ChatMessage};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Hello!")
    /// ]);
    /// let response = client.chat(&request).await?;
    /// println!("Response: {:?}", response.content());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Multi-turn conversation:
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, ChatRequest, ChatMessage};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::system("You are a helpful assistant."),
    ///     ChatMessage::user("What is Rust?"),
    ///     ChatMessage::assistant("Rust is a systems programming language."),
    ///     ChatMessage::user("What are its main features?"),
    /// ]);
    /// let response = client.chat(&request).await?;
    /// println!("Response: {:?}", response.content());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// With tools (function calling) - requires `tools` feature:
    /// ```ignore
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, ChatRequest, ChatMessage, ToolDefinition};
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("What's the weather in Paris?")
    /// ]).with_tools(vec![
    ///     ToolDefinition::function("get_weather", json!({
    ///         "type": "object",
    ///         "properties": {"location": {"type": "string"}},
    ///         "required": ["location"]
    ///     })).with_description("Get current weather")
    /// ]);
    ///
    /// let response = client.chat(&request).await?;
    /// if response.has_tool_calls() {
    ///     for call in response.tool_calls().unwrap() {
    ///         println!("Tool call: {:?}", call.function_name());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse>;

    /// Chat completion with streaming (NDJSON).
    ///
    /// Sends `stream: true` regardless of the value on `request`. Each line of the
    /// response body is deserialized as [`ChatResponse`]. The final event typically
    /// has `done: Some(true)` and timing fields populated.
    ///
    /// # Arguments
    ///
    /// * `request` - Chat request (model, messages, options). Streaming methods set
    ///   `stream` to `true` on a clone of this value before sending.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP status is not success (including 4xx)
    /// - A chunk cannot be read or a line is not valid JSON (see [`Error::StreamError`](crate::Error::StreamError))
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatMessage, ChatRequest, OllamaApiAsync, OllamaClient};
    ///
    /// # async fn example() -> ollama_oxide::Result<()> {
    /// let client = OllamaClient::default()?;
    /// let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hi!")]);
    /// let stream = client.chat_stream(&request).await?;
    /// while let Some(ev) = stream.next().await {
    ///     let chunk = ev?;
    ///     if let Some(t) = chunk.content() {
    ///         print!("{}", t);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream>;

    /// Create a custom model (async, non-streaming)
    ///
    /// Creates a new model from an existing model with custom configuration.
    /// This method uses non-streaming mode.
    ///
    /// # Arguments
    ///
    /// * `request` - Create request containing model name, base model, and options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Base model doesn't exist (404)
    /// - Model name is invalid
    /// - Network request fails
    /// - Maximum retry attempts exceeded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, CreateRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = CreateRequest::from_model("mario", "qwen3:0.6b")
    ///     .with_system("You are Mario from Super Mario Bros.");
    /// let response = client.create_model(&request).await?;
    /// println!("Status: {:?}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn create_model(&self, request: &CreateRequest) -> Result<CreateResponse>;

    /// Pull (download) a model from the Ollama registry.
    ///
    /// Downloads the specified model from the remote registry to the local
    /// Ollama server. This operation may take several minutes depending on
    /// model size and network speed.
    ///
    /// # Arguments
    ///
    /// * `request` - The pull request containing the model name and options
    ///
    /// # Returns
    ///
    /// A `PullResponse` indicating the success or failure of the operation.
    ///
    /// # Errors
    ///
    /// * `HttpStatusError(404)` - Model not found in registry
    /// * `HttpError` - Network or HTTP errors
    /// * `MaxRetriesExceededError` - Server errors after all retries
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, PullRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = PullRequest::new("all-minilm:33m");
    /// let response = client.pull_model(&request).await?;
    /// println!("Status: {:?}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn pull_model(&self, request: &PullRequest) -> Result<PullResponse>;

    /// Push (upload) a model to the Ollama registry.
    ///
    /// Uploads the specified model to a remote registry. Requires proper
    /// authentication and namespace permissions.
    ///
    /// # Arguments
    ///
    /// * `request` - The push request containing the model name and options
    ///
    /// # Returns
    ///
    /// A `PushResponse` indicating the success or failure of the operation.
    ///
    /// # Errors
    ///
    /// * `HttpStatusError(404)` - Model not found locally
    /// * `HttpStatusError(401)` - Unauthorized (invalid credentials)
    /// * `HttpError` - Network or HTTP errors
    /// * `MaxRetriesExceededError` - Server errors after all retries
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiAsync, PushRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = PushRequest::new("myuser/mymodel:latest");
    /// let response = client.push_model(&request).await?;
    /// println!("Status: {:?}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    async fn push_model(&self, request: &PushRequest) -> Result<PushResponse>;
}

#[async_trait]
impl OllamaApiAsync for OllamaClient {
    async fn version(&self) -> Result<VersionResponse> {
        let url = self.config.url(Endpoints::VERSION);
        self.get_with_retry(&url).await
    }

    #[cfg(feature = "model")]
    async fn list_models(&self) -> Result<ListResponse> {
        let url = self.config.url(Endpoints::TAGS);
        self.get_with_retry(&url).await
    }

    #[cfg(feature = "model")]
    async fn copy_model(&self, request: &CopyRequest) -> Result<()> {
        let url = self.config.url(Endpoints::COPY);
        self.post_empty_with_retry(&url, request).await
    }

    #[cfg(feature = "model")]
    async fn list_running_models(&self) -> Result<PsResponse> {
        let url = self.config.url(Endpoints::PS);
        self.get_with_retry(&url).await
    }

    #[cfg(feature = "model")]
    async fn delete_model(&self, request: &DeleteRequest) -> Result<()> {
        let url = self.config.url(Endpoints::DELETE);
        self.delete_empty_with_retry(&url, request).await
    }

    #[cfg(feature = "model")]
    async fn show_model(&self, request: &ShowRequest) -> Result<ShowResponse> {
        let url = self.config.url(Endpoints::SHOW);
        self.post_with_retry(&url, request).await
    }

    async fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse> {
        let url = self.config.url(Endpoints::EMBED);
        self.post_with_retry(&url, request).await
    }

    async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse> {
        let url = self.config.url(Endpoints::GENERATE);
        self.post_with_retry(&url, request).await
    }

    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let url = self.config.url(Endpoints::CHAT);
        self.post_with_retry(&url, request).await
    }

    async fn chat_stream(&self, request: &ChatRequest) -> Result<ChatStream> {
        let mut req = request.clone();
        req.stream = Some(true);
        let url = self.config.url(Endpoints::CHAT);
        let rx = self.post_ndjson_stream(&url, &req).await?;
        Ok(ChatStream::new(rx))
    }

    #[cfg(feature = "model")]
    async fn create_model(&self, request: &CreateRequest) -> Result<CreateResponse> {
        let url = self.config.url(Endpoints::CREATE);
        self.post_with_retry(&url, request).await
    }

    #[cfg(feature = "model")]
    async fn pull_model(&self, request: &PullRequest) -> Result<PullResponse> {
        let url = self.config.url(Endpoints::PULL);
        self.post_with_retry(&url, request).await
    }

    #[cfg(feature = "model")]
    async fn push_model(&self, request: &PushRequest) -> Result<PushResponse> {
        let url = self.config.url(Endpoints::PUSH);
        self.post_with_retry(&url, request).await
    }
}
