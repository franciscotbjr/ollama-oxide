//! Ollama HTTP client implementation

use crate::{Error, Result};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use super::ClientConfig;

/// HTTP client for Ollama API
///
/// This client is cloneable and can be safely shared across threads.
/// The internal HTTP client is wrapped in Arc for efficient cloning.
///
/// # Thread Safety
///
/// `OllamaClient` implements `Send + Sync` and can be safely shared
/// across threads or async tasks.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::{OllamaClient, ClientConfig};
/// use std::time::Duration;
///
/// // Create with default configuration
/// let client = OllamaClient::default().unwrap();
///
/// // Create with custom configuration
/// let config = ClientConfig::new(
///     "http://localhost:11434".to_string(),
///     Duration::from_secs(30),
///     3,
/// ).unwrap();
/// let client = OllamaClient::new(config).unwrap();
///
/// // Create with custom base URL
/// let client = OllamaClient::with_base_url("http://localhost:8080").unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct OllamaClient {
    pub(super) config: ClientConfig,
    pub(super) client: Arc<Client>,
}

impl OllamaClient {
    /// Create a new Ollama client with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration including base URL, timeout, and retry settings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The base URL is invalid or malformed
    /// - The URL scheme is not http or https
    /// - The HTTP client cannot be built
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{OllamaClient, ClientConfig};
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::new(
    ///     "http://localhost:11434".to_string(),
    ///     Duration::from_secs(30),
    ///     3,
    /// )?;
    ///
    /// let client = OllamaClient::new(config)?;
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    pub fn new(config: ClientConfig) -> Result<Self> {
        let client = Client::builder().timeout(config.timeout()).build()?;

        Ok(Self {
            config,
            client: Arc::new(client),
        })
    }

    /// Create client with custom base URL and default timeout/retry
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL for the Ollama API (must include http:// or https://)
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or has an unsupported scheme
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::OllamaClient;
    ///
    /// let client = OllamaClient::with_base_url("http://localhost:8080")?;
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self> {
        let config = ClientConfig::with_base_url(base_url.into())?;
        Self::new(config)
    }

    /// Create client with custom base URL and timeout, default retry settings
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL for the Ollama API (must include http:// or https://)
    /// * `timeout` - Request timeout duration
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or has an unsupported scheme
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::OllamaClient;
    /// use std::time::Duration;
    ///
    /// let client = OllamaClient::with_base_url_and_timeout(
    ///     "http://localhost:8080",
    ///     Duration::from_secs(60),
    /// )?;
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    pub fn with_base_url_and_timeout(
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let config = ClientConfig::with_base_url_and_timeout(base_url.into(), timeout)?;
        Self::new(config)
    }

    /// Create client with default configuration (http://localhost:11434)
    ///
    /// # Errors
    ///
    /// Returns an error if client creation fails (unlikely with default config)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::OllamaClient;
    ///
    /// let client = OllamaClient::default()?;
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self> {
        Self::new(ClientConfig::default())
    }

    /// Execute async HTTP GET request with retry logic
    ///
    /// This helper handles exponential backoff and automatic retries for:
    /// - Network errors
    /// - Server errors (5xx status codes)
    ///
    /// # Type Parameters
    ///
    /// * `T` - Response type that implements `DeserializeOwned`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    /// - Client errors (4xx) occur (no retry)
    pub(super) async fn get_with_retry<T>(&self, url: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        for attempt in 0..=self.config.max_retries() {
            match self.client.get(url).send().await {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                        continue;
                    }

                    // Deserialize and return
                    let result = response.json::<T>().await?;
                    return Ok(result);
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute blocking HTTP GET request with retry logic
    ///
    /// This helper handles exponential backoff and automatic retries for:
    /// - Network errors
    /// - Server errors (5xx status codes)
    ///
    /// # Type Parameters
    ///
    /// * `T` - Response type that implements `DeserializeOwned`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    /// - Client errors (4xx) occur (no retry)
    pub(super) fn get_blocking_with_retry<T>(&self, url: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Create blocking client
        let blocking_client = reqwest::blocking::Client::builder()
            .timeout(self.config.timeout())
            .build()?;

        for attempt in 0..=self.config.max_retries() {
            match blocking_client.get(url).send() {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                        continue;
                    }

                    // Deserialize and return
                    let result = response.json::<T>()?;
                    return Ok(result);
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute async HTTP POST request with retry logic (with JSON response)
    ///
    /// For endpoints that accept a request body and return a JSON response.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Request type that implements `Serialize`
    /// * `T` - Response type that implements `DeserializeOwned`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    /// - Client errors (4xx) occur (no retry)
    pub(super) async fn post_with_retry<R, T>(&self, url: &str, body: &R) -> Result<T>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        for attempt in 0..=self.config.max_retries() {
            match self.client.post(url).json(body).send().await {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                        continue;
                    }

                    // Check for client errors (no retry)
                    if response.status().is_client_error() {
                        return Err(Error::HttpStatusError(response.status().as_u16()));
                    }

                    // Deserialize and return
                    let result = response.json::<T>().await?;
                    return Ok(result);
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute blocking HTTP POST request with retry logic (with JSON response)
    ///
    /// For endpoints that accept a request body and return a JSON response.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Request type that implements `Serialize`
    /// * `T` - Response type that implements `DeserializeOwned`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Response cannot be deserialized
    /// - Client errors (4xx) occur (no retry)
    pub(super) fn post_blocking_with_retry<R, T>(&self, url: &str, body: &R) -> Result<T>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let blocking_client = reqwest::blocking::Client::builder()
            .timeout(self.config.timeout())
            .build()?;

        for attempt in 0..=self.config.max_retries() {
            match blocking_client.post(url).json(body).send() {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                        continue;
                    }

                    // Check for client errors (no retry)
                    if response.status().is_client_error() {
                        return Err(Error::HttpStatusError(response.status().as_u16()));
                    }

                    // Deserialize and return
                    let result = response.json::<T>()?;
                    return Ok(result);
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute async HTTP POST request with retry logic (no response body)
    ///
    /// For endpoints that return 200 OK with empty body.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Request type that implements `Serialize`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Client errors (4xx) occur (no retry)
    #[cfg(feature = "model")]
    pub(super) async fn post_empty_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
    where
        R: serde::Serialize,
    {
        for attempt in 0..=self.config.max_retries() {
            match self.client.post(url).json(body).send().await {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                        continue;
                    }

                    // Check for success status
                    if response.status().is_success() {
                        return Ok(());
                    }

                    // Client error - no retry
                    return Err(Error::HttpStatusError(response.status().as_u16()));
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute blocking HTTP POST request with retry logic (no response body)
    ///
    /// For endpoints that return 200 OK with empty body.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Request type that implements `Serialize`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Client errors (4xx) occur (no retry)
    #[cfg(feature = "model")]
    pub(super) fn post_empty_blocking_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
    where
        R: serde::Serialize,
    {
        let blocking_client = reqwest::blocking::Client::builder()
            .timeout(self.config.timeout())
            .build()?;

        for attempt in 0..=self.config.max_retries() {
            match blocking_client.post(url).json(body).send() {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                        continue;
                    }

                    // Check for success status
                    if response.status().is_success() {
                        return Ok(());
                    }

                    // Client error - no retry
                    return Err(Error::HttpStatusError(response.status().as_u16()));
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute async HTTP DELETE request with retry logic (no response body)
    ///
    /// For endpoints that return 200 OK with empty body.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Request type that implements `Serialize`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Client errors (4xx) occur (no retry)
    #[cfg(feature = "model")]
    pub(super) async fn delete_empty_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
    where
        R: serde::Serialize,
    {
        for attempt in 0..=self.config.max_retries() {
            match self.client.delete(url).json(body).send().await {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                        continue;
                    }

                    // Check for success status
                    if response.status().is_success() {
                        return Ok(());
                    }

                    // Client error - no retry
                    return Err(Error::HttpStatusError(response.status().as_u16()));
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }

    /// Execute blocking HTTP DELETE request with retry logic (no response body)
    ///
    /// For endpoints that return 200 OK with empty body.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Request type that implements `Serialize`
    ///
    /// # Arguments
    ///
    /// * `url` - Full URL to request
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum retry attempts exceeded
    /// - Client errors (4xx) occur (no retry)
    #[cfg(feature = "model")]
    pub(super) fn delete_empty_blocking_with_retry<R>(&self, url: &str, body: &R) -> Result<()>
    where
        R: serde::Serialize,
    {
        let blocking_client = reqwest::blocking::Client::builder()
            .timeout(self.config.timeout())
            .build()?;

        for attempt in 0..=self.config.max_retries() {
            match blocking_client.delete(url).json(body).send() {
                Ok(response) => {
                    // Retry on server errors (5xx)
                    if response.status().is_server_error() && attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                        continue;
                    }

                    // Check for success status
                    if response.status().is_success() {
                        return Ok(());
                    }

                    // Client error - no retry
                    return Err(Error::HttpStatusError(response.status().as_u16()));
                }
                Err(_e) => {
                    // Retry on network errors
                    if attempt < self.config.max_retries() {
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    }
                }
            }
        }

        Err(Error::MaxRetriesExceededError(self.config.max_retries()))
    }
}
