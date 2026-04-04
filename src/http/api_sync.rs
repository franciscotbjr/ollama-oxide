//! Sync (blocking) API trait and implementations

use crate::{
    ChatRequest, ChatResponse, EmbedRequest, EmbedResponse, GenerateRequest, GenerateResponse,
    Result, VersionResponse,
};

use super::streaming::ChatStreamBlocking;

#[cfg(feature = "model")]
use crate::{
    CopyRequest, CreateRequest, CreateResponse, DeleteRequest, ListResponse, PsResponse,
    PullRequest, PullResponse, PushRequest, PushResponse, ShowRequest, ShowResponse,
};

use super::OllamaClient;
use super::endpoints::Endpoints;

/// Sync API operations trait
///
/// This trait defines all synchronous (blocking) methods for interacting with
/// the Ollama API. All methods block the current thread until completion.
///
/// # Thread Safety
///
/// Implementations of this trait must be `Send + Sync` to support concurrent usage
/// across threads.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, OllamaApiSync};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = OllamaClient::default()?;
///     let version = client.version_blocking()?;
///     println!("Ollama version: {}", version.version);
///     Ok(())
/// }
/// ```
pub trait OllamaApiSync: Send + Sync {
    /// Get Ollama server version (blocking)
    ///
    /// This method blocks the current thread until the request completes.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.version_blocking()?;
    /// println!("Version: {}", response.version);
    /// # Ok(())
    /// # }
    /// ```
    fn version_blocking(&self) -> Result<VersionResponse>;

    /// List locally available models (blocking)
    ///
    /// Returns a list of models installed on the Ollama server with their details.
    /// This method blocks the current thread until the request completes.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.list_models_blocking()?;
    /// for model in &response.models {
    ///     println!("Model: {}", model.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn list_models_blocking(&self) -> Result<ListResponse>;

    /// Copy a model (blocking)
    ///
    /// Creates a copy of an existing model with a new name. This is useful for
    /// creating backups or variants of models without downloading them again.
    /// This method blocks the current thread until the request completes.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, CopyRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = CopyRequest::new("llama3.1", "llama3.1-backup");
    /// client.copy_model_blocking(&request)?;
    /// println!("Model copied successfully!");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn copy_model_blocking(&self, request: &CopyRequest) -> Result<()>;

    /// List currently running models (blocking)
    ///
    /// Returns a list of models that are currently loaded in memory and ready
    /// for inference. This includes information about VRAM usage, context length,
    /// and expiration time.
    /// This method blocks the current thread until the request completes.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let response = client.list_running_models_blocking()?;
    /// for model in &response.models {
    ///     println!("Running: {} (VRAM: {:?})", model.model, model.size_vram);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn list_running_models_blocking(&self) -> Result<PsResponse>;

    /// Delete a model (blocking)
    ///
    /// Permanently removes a model from the Ollama server. This operation
    /// cannot be undone.
    /// This method blocks the current thread until the request completes.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, DeleteRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = DeleteRequest::new("llama3.1-backup");
    /// client.delete_model_blocking(&request)?;
    /// println!("Model deleted successfully!");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn delete_model_blocking(&self, request: &DeleteRequest) -> Result<()>;

    /// Show detailed information about a model (blocking)
    ///
    /// Retrieves comprehensive metadata including parameters,
    /// license, capabilities, and model-specific configuration.
    /// This method blocks the current thread until the request completes.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, ShowRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    ///
    /// // Basic request
    /// let request = ShowRequest::new("llama3.1");
    /// let response = client.show_model_blocking(&request)?;
    /// println!("Capabilities: {:?}", response.capabilities);
    ///
    /// // Verbose request
    /// let verbose_request = ShowRequest::verbose("llama3.1");
    /// let verbose_response = client.show_model_blocking(&verbose_request)?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn show_model_blocking(&self, request: &ShowRequest) -> Result<ShowResponse>;

    /// Generate embeddings for text (blocking)
    ///
    /// Creates vector embeddings representing the input text(s).
    /// This method blocks the current thread until the request completes.
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
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, EmbedRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = EmbedRequest::new("nomic-embed-text", "Hello, world!");
    /// let response = client.embed_blocking(&request)?;
    /// println!("Embedding dimensions: {:?}", response.dimensions());
    /// # Ok(())
    /// # }
    /// ```
    fn embed_blocking(&self, request: &EmbedRequest) -> Result<EmbedResponse>;

    /// Generate text completion (blocking, non-streaming)
    ///
    /// Generates a text completion for the provided prompt.
    /// This method blocks the current thread until completion.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, GenerateRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = GenerateRequest::new("qwen3:0.6b", "Tell me a joke.");
    /// let response = client.generate_blocking(&request)?;
    /// println!("{}", response.text().unwrap_or("No response"));
    /// # Ok(())
    /// # }
    /// ```
    fn generate_blocking(&self, request: &GenerateRequest) -> Result<GenerateResponse>;

    /// Chat completion (blocking, non-streaming)
    ///
    /// Generates the next message in a chat conversation.
    /// This method blocks the current thread until completion.
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
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, ChatRequest, ChatMessage};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Tell me a short joke.")
    /// ]);
    /// let response = client.chat_blocking(&request)?;
    /// println!("{}", response.content().unwrap_or("No response"));
    /// # Ok(())
    /// # }
    /// ```
    fn chat_blocking(&self, request: &ChatRequest) -> Result<ChatResponse>;

    /// Chat completion with streaming (NDJSON), blocking iterator.
    ///
    /// Sends `stream: true` regardless of the value on `request`. Each line of the
    /// response body is deserialized as [`ChatResponse`].
    ///
    /// # Arguments
    ///
    /// * `request` - Chat request. Streaming methods set `stream` to `true` on a
    ///   clone of this value before sending.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP status is not success before the body is read.
    /// Per-line JSON errors are yielded as [`Err`](crate::Result) from the iterator.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatMessage, ChatRequest, OllamaApiSync, OllamaClient};
    ///
    /// # fn example() -> ollama_oxide::Result<()> {
    /// let client = OllamaClient::default()?;
    /// let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hi!")]);
    /// for ev in client.chat_stream_blocking(&request)? {
    ///     let chunk = ev?;
    ///     if let Some(t) = chunk.content() {
    ///         print!("{}", t);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn chat_stream_blocking(&self, request: &ChatRequest) -> Result<ChatStreamBlocking>;

    /// Create a custom model (blocking, non-streaming)
    ///
    /// Creates a new model from an existing model with custom configuration.
    /// This method blocks the current thread until completion.
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, CreateRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = CreateRequest::from_model("mario", "qwen3:0.6b")
    ///     .with_system("You are Mario from Super Mario Bros.");
    /// let response = client.create_model_blocking(&request)?;
    /// println!("Status: {:?}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn create_model_blocking(&self, request: &CreateRequest) -> Result<CreateResponse>;

    /// Pull (download) a model from the Ollama registry (blocking).
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, PullRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = PullRequest::new("all-minilm:33m");
    /// let response = client.pull_model_blocking(&request)?;
    /// println!("Status: {:?}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn pull_model_blocking(&self, request: &PullRequest) -> Result<PullResponse>;

    /// Push (upload) a model to the Ollama registry (blocking).
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
    /// use ollama_oxide::{OllamaClient, OllamaApiSync, PushRequest};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OllamaClient::default()?;
    /// let request = PushRequest::new("myuser/mymodel:latest");
    /// let response = client.push_model_blocking(&request)?;
    /// println!("Status: {:?}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "model")]
    fn push_model_blocking(&self, request: &PushRequest) -> Result<PushResponse>;
}

impl OllamaApiSync for OllamaClient {
    fn version_blocking(&self) -> Result<VersionResponse> {
        let url = self.config.url(Endpoints::VERSION);
        self.get_blocking_with_retry(&url)
    }

    #[cfg(feature = "model")]
    fn list_models_blocking(&self) -> Result<ListResponse> {
        let url = self.config.url(Endpoints::TAGS);
        self.get_blocking_with_retry(&url)
    }

    #[cfg(feature = "model")]
    fn copy_model_blocking(&self, request: &CopyRequest) -> Result<()> {
        let url = self.config.url(Endpoints::COPY);
        self.post_empty_blocking_with_retry(&url, request)
    }

    #[cfg(feature = "model")]
    fn list_running_models_blocking(&self) -> Result<PsResponse> {
        let url = self.config.url(Endpoints::PS);
        self.get_blocking_with_retry(&url)
    }

    #[cfg(feature = "model")]
    fn delete_model_blocking(&self, request: &DeleteRequest) -> Result<()> {
        let url = self.config.url(Endpoints::DELETE);
        self.delete_empty_blocking_with_retry(&url, request)
    }

    #[cfg(feature = "model")]
    fn show_model_blocking(&self, request: &ShowRequest) -> Result<ShowResponse> {
        let url = self.config.url(Endpoints::SHOW);
        self.post_blocking_with_retry(&url, request)
    }

    fn embed_blocking(&self, request: &EmbedRequest) -> Result<EmbedResponse> {
        let url = self.config.url(Endpoints::EMBED);
        self.post_blocking_with_retry(&url, request)
    }

    fn generate_blocking(&self, request: &GenerateRequest) -> Result<GenerateResponse> {
        let url = self.config.url(Endpoints::GENERATE);
        self.post_blocking_with_retry(&url, request)
    }

    fn chat_blocking(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let url = self.config.url(Endpoints::CHAT);
        self.post_blocking_with_retry(&url, request)
    }

    fn chat_stream_blocking(&self, request: &ChatRequest) -> Result<ChatStreamBlocking> {
        let mut req = request.clone();
        req.stream = Some(true);
        let url = self.config.url(Endpoints::CHAT);
        self.post_ndjson_stream_blocking(&url, &req)
    }

    #[cfg(feature = "model")]
    fn create_model_blocking(&self, request: &CreateRequest) -> Result<CreateResponse> {
        let url = self.config.url(Endpoints::CREATE);
        self.post_blocking_with_retry(&url, request)
    }

    #[cfg(feature = "model")]
    fn pull_model_blocking(&self, request: &PullRequest) -> Result<PullResponse> {
        let url = self.config.url(Endpoints::PULL);
        self.post_blocking_with_retry(&url, request)
    }

    #[cfg(feature = "model")]
    fn push_model_blocking(&self, request: &PushRequest) -> Result<PushResponse> {
        let url = self.config.url(Endpoints::PUSH);
        self.post_blocking_with_retry(&url, request)
    }
}
